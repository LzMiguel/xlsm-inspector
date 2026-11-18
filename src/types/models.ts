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
  /** Preenchido só quando o escopo cortou linhas desta aba. */
  sampled: SampleInfo | null;
}

export interface SampleInfo {
  total_rows: number;
  kept_rows: number;
}

/** Quais linhas extrair (espelho de `ExtractScope` em model.rs). */
export type ExtractScope =
  | { kind: "full" }
  | { kind: "head"; rows: number }
  | { kind: "sample"; rows: number };

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

export type WarningKind = "MissingCache" | "Sampled";

export interface Warning {
  kind: WarningKind;
  sheet: string;
  /** Quantidade de células afetadas. */
  count: number;
  message: string;
}

/** Célula com problema: `detail` é o erro (`#DIV/0!`) ou a fórmula com referência externa. */
export interface CellIssue {
  sheet: string;
  cell: string;
  detail: string;
}

export interface DefinedName {
  name: string;
  refers_to: string;
}

export interface HiddenSheet {
  name: string;
  very_hidden: boolean;
}

/** Tabela ou tabela dinâmica (`range` só nas tabelas). */
export interface SheetObject {
  sheet: string;
  name: string;
  range: string | null;
}

/** As listas de células vêm cortadas; `*_count` tem o total real. */
export interface Diagnostics {
  errors: CellIssue[];
  error_count: number;
  external_refs: CellIssue[];
  external_count: number;
  names: DefinedName[];
  hidden_sheets: HiddenSheet[];
  tables: SheetObject[];
  pivots: SheetObject[];
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
  /** Guarda no disco o resultado das últimas 5 planilhas. */
  use_cache: boolean;
}

/** Espelho de store/recent.rs. */
export interface RecentFile {
  path: string;
  name: string;
  /** Epoch em ms da última abertura. */
  opened_at: number;
}

/** Cached: há cache e o arquivo não mudou. Modified: há cache, mas o arquivo mudou. */
export type PathStatus = "Cached" | "Modified" | "NotCached" | "Missing" | "Locked";

export interface RecentStatus {
  path: string;
  status: PathStatus;
}

/** Última extração guardada no cache (espelho de store/cache.rs). */
export interface CachedExtraction {
  sheets: string[];
  scope: ExtractScope;
  result: ExtractionResult;
}

export interface OpenResult {
  summary: WorkbookSummary;
  cached: CachedExtraction | null;
}

/** Espelho de output/naming.rs: padrões sem extensão. `{arquivo}` e `{data}`. */
export interface FileNames {
  json: string;
  cells_md: string;
  vba_md: string;
  /** Arquivo único para IA; dividido vira `nome_parte_1.md`, `nome_parte_2.md`... */
  bundle: string;
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
  /** Markdown único para IA (resumo + células + VBA). */
  bundle: boolean;
}

/** Espelho de `BundleEstimate` (output/bundle.rs). `tokens` é uma estimativa (≈ caracteres / 4). */
export interface BundleEstimate {
  tokens: number;
  parts: number;
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
