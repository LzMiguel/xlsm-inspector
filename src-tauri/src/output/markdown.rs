use indexmap::IndexMap;

use super::template::{OutputTemplate, fill};
use crate::model::{CellContent, CellValue, SheetCells, VbaComponent, VbaComponentKind};

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
fn escape(s: &str) -> String {
    s.replace('|', "\\|").replace("\r\n", "\n").replace('\n', "|")
}

/// `file_name` é o nome do arquivo de entrada, com extensão; `date`, `AAAA-MM-DD`.
pub fn cells_to_markdown(
    file_name: &str,
    sheets: &[SheetCells],
    t: &OutputTemplate,
    date: &str,
) -> String {
    let vars = [("arquivo", file_name), ("data", date)];
    let mut out = format!("{}\n", fill(&t.title, &vars));
    if !t.intro.is_empty() {
        out.push_str(&format!("\n{}\n", fill(&t.intro, &vars)));
    }
    for (i, sheet) in sheets.iter().enumerate() {
        if i > 0 {
            out.push_str("\n---\n");
        }
        let heading = fill(&t.sheet_heading, &[vars[0], vars[1], ("aba", &sheet.name)]);
        out.push_str(&format!(
            "\n{heading}\n\n| {} | {} |\n| ------ | ----------------- |\n",
            escape(&t.cell_header),
            escape(&t.content_header)
        ));
        for (coord, content) in &sheet.cells {
            let text = match content {
                CellContent::Plain(v) => escape(&value_text(v)),
                CellContent::Formula { formula, valor } => format!(
                    "`{}: {}` <br> **{}:** {}",
                    t.formula_label,
                    escape(formula),
                    t.value_label,
                    escape(&value_text(valor))
                ),
            };
            out.push_str(&format!("| **{coord}** | {text} |\n"));
        }
    }
    if !t.footer.is_empty() {
        out.push_str(&format!("\n{}\n", fill(&t.footer, &vars)));
    }
    out
}

pub fn vba_to_markdown(
    components: &[VbaComponent],
    file_name: &str,
    t: &OutputTemplate,
    date: &str,
) -> String {
    components
        .iter()
        .map(|c| {
            let heading = fill(
                &t.vba_heading,
                &[("arquivo", file_name), ("data", date), ("componente", &c.name)],
            );
            format!("{heading}\n\n```vba\n{}\n```\n", c.code)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Pré-visualização do modelo com dados fictícios (mapeamento + VBA).
pub fn preview(t: &OutputTemplate, date: &str) -> String {
    let file = "exemplo.xlsm";
    let mut cells = IndexMap::new();
    cells.insert("A1".to_string(), CellContent::Plain(CellValue::String("Total".into())));
    cells.insert(
        "B1".to_string(),
        CellContent::Formula { formula: "=SOMA(B2:B9)".into(), valor: CellValue::Int(84) },
    );
    let sheets = [SheetCells { name: "Resumo".into(), cells, tables: vec![] }];
    let code = "Sub Exemplo()\n    MsgBox \"Olá\"\nEnd Sub";
    let vba = [VbaComponent {
        name: "Module1".into(),
        kind: VbaComponentKind::Module,
        code: code.into(),
        line_count: 3,
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

    fn sheets() -> Vec<SheetCells> {
        let mut cells = IndexMap::new();
        cells.insert("A1".to_string(), CellContent::Plain(CellValue::String("a|b\nc".into())));
        cells.insert(
            "C3".to_string(),
            CellContent::Formula { formula: "=SOMA(B1:B2)".into(), valor: CellValue::Int(84) },
        );
        vec![
            SheetCells { name: "Aba1".into(), cells, tables: vec![] },
            SheetCells { name: "Vazia".into(), cells: IndexMap::new(), tables: vec![] },
        ]
    }

    fn comp(name: &str, code: &str) -> VbaComponent {
        VbaComponent {
            name: name.into(),
            kind: VbaComponentKind::Module,
            code: code.into(),
            line_count: 1,
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
        assert_eq!(cells_to_markdown("arq.xlsm", &sheets(), &t, "2026-01-02"), esperado);

        let vba = vba_to_markdown(&[comp("Module1", "Sub A()\nEnd Sub"), comp("Classe1", "x")], "arq.xlsm", &t, "2026-01-02");
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
        assert!(md.starts_with("# arq.xlsm em 2026-01-02\n\nLeia com atenção.\n\n### Aba1\n\n| Ref | Texto |\n"));
        assert!(md.contains("| **C3** | `Formula: =SOMA(B1:B2)` <br> **Resultado:** 84 |"));
        assert!(md.ends_with("\nFim (arq.xlsm)\n"));
        assert!(vba_to_markdown(&[comp("M", "x")], "arq.xlsm", &t, "d").starts_with("## Módulo M\n"));
    }

    #[test]
    fn previa_tem_cells_e_vba() {
        let p = preview(&OutputTemplate::default(), "2026-01-02");
        assert!(p.contains("## Aba: Resumo") && p.contains("## COMPONENTE: Module1"));
    }
}
