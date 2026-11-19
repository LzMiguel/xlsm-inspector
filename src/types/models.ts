// Espelho EXATO de src-tauri/src/model.rs e error.rs. Alterar os dois juntos.
// Campos em snake_case, como serializados pelo Rust.

export interface SheetInfo {
  name: string
  cell_count: number
  formula_count: number
}

export interface WorkbookSummary {
  path: string
  file_name: string
  size_bytes: number
  sheets: SheetInfo[]
  has_vba: boolean
}

export type CellValue = null | boolean | number | string

export interface FormulaContent {
  formula: string
  valor: CellValue
}

export type CellContent = CellValue | FormulaContent

export interface TableInfo {
  name: string
  range: string
  columns: string[]
  rows: CellContent[][]
  cells: string[]
}

export interface SheetCells {
  name: string
  cells: Record<string, CellContent>
  tables: TableInfo[]
  sampled: SampleInfo | null
}

export interface SampleInfo {
  total_rows: number
  kept_rows: number
}

export type ExtractScope =
  | { kind: 'full' }
  | { kind: 'head'; rows: number }
  | { kind: 'sample'; rows: number }

export type VbaComponentKind =
  | 'Module'
  | 'Class'
  | 'Document'
  | 'Workbook'
  | 'UserForm'
  | 'Unknown'

export interface VbaComponent {
  name: string
  kind: VbaComponentKind
  code: string
  line_count: number
  procedures: VbaProcedure[]
}

export interface VbaProcedure {
  name: string
  /** "Sub" | "Function" | "Property Get" | "Property Let" | "Property Set". */
  kind: string
  visibility: string
  start_line: number
  end_line: number
}

export type WarningKind = 'MissingCache' | 'Sampled'

export interface Warning {
  kind: WarningKind
  sheet: string
  count: number
  message: string
}

export interface CellIssue {
  sheet: string
  cell: string
  detail: string
}

export interface DefinedName {
  name: string
  refers_to: string
}

export interface HiddenSheet {
  name: string
  very_hidden: boolean
}

export interface SheetObject {
  sheet: string
  name: string
  range: string | null
}

export interface Diagnostics {
  errors: CellIssue[]
  error_count: number
  external_refs: CellIssue[]
  external_count: number
  names: DefinedName[]
  hidden_sheets: HiddenSheet[]
  tables: SheetObject[]
  pivots: SheetObject[]
}

export interface PowerQueryItem {
  name: string
  code: string
}

export interface ConnectionInfo {
  name: string
  description: string
  connection: string
}

export interface PowerQuery {
  queries: PowerQueryItem[]
  connections: ConnectionInfo[]
}

export interface ExtractionResult {
  sheets: SheetCells[]
  vba: VbaComponent[]
  warnings: Warning[]
  power_query: PowerQuery
}

export interface Progress {
  stage: 'cells' | 'vba'
  current: number
  total: number
  label: string
}

export interface VbaOptions {
  remove_blank_lines: boolean
  hide_attributes: boolean
}

export interface Settings {
  version: number
  default_out_dir: string | null
  vba_options: VbaOptions
  file_names: FileNames
  template: OutputTemplate
  presets: TemplatePreset[]
  use_cache: boolean
}

export interface RecentFile {
  path: string
  name: string
  opened_at: number
}

export type PathStatus =
  | 'Cached'
  | 'Modified'
  | 'NotCached'
  | 'Missing'
  | 'Locked'

export interface RecentStatus {
  path: string
  status: PathStatus
}

export interface CachedExtraction {
  sheets: string[]
  scope: ExtractScope
  result: ExtractionResult
}

export interface OpenResult {
  summary: WorkbookSummary
  cached: CachedExtraction | null
}

export interface FileNames {
  json: string
  cells_md: string
  vba_md: string
  power_query: string
  bundle: string
}

export interface OutputTemplate {
  title: string
  intro: string
  sheet_heading: string
  cell_header: string
  content_header: string
  formula_label: string
  value_label: string
  vba_heading: string
  footer: string
}

export interface TemplatePreset {
  name: string
  template: OutputTemplate
}

export type OnExists = 'fail' | 'overwrite' | 'suffix'

export interface ExportTargets {
  json: boolean
  cells_md: boolean
  vba_md: boolean
  power_query: boolean
  bundle: boolean
}

export interface BundleEstimate {
  tokens: number
  parts: number
}

export type AppErrorCode =
  | 'FileNotFound'
  | 'FileLocked'
  | 'UnsupportedFormat'
  | 'CorruptedFile'
  | 'NoVbaProject'
  | 'WriteFailed'
  | 'InvalidName'
  | 'FileExists'
  | 'Internal'

export interface AppError {
  code: AppErrorCode
  message: string
}
