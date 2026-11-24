//! Cache em disco do resultado da extração (até 5 planilhas, só local).
//! Um JSON por planilha em `<base>/cache/`. A entrada vale enquanto tamanho + data de
//! modificação do arquivo forem os mesmos. Nunca escreve na planilha (RNF07).
//! shortcut: tamanho + mtime pode errar em pastas sincronizadas (OneDrive); "Recarregar" é a válvula.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::write_atomic;
use crate::error::AppError;
use crate::excel::cells::extract_with_scope_from_book;
use crate::excel::power_query;
use crate::excel::vba::{self, VbaOptions};
use crate::excel::workbook::{self, inspect, io_err};
use crate::model::{
    ExtractScope, ExtractionResult, Progress, SheetInfo, Warning, WarningKind, WorkbookSummary,
};

pub const CACHE_VERSION: u32 = 4; // 4: CellValue lossless + hash FNV-1a estável (3: power_query, 2: procedures)
pub const MAX_ENTRIES: usize = 5;
/// Planilhas maiores que isso não entram no cache (o JSON ficaria enorme).
pub const MAX_FILE_BYTES: u64 = 50 * 1024 * 1024;
const DIR: &str = "cache";

/// Serialização interna com tags explícitas em `CellValue` para preservar `DateTime` e `Error`
/// no cache em disco (o JSON exportado continua untagged pelo `model.rs`).
mod wire_result {
    use indexmap::IndexMap;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use crate::model::{
        CellContent, CellValue, ExtractionResult, PowerQuery, SampleInfo, SheetCells, TableInfo,
        VbaComponent, Warning,
    };

    #[derive(Serialize)]
    enum ValRef<'a> {
        Empty,
        Bool(bool),
        Int(i64),
        Float(f64),
        String(&'a str),
        DateTime(&'a str),
        Error(&'a str),
    }

    impl<'a> From<&'a CellValue> for ValRef<'a> {
        fn from(v: &'a CellValue) -> Self {
            match v {
                CellValue::Empty => Self::Empty,
                CellValue::Bool(b) => Self::Bool(*b),
                CellValue::Int(i) => Self::Int(*i),
                CellValue::Float(f) if f.is_finite() => Self::Float(*f),
                CellValue::Float(_) => Self::Empty,
                CellValue::String(s) => Self::String(s),
                CellValue::DateTime(s) => Self::DateTime(s),
                CellValue::Error(s) => Self::Error(s),
            }
        }
    }

    #[derive(Serialize)]
    enum ContentRef<'a> {
        Plain(ValRef<'a>),
        Formula { formula: &'a str, valor: ValRef<'a> },
    }

    impl<'a> From<&'a CellContent> for ContentRef<'a> {
        fn from(c: &'a CellContent) -> Self {
            match c {
                CellContent::Plain(v) => Self::Plain(v.into()),
                CellContent::Formula { formula, valor } => Self::Formula {
                    formula,
                    valor: valor.into(),
                },
            }
        }
    }

