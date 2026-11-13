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

/// Tabela do Excel (ListObject): só metadado; o conteúdo continua em `SheetCells::cells`.
#[derive(Debug, Clone, Serialize)]
pub struct TableInfo {
    pub name: String,
    /// Intervalo com cabeçalho, ex.: "A1:Z300".
    pub range: String,
    /// Nomes das colunas (cabeçalho).
    pub columns: Vec<String>,
    /// Corpo da tabela, linha a linha (sem o cabeçalho).
    pub rows: Vec<Vec<CellContent>>,
    /// Coordenadas (chaves de `cells`) que pertencem à tabela.
    pub cells: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SheetCells {
    pub name: String,
    pub cells: IndexMap<String, CellContent>,
    pub tables: Vec<TableInfo>,
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

/// Payload do evento `extraction://progress`.
#[derive(Debug, Clone, Serialize)]
pub struct Progress {
    pub stage: &'static str, // "cells" | "vba"
    pub current: usize,
    pub total: usize,
    pub label: String,
}

impl Progress {
    pub fn new(stage: &'static str, current: usize, total: usize, label: &str) -> Self {
        Self { stage, current, total, label: label.to_string() }
    }
}

/// Saída do comando `extract_cells` (células + avisos).
#[derive(Debug, Clone, Serialize)]
pub struct CellsExtraction {
    pub sheets: Vec<SheetCells>,
    pub warnings: Vec<Warning>,
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
