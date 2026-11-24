use serde::Serialize;

use super::markdown::{
    SHEET_SEPARATOR, footer_block, header_block, pq_connections_block, pq_query_block, sheet_block,
    vba_block,
};
use super::template::OutputTemplate;
use super::tokens;
use crate::model::{CellContent, PowerQuery, SheetCells, VbaComponent};

fn summary_block(sheets: &[SheetCells], vba: &[VbaComponent], pq: &PowerQuery) -> String {
    let mut out = String::from("\n## Resumo\n\n");
    for s in sheets {
        let formulas = s
            .cells
            .values()
            .filter(|c| matches!(c, CellContent::Formula { .. }))
            .count();
        out.push_str(&format!(
            "- Aba `{}`: {} célula(s), {} fórmula(s)\n",
            s.name,
            s.cells.len(),
            formulas
        ));
    }
    if !vba.is_empty() {
        let names: Vec<&str> = vba.iter().map(|c| c.name.as_str()).collect();
        out.push_str(&format!(
            "- Componentes VBA ({}): {}\n",
            vba.len(),
            names.join(", ")
        ));
    }
    if !pq.queries.is_empty() {
        let names: Vec<&str> = pq.queries.iter().map(|q| q.name.as_str()).collect();
        out.push_str(&format!(
            "- Consultas Power Query ({}): {}\n",
            pq.queries.len(),
            names.join(", ")
        ));
    }
    out
}

pub fn bundle_units(
    file_name: &str,
    sheets: &[SheetCells],
    vba: &[VbaComponent],
    pq: &PowerQuery,
    t: &OutputTemplate,
    date: &str,
) -> Vec<String> {
    let mut units = vec![header_block(file_name, t, date) + &summary_block(sheets, vba, pq)];
    for (i, sheet) in sheets.iter().enumerate() {
        let sep = if i > 0 { SHEET_SEPARATOR } else { "" };
        units.push(format!("{sep}{}", sheet_block(sheet, file_name, t, date)));
    }
    for (i, c) in vba.iter().enumerate() {
        let sep = if i == 0 { "\n---\n\n" } else { "\n" };
        units.push(format!("{sep}{}", vba_block(c, file_name, t, date)));
    }
    for (i, q) in pq.queries.iter().enumerate() {
        let sep = if i == 0 { "\n---\n\n" } else { "\n" };
        units.push(format!("{sep}{}", pq_query_block(q)));
    }
    if !pq.connections.is_empty() {
        units.push(format!("\n{}", pq_connections_block(&pq.connections)));
    }
    if !t.footer.is_empty() {
        units.push(footer_block(file_name, t, date));
    }
    units
}

pub fn bundle_markdown(
    file_name: &str,
    sheets: &[SheetCells],
    vba: &[VbaComponent],
    pq: &PowerQuery,
    t: &OutputTemplate,
    date: &str,
) -> String {
    bundle_units(file_name, sheets, vba, pq, t, date).concat()
}