    #[derive(Serialize)]
    struct TableRef<'a> {
        name: &'a str,
        range: &'a str,
        columns: &'a [String],
        rows: Vec<Vec<ContentRef<'a>>>,
        cells: &'a [String],
    }

    #[derive(Serialize)]
    struct SheetRef<'a> {
        name: &'a str,
        cells: IndexMap<&'a str, ContentRef<'a>>,
        tables: Vec<TableRef<'a>>,
        sampled: Option<SampleInfo>,
    }

    #[derive(Serialize)]
    struct ResultRef<'a> {
        sheets: Vec<SheetRef<'a>>,
        vba: &'a [VbaComponent],
        warnings: &'a [Warning],
        power_query: &'a PowerQuery,
    }

    #[derive(Deserialize)]
    enum ValOwned {
        Empty,
        Bool(bool),
        Int(i64),
        Float(f64),
        String(String),
        DateTime(String),
        Error(String),
    }

    impl From<ValOwned> for CellValue {
        fn from(v: ValOwned) -> Self {
            match v {
                ValOwned::Empty => Self::Empty,
                ValOwned::Bool(b) => Self::Bool(b),
                ValOwned::Int(i) => Self::Int(i),
                ValOwned::Float(f) => Self::Float(f),
                ValOwned::String(s) => Self::String(s),
                ValOwned::DateTime(s) => Self::DateTime(s),
                ValOwned::Error(s) => Self::Error(s),
            }
        }
    }

    #[derive(Deserialize)]
    enum ContentOwned {
        Plain(ValOwned),
        Formula { formula: String, valor: ValOwned },
    }

    impl From<ContentOwned> for CellContent {
        fn from(c: ContentOwned) -> Self {
            match c {
                ContentOwned::Plain(v) => Self::Plain(v.into()),
                ContentOwned::Formula { formula, valor } => Self::Formula {
                    formula,
                    valor: valor.into(),
                },
            }
        }
    }

    #[derive(Deserialize)]
    struct TableOwned {
        name: String,
        range: String,
        columns: Vec<String>,
        rows: Vec<Vec<ContentOwned>>,
        cells: Vec<String>,
    }

    #[derive(Deserialize)]
    struct SheetOwned {
        name: String,
        cells: IndexMap<String, ContentOwned>,
        tables: Vec<TableOwned>,
        sampled: Option<SampleInfo>,
    }

    #[derive(Deserialize)]
    struct ResultOwned {
        sheets: Vec<SheetOwned>,
        vba: Vec<VbaComponent>,
        warnings: Vec<Warning>,
        #[serde(default)]
        power_query: PowerQuery,
    }

    pub fn serialize<S: Serializer>(r: &ExtractionResult, s: S) -> Result<S::Ok, S::Error> {
        let wire = ResultRef {
            sheets: r
                .sheets
                .iter()
                .map(|sh| SheetRef {
                    name: &sh.name,
                    cells: sh
                        .cells
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.into()))
                        .collect(),
                    tables: sh
                        .tables
                        .iter()
                        .map(|t| TableRef {
                            name: &t.name,
                            range: &t.range,
                            columns: &t.columns,
                            rows: t
                                .rows
                                .iter()
                                .map(|row| row.iter().map(ContentRef::from).collect())
                                .collect(),
                            cells: &t.cells,
                        })
                        .collect(),
                    sampled: sh.sampled,
                })
                .collect(),
            vba: &r.vba,
            warnings: &r.warnings,
            power_query: &r.power_query,
        };
        wire.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<ExtractionResult, D::Error> {
        let w = ResultOwned::deserialize(d)?;
        Ok(ExtractionResult {
            sheets: w
                .sheets
                .into_iter()
                .map(|sh| SheetCells {
                    name: sh.name,
                    cells: sh.cells.into_iter().map(|(k, v)| (k, v.into())).collect(),
                    tables: sh
                        .tables
                        .into_iter()
                        .map(|t| TableInfo {
                            name: t.name,
                            range: t.range,
                            columns: t.columns,
                            rows: t
                                .rows
                                .into_iter()
                                .map(|row| row.into_iter().map(CellContent::from).collect())
                                .collect(),
                            cells: t.cells,
                        })
                        .collect(),
                    sampled: sh.sampled,
                })
                .collect(),
            vba: w.vba,
            warnings: w.warnings,
            power_query: w.power_query,
        })
    }
}

fn fnv1a_64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0100_0000_01b3;
    let mut h = OFFSET;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(PRIME);
    }
    h
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fingerprint {
    pub size: u64,
    pub mtime_ms: u64,
}

/// Só `metadata`: não abre nem hidrata o arquivo.
pub fn fingerprint(path: &Path) -> Result<Fingerprint, AppError> {
    let m = fs::metadata(path).map_err(io_err)?;
    let mtime_ms = m
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_millis() as u64);
    Ok(Fingerprint {
        size: m.len(),
        mtime_ms,
    })
}

/// O que foi extraído: abas (ordenadas), escopo e opções de VBA.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Options {
    pub sheets: Vec<String>,
    pub scope: ExtractScope,
    pub vba: VbaOptions,
}

