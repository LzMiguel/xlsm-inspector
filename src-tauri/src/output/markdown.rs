use std::borrow::Cow;
use std::fmt::Write;

use indexmap::IndexMap;

use super::template::{OutputTemplate, fill};
use crate::model::{
    CellContent, CellValue, ConnectionInfo, PowerQuery, PowerQueryItem, SheetCells, VbaComponent,
    VbaComponentKind,
};

fn value_text(v: &CellValue) -> String {
    match v {
        CellValue::Empty => String::new(),
        CellValue::Bool(true) => "True".into(),
        CellValue::Bool(false) => "False".into(),
        CellValue::Int(i) => i.to_string(),
        CellValue::Float(f) => f.to_string(),
        CellValue::String(s) | CellValue::DateTime(s) | CellValue::Error(s) => s.clone(),
    }
}

/// Como no script original: `|` vira `\|` e quebra de linha vira `|`.
fn escape(s: &str) -> Cow<'_, str> {
    if !s.contains(['|', '\r', '\n']) {
        return Cow::Borrowed(s);
    }
    let mut out = String::with_capacity(s.len() + 8);
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '|' => out.push_str("\\|"),
            '\r' if chars.peek() == Some(&'\n') => {}
            '\n' => out.push('|'),
            _ => out.push(c),
        }
    }
    Cow::Owned(out)
}

/// 52000 → "52.000"
fn dots(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(c);
    }
    out
}

/// `file_name` é o nome do arquivo de entrada, com extensão; `date`, `AAAA-MM-DD`.
pub fn cells_to_markdown(
    file_name: &str,
    sheets: &[SheetCells],
    t: &OutputTemplate,
    date: &str,
) -> String {
    let mut out = header_block(file_name, t, date);
    for (i, sheet) in sheets.iter().enumerate() {
        if i > 0 {
            out.push_str(SHEET_SEPARATOR);
        }
        out.push_str(&sheet_block(sheet, file_name, t, date));
    }
    if !t.footer.is_empty() {
        out.push_str(&footer_block(file_name, t, date));
    }
    out
}

/// Entre uma aba e a seguinte.
pub(super) const SHEET_SEPARATOR: &str = "\n---\n";

/// Título e texto de abertura (`# título\n` + `\nintro\n`, se houver).
pub(super) fn header_block(file_name: &str, t: &OutputTemplate, date: &str) -> String {
    let vars = [("arquivo", file_name), ("data", date)];
    let mut out = format!("{}\n", fill(&t.title, &vars));
    if !t.intro.is_empty() {
        out.push_str(&format!("\n{}\n", fill(&t.intro, &vars)));
    }
    out
}

pub(super) fn footer_block(file_name: &str, t: &OutputTemplate, date: &str) -> String {
    format!(
        "\n{}\n",
        fill(&t.footer, &[("arquivo", file_name), ("data", date)])
    )
}

/// Uma aba inteira: título, aviso de amostra, cabeçalho da tabela e todas as linhas.
pub(super) fn sheet_block(
    sheet: &SheetCells,
    file_name: &str,
    t: &OutputTemplate,
    date: &str,
) -> String {
    let heading = fill(
        &t.sheet_heading,
        &[("arquivo", file_name), ("data", date), ("aba", &sheet.name)],
    );
    let note = sheet
        .sampled
        .map(|s| {
            format!(
                "\n> Amostra: {} de {} linhas.\n",
                dots(s.kept_rows),
                dots(s.total_rows)
            )
        })
        .unwrap_or_default();
    let mut out = String::with_capacity(sheet.cells.len().saturating_mul(48) + 128);
    let _ = write!(
        out,
        "\n{heading}\n{note}\n| {} | {} |\n| ------ | ----------------- |\n",
        escape(&t.cell_header),
        escape(&t.content_header)
    );
    let formula_label = escape(&t.formula_label);
    let value_label = escape(&t.value_label);
    for (coord, content) in &sheet.cells {
        match content {
            CellContent::Plain(v) => {
                let val = value_text(v);
                let _ = writeln!(out, "| **{coord}** | {} |", escape(&val));
            }
            CellContent::Formula { formula, valor } => {
                let val = value_text(valor);
                let _ = writeln!(
                    out,
                    "| **{coord}** | `{formula_label}: {}` <br> **{value_label}:** {} |",
                    escape(formula),
                    escape(&val)
                );
            }
        }
    }
    out
}

