use crate::model::{CellContent, CellValue, SheetCells, VbaComponent};

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

pub fn cells_to_markdown(file_name: &str, sheets: &[SheetCells]) -> String {
    let mut out = format!("# Mapeamento de Estrutura: {file_name}\n");
    for (i, sheet) in sheets.iter().enumerate() {
        if i > 0 {
            out.push_str("\n---\n");
        }
        out.push_str(&format!(
            "\n## Aba: {}\n\n| Célula | Conteúdo Extraído |\n| ------ | ----------------- |\n",
            sheet.name
        ));
        for (coord, content) in &sheet.cells {
            let text = match content {
                CellContent::Plain(v) => escape(&value_text(v)),
                CellContent::Formula { formula, valor } => format!(
                    "`Fórmula: {}` <br> **Valor:** {}",
                    escape(formula),
                    escape(&value_text(valor))
                ),
            };
            out.push_str(&format!("| **{coord}** | {text} |\n"));
        }
    }
    out
}

pub fn vba_to_markdown(components: &[VbaComponent]) -> String {
    components
        .iter()
        .map(|c| format!("## COMPONENTE: {}\n\n```vba\n{}\n```\n", c.name, c.code))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::VbaComponentKind;
    use indexmap::IndexMap;

    #[test]
    fn celulas() {
        let mut cells = IndexMap::new();
        cells.insert("A1".to_string(), CellContent::Plain(CellValue::String("a|b\nc".into())));
        cells.insert(
            "C3".to_string(),
            CellContent::Formula { formula: "=SOMA(B1:B2)".into(), valor: CellValue::Int(84) },
        );
        let sheets = vec![
            SheetCells { name: "Aba1".into(), cells },
            SheetCells { name: "Vazia".into(), cells: IndexMap::new() },
        ];
        let md = cells_to_markdown("arq.xlsm", &sheets);

        assert!(md.starts_with("# Mapeamento de Estrutura: arq.xlsm\n"));
        assert!(md.contains("## Aba: Aba1"));
        assert!(md.contains("| **A1** | a\\|b|c |"));
        assert!(md.contains("| **C3** | `Fórmula: =SOMA(B1:B2)` <br> **Valor:** 84 |"));
        assert!(md.contains("\n---\n"));
        assert!(md.contains("## Aba: Vazia"));
    }

    #[test]
    fn vba() {
        let c = |name: &str, code: &str| VbaComponent {
            name: name.into(),
            kind: VbaComponentKind::Module,
            code: code.into(),
            line_count: 1,
        };
        let md = vba_to_markdown(&[c("Module1", "Sub A()\nEnd Sub"), c("Classe1", "x")]);
        assert!(md.starts_with("## COMPONENTE: Module1\n\n```vba\nSub A()\nEnd Sub\n```\n"));
        assert!(md.contains("## COMPONENTE: Classe1"));
        assert_eq!(vba_to_markdown(&[]), "");
    }
}