impl Options {
    /// `sheets: None` = todas (`all`). A ordem pedida não importa.
    pub fn new(
        sheets: Option<&[String]>,
        all: &[SheetInfo],
        scope: ExtractScope,
        vba: VbaOptions,
    ) -> Self {
        let mut sheets: Vec<String> = match sheets {
            Some(s) => s.to_vec(),
            None => all.iter().map(|s| s.name.clone()).collect(),
        };
        sheets.sort();
        Self { sheets, scope, vba }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extraction {
    pub options: Options,
    #[serde(with = "wire_result")]
    pub result: ExtractionResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub cache_version: u32,
    pub path: String,
    pub fingerprint: Fingerprint,
    pub summary: WorkbookSummary,
    pub extraction: Option<Extraction>,
}

#[derive(Serialize)]
struct ExtractionRef<'a> {
    options: &'a Options,
    #[serde(serialize_with = "wire_result::serialize")]
    result: &'a ExtractionResult,
}

#[derive(Serialize)]
struct EntryRef<'a> {
    cache_version: u32,
    path: &'a str,
    fingerprint: Fingerprint,
    summary: &'a WorkbookSummary,
    extraction: Option<ExtractionRef<'a>>,
}

impl Entry {
    /// O resultado guardado, se foi gerado com exatamente estas opções.
    pub fn extraction_for(&self, opts: &Options) -> Option<&ExtractionResult> {
        self.extraction
            .as_ref()
            .filter(|x| x.options == *opts)
            .map(|x| &x.result)
    }
}

fn dir(base: &Path) -> PathBuf {
    base.join(DIR)
}

fn file_for(base: &Path, path: &str) -> PathBuf {
    let lower = path.to_lowercase(); // o Windows não diferencia maiúsculas
    dir(base).join(format!("{:016x}.json", fnv1a_64(lower.as_bytes())))
}

/// Entrada gravada para este caminho (versão e caminho conferem), válida ou não.
/// Corrompida = ausente.
fn read_entry(base: &Path, path: &str) -> Option<(PathBuf, Entry)> {
    let file = file_for(base, path);
    let e: Entry = serde_json::from_str(&fs::read_to_string(&file).ok()?).ok()?;
    (e.cache_version == CACHE_VERSION && e.path.to_lowercase() == path.to_lowercase())
        .then_some((file, e))
}

/// Marca como usada agora (o mais antigo sai primeiro).
fn touch(file: &Path) {
    if let Ok(f) = fs::OpenOptions::new().write(true).open(file) {
        let _ = f.set_modified(SystemTime::now());
    }
}

fn lookup_with_fp(base: &Path, path: &Path, fp: Fingerprint) -> Option<Entry> {
    let (file, e) = read_entry(base, &path.to_string_lossy())?;
    (fp == e.fingerprint).then(|| {
        touch(&file);
        e
    })
}

/// Entrada válida: o arquivo continua com o mesmo tamanho e a mesma data de modificação.
pub fn lookup(base: &Path, path: &Path) -> Option<Entry> {
    let fp = fingerprint(path).ok()?;
    lookup_with_fp(base, path, fp)
}

fn prune(base: &Path) {
    let Ok(rd) = fs::read_dir(dir(base)) else {
        return;
    };
    let mut files: Vec<(SystemTime, PathBuf)> = rd
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
        .collect();
    files.sort_by_key(|f| std::cmp::Reverse(f.0));
    for (_, p) in files.into_iter().skip(MAX_ENTRIES) {
        let _ = fs::remove_file(p);
    }
}

fn write_entry_ref(base: &Path, e: &EntryRef<'_>) -> Result<(), AppError> {
    let json = serde_json::to_string(e).map_err(|e| AppError::WriteFailed(e.to_string()))?;
    let name = file_for(base, e.path);
    let name = name
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    write_atomic(&dir(base), &name, &json)
        .map_err(|e| AppError::WriteFailed(format!("cache: {e}")))?;
    prune(base);
    Ok(())
}