pub fn bundle_parts(
    file_name: &str,
    sheets: &[SheetCells],
    vba: &[VbaComponent],
    pq: &PowerQuery,
    t: &OutputTemplate,
    date: &str,
    max_tokens: Option<usize>,
) -> Vec<String> {
    let units = bundle_units(file_name, sheets, vba, pq, t, date);
    match max_tokens {
        Some(n) => tokens::split_units(&units, n),
        None => vec![units.concat()],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BundleEstimate {
    pub tokens: usize,
    pub parts: usize,
}

pub fn estimate_bundle(
    file_name: &str,
    sheets: &[SheetCells],
    vba: &[VbaComponent],
    pq: &PowerQuery,
    t: &OutputTemplate,
    date: &str,
    max_tokens: Option<usize>,
) -> BundleEstimate {
    let units = bundle_units(file_name, sheets, vba, pq, t, date);
    BundleEstimate {
        tokens: units.iter().map(|u| tokens::estimate(u)).sum(),
        parts: match max_tokens {
            Some(n) => tokens::count_parts(&units, n),
            None => 1,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{CellValue, VbaComponentKind};
    use indexmap::IndexMap;

    fn sheet(name: &str, rows: usize) -> SheetCells {
        let mut cells = IndexMap::new();
        for r in 1..=rows {
            cells.insert(
                format!("A{r}"),
                CellContent::Plain(CellValue::String(format!("linha {r}"))),
            );
        }
        cells.insert(
            "B1".into(),
            CellContent::Formula {
                formula: "=SOMA(A1:A2)".into(),
                valor: CellValue::Int(3),
            },
        );
        SheetCells {
            name: name.into(),
            cells,
            tables: vec![],
            sampled: None,
        }
    }

    fn comp(name: &str, lines: usize) -> VbaComponent {
        VbaComponent {
            name: name.into(),
            kind: VbaComponentKind::Module,
            code: (0..lines)
                .map(|i| format!("' linha {i}"))
                .collect::<Vec<_>>()
                .join("\n"),
            line_count: lines,
            procedures: vec![],
        }
    }

    fn dados() -> (Vec<SheetCells>, Vec<VbaComponent>) {
        (
            vec![sheet("Aba1", 30), sheet("Aba2", 5)],
            vec![comp("Module1", 20), comp("Classe1", 3)],
        )
    }

    fn t() -> OutputTemplate {
        OutputTemplate::default()
    }

    #[test]
    fn arquivo_unico_tem_resumo_celulas_e_vba() {
        let (s, v) = dados();
        let md = bundle_markdown("a.xlsm", &s, &v, &PowerQuery::default(), &t(), "2026-01-02");
        assert!(md.starts_with("# Mapeamento de Estrutura: a.xlsm\n"));
        assert!(md.contains("## Resumo"));
        assert!(md.contains("- Aba `Aba1`: 31 célula(s), 1 fórmula(s)"));
        assert!(md.contains("- Componentes VBA (2): Module1, Classe1"));
        assert!(md.contains("## Aba: Aba1") && md.contains("## Aba: Aba2"));
        assert!(md.contains("## COMPONENTE: Module1") && md.contains("## COMPONENTE: Classe1"));
        assert!(md.find("## Aba: Aba2") < md.find("## COMPONENTE: Module1"));
    }

    #[test]
    fn sem_vba_omite_a_secao() {
        let (s, _) = dados();
        let md = bundle_markdown("a.xlsx", &s, &[], &PowerQuery::default(), &t(), "d");
        assert!(!md.contains("COMPONENTE") && !md.contains("Componentes VBA"));
    }

    #[test]
    fn power_query_no_arquivo_unico() {
        use crate::model::{ConnectionInfo, PowerQueryItem};
        let (s, v) = dados();
        let pq = PowerQuery {
            queries: vec![PowerQueryItem {
                name: "Relatório Mensal".into(),
                code: "let x = 1 in x".into(),
            }],
            connections: vec![ConnectionInfo {
                name: "Consulta - Vendas".into(),
                description: String::new(),
                connection: "Location=Vendas".into(),
            }],
        };
        let md = bundle_markdown("a.xlsm", &s, &v, &pq, &t(), "d");
        assert!(md.contains("- Consultas Power Query (1): Relatório Mensal\n"));
        assert!(md.contains("## Consulta: Relatório Mensal\n\n```m\nlet x = 1 in x\n```\n"));
        assert!(md.contains("## Conexões\n\n### Consulta - Vendas"));
        let partes = bundle_parts("a.xlsm", &s, &v, &pq, &t(), "d", Some(1));
        assert!(partes.len() > 1);
        assert_eq!(partes.concat(), md);
    }

    #[test]
    fn rodape_e_intro_do_modelo() {
        let (s, v) = dados();
        let tpl = OutputTemplate {
            intro: "Contexto de {arquivo}.".into(),
            footer: "Fim.".into(),
            ..t()
        };
        let md = bundle_markdown("a.xlsm", &s, &v, &PowerQuery::default(), &tpl, "d");
        assert!(md.contains("\nContexto de a.xlsm.\n"));
        assert!(md.ends_with("\nFim.\n"));
    }

    #[test]
    fn partes_juntas_reproduzem_o_arquivo_unico() {
        let (s, v) = dados();
        let todo = bundle_markdown("a.xlsm", &s, &v, &PowerQuery::default(), &t(), "d");
        for max in [1, 50, 200, 500, 100_000] {
            let partes = bundle_parts(
                "a.xlsm",
                &s,
                &v,
                &PowerQuery::default(),
                &t(),
                "d",
                Some(max),
            );
            assert_eq!(partes.concat(), todo, "max={max}");
        }
        let uma = bundle_parts("a.xlsm", &s, &v, &PowerQuery::default(), &t(), "d", None);
        assert_eq!(uma, vec![todo]);
    }

    #[test]
    fn nenhuma_parte_quebra_bloco_de_codigo_ou_tabela() {
        let (s, v) = dados();
        for max in [1, 50, 100, 200] {
            let partes = bundle_parts(
                "a.xlsm",
                &s,
                &v,
                &PowerQuery::default(),
                &t(),
                "d",
                Some(max),
            );
            assert!(partes.len() > 1, "max={max} deveria dividir");
            for (i, p) in partes.iter().enumerate() {
                assert_eq!(p.matches("```").count() % 2, 0, "parte {i} max={max}");
                if p.contains("| **") {
                    assert!(p.contains("| Célula |"), "parte {i} sem cabeçalho");
                }
                for line in p.lines().filter(|l| l.starts_with("| **")) {
                    assert!(line.ends_with(" |"), "linha cortada: {line}");
                }
            }
        }
    }

    #[test]
    fn estimativa_bate_com_as_partes() {
        let (s, v) = dados();
        let e = estimate_bundle("a.xlsm", &s, &v, &PowerQuery::default(), &t(), "d", Some(1));
        assert_eq!(e.parts, 5);
        let unico = estimate_bundle("a.xlsm", &s, &v, &PowerQuery::default(), &t(), "d", None);
        assert_eq!(unico.parts, 1);
        assert!(unico.tokens > 0);
        let todo = tokens::estimate(&bundle_markdown(
            "a.xlsm",
            &s,
            &v,
            &PowerQuery::default(),
            &t(),
            "d",
        ));
        assert!(unico.tokens >= todo && unico.tokens <= todo + 5);
    }

    #[test]
    fn corte_so_em_limite_de_secao() {
        let (s, v) = dados();
        let partes = bundle_parts("a.xlsm", &s, &v, &PowerQuery::default(), &t(), "d", Some(1));
        assert_eq!(partes.len(), 5);
        assert!(partes[1].contains("## Aba: Aba1") && !partes[1].contains("Aba2"));
        assert!(partes[3].contains("## COMPONENTE: Module1") && !partes[3].contains("Classe1"));
    }
}
