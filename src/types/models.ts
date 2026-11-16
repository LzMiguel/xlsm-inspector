// Espelho EXATO de src-tauri/src/model.rs e error.rs. Alterar os dois juntos.
// Campos em snake_case, como serializados pelo Rust.

export interface SheetInfo {
  name: string;
  cell_count: number;
  formula_count: number;
}

export interface WorkbookSummary {
  path: string;
  file_name: string;
  size_bytes: number;
  sheets: SheetInfo[];
  has_vba: boolean;
}

// CellValue é "untagged" no Rust: Empty = null; DateTime e Error chegam como string.
export type CellValue = null | boolean | number | string;

export interface FormulaContent {
  formula: string;
  valor: CellValue;
}

export type CellContent = CellValue | FormulaContent;

/** Tabela do Excel (ListObject): só metadado; o conteúdo está em `SheetCells.cells`. */
export interface TableInfo {
  name: string;
  /** Intervalo com cabeçalho, ex.: "A1:Z300". */
  range: string;
  columns: string[];
  /** Corpo da tabela, linha a linha (sem o cabeçalho). */
  rows: CellContent[][];
  /** Coordenadas (chaves de `cells`) que pertencem à tabela. */
  cells: string[];
}

export interface SheetCells {
  name: string;
  /** "A1" -> conteúdo, na ordem da planilha. */
  cells: Record<string, CellContent>;
  tables: TableInfo[];
}

export type VbaComponentKind =
  | "Module"
  | "Class"
  | "Document"
  | "Workbook"
  | "UserForm"
  | "Unknown";

export interface VbaComponent {
  name: string;
  kind: VbaComponentKind;
  code: string;
  line_count: number;
}

export type WarningKind = "MissingCache";

export interface Warning {
  kind: WarningKind;
  sheet: string;
  /** Quantidade de células afetadas. */
  count: number;
  message: string;
}

export interface CellsExtraction {
  sheets: SheetCells[];
  warnings: Warning[];
}

export interface ExtractionResult {
  sheets: SheetCells[];
  vba: VbaComponent[];
  warnings: Warning[];
}

/** Payload do evento `extraction://progress`. */
export interface Progress {
  stage: "cells" | "vba";
  current: number;
  total: number;
  label: string;
}

export interface VbaOptions {
  remove_blank_lines: boolean;
  hide_attributes: boolean;
}

/** Espelho de store/settings.rs. */
export interface Settings {
  version: number;
  /** Pasta onde o diálogo de exportação abre. */
  default_out_dir: string | null;
  vba_options: VbaOptions;
  file_names: FileNames;
  /** Modelo ativo do Markdown. */
  template: OutputTemplate;
  /** Predefinições do usuário (as embutidas vêm de `builtin_templates`). */
  presets: TemplatePreset[];
}

/** Espelho de output/naming.rs: padrões sem extensão. `{arquivo}` e `{data}`. */
export interface FileNames {
  json: string;
  cells_md: string;
  vba_md: string;
}

/** Espelho de output/template.rs. `{arquivo}` (com extensão), `{data}`, `{aba}`, `{componente}`. */
export interface OutputTemplate {
  title: string;
  intro: string;
  sheet_heading: string;
  cell_header: string;
  content_header: string;
  formula_label: string;
  value_label: string;
  vba_heading: string;
  footer: string;
}

export interface TemplatePreset {
  name: string;
  template: OutputTemplate;
}

/** Arquivo de destino já existe: `fail` devolve o erro `FileExists`. */
export type OnExists = "fail" | "overwrite" | "suffix";

export interface ExportTargets {
  json: boolean;
  cells_md: boolean;
  vba_md: boolean;
}

export type AppErrorCode =
  | "FileNotFound"
  | "FileLocked"
  | "UnsupportedFormat"
  | "CorruptedFile"
  | "NoVbaProject"
  | "WriteFailed"
  | "InvalidName"
  | "FileExists"
  | "Internal";

export interface AppError {
  code: AppErrorCode;
  message: string;
}