/// Guarda o resumo (sem extração). `fp` deve ser lido ANTES de abrir o arquivo.
pub fn store_summary(
    base: &Path,
    path: &Path,
    fp: Fingerprint,
    summary: &WorkbookSummary,
) -> Result<(), AppError> {
    if fp.size > MAX_FILE_BYTES {
        return Ok(());
    }
    let path_str = path.to_string_lossy();
    write_entry_ref(
        base,
        &EntryRef {
            cache_version: CACHE_VERSION,
            path: &path_str,
            fingerprint: fp,
            summary,
            extraction: None,
        },
    )
}

/// Acrescenta a extração à entrada do arquivo, se ela ainda for a do mesmo `fp`.
pub fn store_extraction(
    base: &Path,
    path: &Path,
    fp: Fingerprint,
    opts: impl FnOnce(&[SheetInfo]) -> Options,
    result: &ExtractionResult,
) -> Result<(), AppError> {
    // Re-verifica no disco para fechar janela TOCTOU caso o arquivo tenha mudado durante a leitura
    if fingerprint(path).ok() != Some(fp) {
        return Ok(());
    }
    let Some((_, e)) = read_entry(base, &path.to_string_lossy()) else {
        return Ok(());
    };
    if e.fingerprint != fp {
        return Ok(());
    }
    let options = opts(&e.summary.sheets);
    write_entry_ref(
        base,
        &EntryRef {
            cache_version: e.cache_version,
            path: &e.path,
            fingerprint: e.fingerprint,
            summary: &e.summary,
            extraction: Some(ExtractionRef {
                options: &options,
                result,
            }),
        },
    )
}