/// Um componente: título + bloco de código.
pub(super) fn vba_block(
    c: &VbaComponent,
    file_name: &str,
    t: &OutputTemplate,
    date: &str,
) -> String {
    let heading = fill(
        &t.vba_heading,
        &[
            ("arquivo", file_name),
            ("data", date),
            ("componente", &c.name),
        ],
    );
    let fence = fence_for(&c.code);
    format!("{heading}\n\n{fence}vba\n{}\n{fence}\n", c.code)
}

pub fn vba_to_markdown(
    components: &[VbaComponent],
    file_name: &str,
    t: &OutputTemplate,
    date: &str,
) -> String {
    components
        .iter()
        .map(|c| vba_block(c, file_name, t, date))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Uma consulta: título e código `m`.
pub fn pq_query_block(q: &PowerQueryItem) -> String {
    let fence = fence_for(&q.code);
    format!("## Consulta: {}\n\n{fence}m\n{}\n{fence}\n", q.name, q.code)
}

/// Seção "Conexões": nome, descrição e string de conexão de cada uma.
pub fn pq_connections_block(connections: &[ConnectionInfo]) -> String {
    let mut out = String::from("## Conexões\n");
    for c in connections {
        let _ = write!(out, "\n### {}\n", c.name);
        if !c.description.is_empty() {
            let _ = write!(out, "\n{}\n", c.description);
        }
        if !c.connection.is_empty() {
            let fence = fence_for(&c.connection);
            let _ = write!(out, "\n{fence}\n{}\n{fence}\n", c.connection);
        }
    }
    out
}

pub fn power_query_to_markdown(pq: &PowerQuery, file_name: &str) -> String {
    let mut out = format!("# Power Query: {file_name}\n");
    for q in &pq.queries {
        let _ = write!(out, "\n{}", pq_query_block(q));
    }
    if !pq.connections.is_empty() {
        let _ = write!(out, "\n{}", pq_connections_block(&pq.connections));
    }
    out
}

/// Cerca de crases maior que qualquer sequência de crases dentro do texto.
fn fence_for(text: &str) -> String {
    let longest = text.split(|c| c != '`').map(str::len).max().unwrap_or(0);
    "`".repeat((longest + 1).max(3))
}

/// Pré-visualização do modelo com dados fictícios (mapeamento + VBA).
pub fn preview(t: &OutputTemplate, date: &str) -> String {
    let file = "exemplo.xlsm";
    let mut cells = IndexMap::new();
    cells.insert(
        "A1".to_string(),
        CellContent::Plain(CellValue::String("Total".into())),
    );
    cells.insert(
        "B1".to_string(),
        CellContent::Formula {
            formula: "=SOMA(B2:B9)".into(),
            valor: CellValue::Int(84),
        },
    );
    let sheets = [SheetCells {
        name: "Resumo".into(),
        cells,
        tables: vec![],
        sampled: None,
    }];
    let code = "Sub Exemplo()\n    MsgBox \"Olá\"\nEnd Sub";
    let vba = [VbaComponent {
        name: "Module1".into(),
        kind: VbaComponentKind::Module,
        code: code.into(),
        line_count: 3,
        procedures: vec![],
    }];
    format!(
        "{}\n<!-- vba_code.md -->\n\n{}",
        cells_to_markdown(file, &sheets, t, date),
        vba_to_markdown(&vba, file, t, date)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_query_markdown() {
        let pq = PowerQuery {
            queries: vec![PowerQueryItem {
                name: "Relatório Mensal".into(),
                code: "let x = \"```\" in x".into(),
            }],
            connections: vec![ConnectionInfo {
                name: "Consulta - Vendas".into(),
                description: String::new(),
                connection: "Location=Vendas".into(),
            }],
        };
        let md = power_query_to_markdown(&pq, "a.xlsx");
        assert!(md.starts_with("# Power Query: a.xlsx\n"));
        assert!(md.contains("## Consulta: Relatório Mensal\n\n````m\nlet x"));
        assert!(md.contains("## Conexões\n\n### Consulta - Vendas\n\n```\nLocation=Vendas\n```\n"));
    }

    fn sheets() -> Vec<SheetCells> {
        let mut cells = IndexMap::new();
        cells.insert(
            "A1".to_string(),
            CellContent::Plain(CellValue::String("a|b\nc".into())),
        );
        cells.insert(
            "C3".to_string(),
            CellContent::Formula {
                formula: "=SOMA(B1:B2)".into(),
                valor: CellValue::Int(84),
            },
        );
        vec![
            SheetCells {
                name: "Aba1".into(),
                cells,
                tables: vec![],
                sampled: None,
            },
            SheetCells {
                name: "Vazia".into(),
                cells: IndexMap::new(),
                tables: vec![],
                sampled: None,
            },
        ]
    }

    fn comp(name: &str, code: &str) -> VbaComponent {
        VbaComponent {
            name: name.into(),
            kind: VbaComponentKind::Module,
            code: code.into(),
            line_count: 1,
            procedures: vec![],
        }
    }

    /// Saída da v1.0, escrita à mão: o modelo padrão não pode mudar nenhum byte.
    #[test]
    fn modelo_padrao_igual_a_v1() {
        let esperado = "# Mapeamento de Estrutura: arq.xlsm\n\
            \n## Aba: Aba1\n\n| Célula | Conteúdo Extraído |\n| ------ | ----------------- |\n\
            | **A1** | a\\|b|c |\n\
            | **C3** | `Fórmula: =SOMA(B1:B2)` <br> **Valor:** 84 |\n\
            \n---\n\
            \n## Aba: Vazia\n\n| Célula | Conteúdo Extraído |\n| ------ | ----------------- |\n";
        let t = OutputTemplate::default();
        assert_eq!(
            cells_to_markdown("arq.xlsm", &sheets(), &t, "2026-01-02"),
            esperado
        );

        let vba = vba_to_markdown(
            &[comp("Module1", "Sub A()\nEnd Sub"), comp("Classe1", "x")],
            "arq.xlsm",
            &t,
            "2026-01-02",
        );
        assert_eq!(
            vba,
            "## COMPONENTE: Module1\n\n```vba\nSub A()\nEnd Sub\n```\n\n## COMPONENTE: Classe1\n\n```vba\nx\n```\n"
        );
        assert_eq!(vba_to_markdown(&[], "arq.xlsm", &t, "2026-01-02"), "");
    }

    #[test]
    fn modelo_personalizado() {
        let t = OutputTemplate {
            title: "# {arquivo} em {data}".into(),
            intro: "Leia com atenção.".into(),
            sheet_heading: "### {aba}".into(),
            cell_header: "Ref".into(),
            content_header: "Texto".into(),
            formula_label: "Formula".into(),
            value_label: "Resultado".into(),
            vba_heading: "## Módulo {componente}".into(),
            footer: "Fim ({arquivo})".into(),
        };
        let md = cells_to_markdown("arq.xlsm", &sheets(), &t, "2026-01-02");
        assert!(md.starts_with(
            "# arq.xlsm em 2026-01-02\n\nLeia com atenção.\n\n### Aba1\n\n| Ref | Texto |\n"
        ));
        assert!(md.contains("| **C3** | `Formula: =SOMA(B1:B2)` <br> **Resultado:** 84 |"));
        assert!(md.ends_with("\nFim (arq.xlsm)\n"));
        assert!(
            vba_to_markdown(&[comp("M", "x")], "arq.xlsm", &t, "d").starts_with("## Módulo M\n")
        );
    }

    #[test]
    fn aviso_de_amostra() {
        use crate::model::SampleInfo;
        let mut s = sheets();
        s[0].sampled = Some(SampleInfo {
            total_rows: 52_000,
            kept_rows: 100,
        });
        let md = cells_to_markdown("arq.xlsm", &s, &OutputTemplate::default(), "d");
        assert!(md.contains("## Aba: Aba1\n\n> Amostra: 100 de 52.000 linhas.\n\n| Célula"));
        assert_eq!(md.matches("Amostra:").count(), 1); // só a aba cortada
        assert_eq!(
            [dots(7), dots(1000), dots(1_234_567)],
            ["7", "1.000", "1.234.567"]
        );
    }

    #[test]
    fn previa_tem_cells_e_vba() {
        let p = preview(&OutputTemplate::default(), "2026-01-02");
        assert!(p.contains("## Aba: Resumo") && p.contains("## COMPONENTE: Module1"));
    }

    #[test]
    fn vba_com_crases_usa_cerca_maior() {
        let md = vba_to_markdown(
            &[comp("M", "' exemplo ```\nSub A()\nEnd Sub")],
            "a.xlsm",
            &OutputTemplate::default(),
            "d",
        );
        assert!(md.contains("````vba\n"));
        assert!(md.ends_with("\n````\n"));
    }
}
