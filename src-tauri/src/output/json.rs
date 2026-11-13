use indexmap::IndexMap;

use crate::error::AppError;
use crate::model::{CellContent, SheetCells};

/// `{ "Aba": { "A1": valor | {formula, valor} } }`, ordem preservada, UTF-8 sem escape.
pub fn cells_to_json(sheets: &[SheetCells]) -> Result<String, AppError> {
    let map: IndexMap<&str, &IndexMap<String, CellContent>> =
        sheets.iter().map(|s| (s.name.as_str(), &s.cells)).collect();
    serde_json::to_string_pretty(&map).map_err(|e| AppError::WriteFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::CellValue;

    #[test]
    fn formato_do_prd() {
        let mut cells = IndexMap::new();
        cells.insert("B1".to_string(), CellContent::Plain(CellValue::String("Olá".into())));
        cells.insert(
            "A1".to_string(),
            CellContent::Formula {
                formula: "=SOMA(B1:B2)".into(),
                valor: CellValue::Int(84),
            },
        );
        let sheets = vec![
            SheetCells { name: "Zeta".into(), cells, tables: vec![] },
            SheetCells { name: "Alfa".into(), cells: IndexMap::new(), tables: vec![] },
        ];
        let json = cells_to_json(&sheets).unwrap();

        // ordem preservada (Zeta antes de Alfa, B1 antes de A1) e acentos intactos
        assert!(json.find("Zeta").unwrap() < json.find("Alfa").unwrap());
        assert!(json.find("B1").unwrap() < json.find("A1").unwrap());
        assert!(json.contains("Olá"));
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["Zeta"]["A1"]["formula"], "=SOMA(B1:B2)");
        assert_eq!(v["Zeta"]["A1"]["valor"], 84);
    }
}
