use indexmap::IndexMap;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct WorkbookSummary {
    pub path: String,
    pub file_name: String,
    pub size_bytes: u64,
    pub sheets: Vec<SheetInfo>,
    pub has_vba: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SheetInfo {
    pub name: String,
    pub cell_count: usize,
    pub formula_count: usize,
}

/// Serializado "untagged": `null`, bool, número ou texto.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum CellValue {
    Empty,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    DateTime(String),
    Error(String),
}

/// Serializado "untagged": valor simples ou `{ "formula", "valor" }`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum CellContent {
    Plain(CellValue),
    Formula { formula: String, valor: CellValue },
}

#[derive(Debug, Clone, Serialize)]
pub struct SheetCells {
    pub name: String,
    pub cells: IndexMap<String, CellContent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum VbaComponentKind {
    Module,
    Class,
    Document,
    Workbook,
    UserForm,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
pub struct VbaComponent {
    pub name: String,
    pub kind: VbaComponentKind,
    pub code: String,
    pub line_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum WarningKind {
    /// Fórmulas sem valor em cache (arquivo nunca salvo no Excel).
    MissingCache,
}

#[derive(Debug, Clone, Serialize)]
pub struct Warning {
    pub kind: WarningKind,
    pub sheet: String,
    /// Quantidade de células afetadas.
    pub count: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtractionResult {
    pub sheets: Vec<SheetCells>,
    pub vba: Vec<VbaComponent>,
    pub warnings: Vec<Warning>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn conteudo_serializa_como_python() {
        let plain = CellContent::Plain(CellValue::String("x".into()));
        assert_eq!(serde_json::to_value(plain).unwrap(), json!("x"));

        let f = CellContent::Formula {
            formula: "=A1".into(),
            valor: CellValue::Int(84),
        };
        assert_eq!(
            serde_json::to_value(f).unwrap(),
            json!({"formula": "=A1", "valor": 84})
        );
        assert_eq!(serde_json::to_value(CellValue::Empty).unwrap(), json!(null));
    }
}