pub fn clear(base: &Path) -> Result<(), AppError> {
    match fs::remove_dir_all(dir(base)) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            Err(AppError::WriteFailed(format!("cache: {e}")))
        }
        _ => Ok(()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PathStatus {
    /// Há cache e o arquivo não mudou.
    Cached,
    /// Há cache, mas o arquivo mudou desde então.
    Modified,
    NotCached,
    Missing,
    Locked,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecentStatus {
    pub path: String,
    pub status: PathStatus,
}

/// Só `metadata` e o JSON do cache; não abre a planilha.
pub fn status(base: &Path, path: &Path) -> PathStatus {
    match fingerprint(path) {
        Err(AppError::FileNotFound(_)) => PathStatus::Missing,
        Err(_) => PathStatus::Locked,
        Ok(fp) => match read_entry(base, &path.to_string_lossy()) {
            Some((_, e)) if e.fingerprint == fp => PathStatus::Cached,
            Some(_) => PathStatus::Modified,
            None => PathStatus::NotCached,
        },
    }
}

/// Resultado em cache gerado com estas mesmas opções (para a exportação não reler o arquivo).
pub fn cached_result(
    base: &Path,
    path: &Path,
    sheets: Option<&[String]>,
    scope: ExtractScope,
    vba: VbaOptions,
) -> Option<ExtractionResult> {
    let e = lookup(base, path)?;
    e.extraction_for(&Options::new(sheets, &e.summary.sheets, scope, vba))
        .cloned()
}

/// Extração guardada que o frontend restaura ao reabrir.
#[derive(Debug, Clone, Serialize)]
pub struct CachedExtraction {
    pub sheets: Vec<String>,
    pub scope: ExtractScope,
    pub result: ExtractionResult,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenResult {
    pub summary: WorkbookSummary,
    pub cached: Option<CachedExtraction>,
}

/// Abre a planilha. Com cache válido (e `reload` falso) responde sem reler o arquivo;
/// a extração só volta se foi feita com as mesmas opções de VBA de agora.
pub fn open(
    base: &Path,
    path: &Path,
    use_cache: bool,
    reload: bool,
    vba: VbaOptions,
) -> Result<OpenResult, AppError> {
    let fp = fingerprint(path)?;
    if use_cache
        && !reload
        && let Some(e) = lookup_with_fp(base, path, fp)
    {
        let cached = e
            .extraction
            .filter(|x| x.options.vba == vba)
            .map(|x| CachedExtraction {
                sheets: x.options.sheets,
                scope: x.options.scope,
                result: x.result,
            });
        return Ok(OpenResult {
            summary: e.summary,
            cached,
        });
    }
    let summary = inspect(path)?;
    if use_cache {
        let _ = store_summary(base, path, fp, &summary); // falha de cache não impede abrir
    }
    Ok(OpenResult {
        summary,
        cached: None,
    })
}

/// Células + VBA em um passo; guarda no cache se ligado.
pub fn extract(
    base: &Path,
    path: &Path,
    use_cache: bool,
    sheets: Option<&[String]>,
    scope: ExtractScope,
    vba_opts: VbaOptions,
    on_progress: &mut dyn FnMut(Progress),
) -> Result<ExtractionResult, AppError> {
    let fp = fingerprint(path)?;
    let mut book = workbook::open(path)?;
    let (cells, mut warnings) =
        extract_with_scope_from_book(&mut book, sheets, scope, on_progress)?;
    on_progress(Progress::new("vba", 0, 1, "VBA"));
    let vba = match vba::extract(&mut book, &vba_opts) {
        Ok(v) => v,
        Err(AppError::NoVbaProject) => Vec::new(),
        Err(e) => return Err(e),
    };
    on_progress(Progress::new("vba", 1, 1, "VBA"));
    // Power Query ilegível não derruba as células, mas o usuário é avisado.
    let power_query = power_query::extract_file(path).unwrap_or_else(|e| {
        warnings.push(Warning {
            kind: WarningKind::PowerQueryUnreadable,
            sheet: String::new(),
            count: 0,
            message: format!("Power Query não pôde ser lido: {e}"),
        });
        Default::default()
    });
    let result = ExtractionResult {
        sheets: cells,
        vba,
        warnings,
        power_query,
    };
    if use_cache {
        let _ = store_extraction(
            base,
            path,
            fp,
            |all| Options::new(sheets, all, scope, vba_opts),
            &result,
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    struct Tmp(PathBuf);
    impl Tmp {
        fn new(name: &str) -> Self {
            let d = std::env::temp_dir().join(format!(
                "xlsm-inspector-{}-cache-{name}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&d);
            fs::create_dir_all(&d).unwrap();
            Tmp(d)
        }
        /// Planilha "falsa": o cache só olha tamanho e data.
        fn file(&self, name: &str, content: &str) -> PathBuf {
            let p = self.0.join(name);
            fs::write(&p, content).unwrap();
            p
        }
    }
    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn summary(path: &Path) -> WorkbookSummary {
        WorkbookSummary {
            path: path.to_string_lossy().into_owned(),
            file_name: "a.xlsx".into(),
            size_bytes: 1,
            sheets: vec![
                SheetInfo {
                    name: "B".into(),
                    cell_count: 1,
                    formula_count: 0,
                },
                SheetInfo {
                    name: "A".into(),
                    cell_count: 1,
                    formula_count: 0,
                },
            ],
            has_vba: false,
        }
    }

    fn result() -> ExtractionResult {
        ExtractionResult {
            sheets: vec![],
            vba: vec![],
            warnings: vec![],
            power_query: Default::default(),
        }
    }

    fn opts(vba: VbaOptions) -> impl FnOnce(&[SheetInfo]) -> Options {
        move |all| Options::new(None, all, ExtractScope::Full, vba)
    }

    /// Guarda resumo + extração do arquivo, como o app faz.
    fn put(t: &Tmp, p: &Path) {
        let fp = fingerprint(p).unwrap();
        store_summary(&t.0, p, fp, &summary(p)).unwrap();
        store_extraction(&t.0, p, fp, opts(VbaOptions::default()), &result()).unwrap();
    }

    #[test]
    fn opcoes_ignoram_a_ordem_das_abas() {
        let all = summary(Path::new("x")).sheets;
        let a = Options::new(
            Some(&["B".into(), "A".into()]),
            &all,
            ExtractScope::Full,
            VbaOptions::default(),
        );
        let b = Options::new(None, &all, ExtractScope::Full, VbaOptions::default());
        assert_eq!(a, b);
        assert_eq!(a.sheets, ["A", "B"]);
    }

    #[test]
    fn acerto_com_as_mesmas_opcoes() {
        let t = Tmp::new("hit");
        let p = t.file("a.xlsx", "dados");
        assert!(lookup(&t.0, &p).is_none());
        put(&t, &p);

        let e = lookup(&t.0, &p).expect("deveria acertar");
        assert_eq!(status(&t.0, &p), PathStatus::Cached);
        let o = Options::new(
            None,
            &e.summary.sheets,
            ExtractScope::Full,
            VbaOptions::default(),
        );
        assert!(e.extraction_for(&o).is_some());
        assert!(cached_result(&t.0, &p, None, ExtractScope::Full, VbaOptions::default()).is_some());
    }

    #[test]
    fn erra_quando_o_arquivo_muda() {
        let t = Tmp::new("mtime");
        let p = t.file("a.xlsx", "dados");
        put(&t, &p);

        // mesmo tamanho, outra data de modificação
        let f = fs::OpenOptions::new().write(true).open(&p).unwrap();
        f.set_modified(SystemTime::now() + Duration::from_secs(60))
            .unwrap();
        drop(f);
        assert!(lookup(&t.0, &p).is_none());
        assert_eq!(status(&t.0, &p), PathStatus::Modified);

        // outro tamanho
        put(&t, &p);
        fs::write(&p, "dados maiores").unwrap();
        assert!(lookup(&t.0, &p).is_none());
    }

    #[test]
    fn erra_quando_as_opcoes_mudam() {
        let t = Tmp::new("opts");
        let p = t.file("a.xlsx", "dados");
        put(&t, &p);
        let outra = VbaOptions {
            remove_blank_lines: false,
            hide_attributes: true,
        };
        assert!(cached_result(&t.0, &p, None, ExtractScope::Full, outra).is_none());
        assert!(
            cached_result(
                &t.0,
                &p,
                None,
                ExtractScope::Head { rows: 5 },
                VbaOptions::default()
            )
            .is_none()
        );
        assert!(
            cached_result(
                &t.0,
                &p,
                Some(&["A".into()]),
                ExtractScope::Full,
                VbaOptions::default()
            )
            .is_none()
        );
        // o resumo continua valendo
        assert!(lookup(&t.0, &p).is_some());
    }

    #[test]
    fn guarda_so_os_5_mais_recentes() {
        let t = Tmp::new("lru");
        let paths: Vec<PathBuf> = (0..6).map(|i| t.file(&format!("p{i}.xlsx"), "x")).collect();
        for p in &paths {
            let fp = fingerprint(p).unwrap();
            store_summary(&t.0, p, fp, &summary(p)).unwrap();
            std::thread::sleep(Duration::from_millis(30));
        }
        assert!(
            lookup(&t.0, &paths[0]).is_none(),
            "o mais antigo deveria ter saído"
        );
        assert!(paths[1..].iter().all(|p| lookup(&t.0, p).is_some()));
        assert_eq!(fs::read_dir(dir(&t.0)).unwrap().count(), MAX_ENTRIES);
    }

    #[test]
    fn usar_renova_a_posicao() {
        let t = Tmp::new("lru-uso");
        let paths: Vec<PathBuf> = (0..5).map(|i| t.file(&format!("p{i}.xlsx"), "x")).collect();
        for p in &paths {
            store_summary(&t.0, p, fingerprint(p).unwrap(), &summary(p)).unwrap();
            std::thread::sleep(Duration::from_millis(30));
        }
        assert!(lookup(&t.0, &paths[0]).is_some()); // renova o mais antigo
        std::thread::sleep(Duration::from_millis(30));
        let novo = t.file("novo.xlsx", "x");
        store_summary(&t.0, &novo, fingerprint(&novo).unwrap(), &summary(&novo)).unwrap();
        assert!(
            lookup(&t.0, &paths[0]).is_some(),
            "p0 foi usado por último, não sai"
        );
        assert!(lookup(&t.0, &paths[1]).is_none(), "p1 virou o mais antigo");
    }

    #[test]
    fn cache_corrompido_e_ignorado() {
        let t = Tmp::new("ruim");
        let p = t.file("a.xlsx", "dados");
        put(&t, &p);
        fs::write(file_for(&t.0, &p.to_string_lossy()), "{ lixo").unwrap();
        assert!(lookup(&t.0, &p).is_none());
        assert_eq!(status(&t.0, &p), PathStatus::NotCached);
        put(&t, &p); // e dá para regravar por cima
        assert!(lookup(&t.0, &p).is_some());
    }

    #[test]
    fn versao_antiga_e_ignorada() {
        let t = Tmp::new("versao");
        let p = t.file("a.xlsx", "dados");
        put(&t, &p);
        let (file, mut e) = read_entry(&t.0, &p.to_string_lossy()).unwrap();
        e.cache_version = CACHE_VERSION + 1;
        fs::write(file, serde_json::to_string(&e).unwrap()).unwrap();
        assert!(lookup(&t.0, &p).is_none());
    }

    #[test]
    fn extracao_de_arquivo_alterado_no_meio_nao_e_guardada() {
        let t = Tmp::new("corrida");
        let p = t.file("a.xlsx", "dados");
        let fp = fingerprint(&p).unwrap();
        store_summary(&t.0, &p, fp, &summary(&p)).unwrap();
        fs::write(&p, "outro conteúdo").unwrap(); // mudou depois de ler `fp`
        let velho = Fingerprint {
            size: fp.size + 99,
            ..fp
        };
        store_extraction(&t.0, &p, velho, opts(VbaOptions::default()), &result()).unwrap();
        assert!(
            read_entry(&t.0, &p.to_string_lossy())
                .unwrap()
                .1
                .extraction
                .is_none()
        );
    }

    #[test]
    fn status_de_arquivo_ausente_e_limpar() {
        let t = Tmp::new("status");
        assert_eq!(
            status(&t.0, &t.0.join("nao_existe.xlsx")),
            PathStatus::Missing
        );
        let p = t.file("a.xlsx", "dados");
        assert_eq!(status(&t.0, &p), PathStatus::NotCached);
        put(&t, &p);
        clear(&t.0).unwrap();
        assert!(lookup(&t.0, &p).is_none());
        clear(&t.0).unwrap(); // sem pasta também é ok
    }

    #[test]
    fn arquivo_grande_nao_entra() {
        let t = Tmp::new("grande");
        let p = t.file("a.xlsx", "x");
        let fp = Fingerprint {
            size: MAX_FILE_BYTES + 1,
            mtime_ms: 0,
        };
        store_summary(&t.0, &p, fp, &summary(&p)).unwrap();
        assert!(!dir(&t.0).exists());
    }

    #[test]
    fn preserva_datetime_e_error_no_cache() {
        use crate::model::{CellContent, CellValue, SheetCells};
        use indexmap::IndexMap;

        let t = Tmp::new("lossless");
        let p = t.file("a.xlsx", "dados");
        let fp = fingerprint(&p).unwrap();
        store_summary(&t.0, &p, fp, &summary(&p)).unwrap();

        let mut cells = IndexMap::new();
        cells.insert(
            "A1".into(),
            CellContent::Plain(CellValue::DateTime("2024-01-15".into())),
        );
        cells.insert(
            "A2".into(),
            CellContent::Formula {
                formula: "=1/0".into(),
                valor: CellValue::Error("#DIV/0!".into()),
            },
        );
        let res = ExtractionResult {
            sheets: vec![SheetCells {
                name: "A".into(),
                cells,
                tables: vec![],
                sampled: None,
            }],
            vba: vec![],
            warnings: vec![],
            power_query: Default::default(),
        };
        store_extraction(&t.0, &p, fp, opts(VbaOptions::default()), &res).unwrap();

        let loaded = cached_result(&t.0, &p, None, ExtractScope::Full, VbaOptions::default())
            .expect("deveria recuperar do cache");
        assert_eq!(
            loaded.sheets[0].cells["A1"],
            CellContent::Plain(CellValue::DateTime("2024-01-15".into()))
        );
        assert_eq!(
            loaded.sheets[0].cells["A2"],
            CellContent::Formula {
                formula: "=1/0".into(),
                valor: CellValue::Error("#DIV/0!".into()),
            }
        );
    }
}
