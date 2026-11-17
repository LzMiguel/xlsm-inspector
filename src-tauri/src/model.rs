use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkbookSummary {
    pub path: String,
    pub file_name: String,
    pub size_bytes: u64,
    pub sheets: Vec<SheetInfo>,
    pub has_vba: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetInfo {
    pub name: String,
    pub cell_count: usize,
    pub formula_count: usize,
}

/// Serializado "untagged": `null`, bool, número ou texto.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CellContent {
    Plain(CellValue),
    Formula { formula: String, valor: CellValue },
}

/// Tabela do Excel (ListObject): só metadado; o conteúdo continua em `SheetCells::cells`.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Quantas linhas (com conteúdo) da aba ficaram na extração.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SampleInfo {
    pub total_rows: usize,
    pub kept_rows: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetCells {
    pub name: String,
    pub cells: IndexMap<String, CellContent>,
    pub tables: Vec<TableInfo>,
    /// `Some` só quando o escopo cortou linhas desta aba.
    pub sampled: Option<SampleInfo>,
}

/// Quais linhas extrair. Reduz saída e renderização; o arquivo continua sendo lido por inteiro.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExtractScope {
    #[default]
    Full,
    /// As primeiras `rows` linhas com conteúdo de cada aba.
    Head { rows: usize },
    /// `rows` linhas com conteúdo espaçadas por igual (determinístico, inclui a primeira).
    Sample { rows: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VbaComponentKind {
    Module,
    Class,
    Document,
    Workbook,
    UserForm,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VbaComponent {
    pub name: String,
    pub kind: VbaComponentKind,
    pub code: String,
    pub line_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningKind {
    /// Fórmulas sem valor em cache (arquivo nunca salvo no Excel).
    MissingCache,
    /// A aba foi cortada pelo escopo (primeiras linhas ou amostra).
    Sampled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        Self {
            stage,
            current,
            total,
            label: label.to_string(),
        }
    }
}

/// Célula com problema: `detail` é o erro (`#DIV/0!`) ou a fórmula com referência externa.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CellIssue {
    pub sheet: String,
    pub cell: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DefinedName {
    pub name: String,
    pub refers_to: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HiddenSheet {
    pub name: String,
    /// "muito oculta": só aparece no editor VBA, não no menu "Reexibir".
    pub very_hidden: bool,
}

/// Tabela ou tabela dinâmica de uma aba (`range` só nas tabelas).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SheetObject {
    pub sheet: String,
    pub name: String,
    pub range: Option<String>,
}

/// Visão geral dos problemas e das peças da planilha (RF13). As listas de células são cortadas
/// em `MAX_LISTED`; `*_count` tem o total real.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Diagnostics {
    pub errors: Vec<CellIssue>,
    pub error_count: usize,
    pub external_refs: Vec<CellIssue>,
    pub external_count: usize,
    pub names: Vec<DefinedName>,
    pub hidden_sheets: Vec<HiddenSheet>,
    pub tables: Vec<SheetObject>,
    pub pivots: Vec<SheetObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
