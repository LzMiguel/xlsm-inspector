//! Camada fina: recebe do frontend, delega a `excel/`, `output/` e `store/` em thread
//! separada e traduz progresso para eventos Tauri.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Emitter, Manager};

use crate::error::AppError;
use crate::excel::{diagnostics, power_query, vba, vba::VbaOptions};
use crate::model::{
    Diagnostics, ExtractScope, ExtractionResult, PowerQuery, Progress, VbaComponent,
};
use crate::output::bundle::{BundleEstimate, add_section, bundle_units, estimate_units};
use crate::output::export::{self, ExportTargets, ItemFilter, OutputConfig};
use crate::output::naming::{FileNames, OnExists};
use crate::output::template::{self, OutputTemplate, TemplatePreset};
use crate::output::{markdown, vba_files};
use crate::store::cache::{self, OpenResult, RecentStatus};
use crate::store::recent::{self, RecentFile};
use crate::store::settings::{self, Settings};

const PROGRESS_EVENT: &str = "extraction://progress";

async fn blocking<T, F>(f: F) -> Result<T, AppError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, AppError> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
}

fn emitter(app: AppHandle) -> impl FnMut(Progress) {
    move |p| {
        let _ = app.emit(PROGRESS_EVENT, p); // progresso é best-effort
    }
}

fn config_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    app.path()
        .app_config_dir()
        .map_err(|e| AppError::Internal(e.to_string()))
}

/// Cache e dados do app ficam na pasta de dados (não na de configuração).
fn data_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    app.path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(e.to_string()))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

/// Abre a planilha: resumo + (se houver cache válido) a última extração.
/// `reload: true` ignora o cache e relê o arquivo. Atualiza os recentes.
#[tauri::command]
pub async fn open_workbook(
    app: AppHandle,
    path: String,
    reload: Option<bool>,
    vba_options: Option<VbaOptions>,
) -> Result<OpenResult, AppError> {
    let (cfg, data) = (config_dir(&app)?, data_dir(&app)?);
    blocking(move || {
        let use_cache = settings::load(&cfg).use_cache;
        let res = cache::open(
            &data,
            Path::new(&path),
            use_cache,
            reload.unwrap_or(false),
            vba_options.unwrap_or_default(),
        );
        // a lista de recentes é conveniência: falhar ao gravá-la não impede abrir
        match &res {
            Ok(r) => {
                let file = RecentFile {
                    path: path.clone(),
                    name: r.summary.file_name.clone(),
                    opened_at: now_ms(),
                };
                let _ = recent::add(&cfg, file);
            }
            Err(AppError::FileNotFound(_)) => {
                let _ = recent::remove(&cfg, &path);
            }
            Err(_) => {}
        }
        res
    })
    .await
}

/// Células + VBA de uma vez. `sheets: None` = todas as abas.
#[tauri::command]
pub async fn extract_workbook(
    app: AppHandle,
    path: String,
    sheets: Option<Vec<String>>,
    scope: Option<ExtractScope>,
    vba_options: Option<VbaOptions>,
) -> Result<ExtractionResult, AppError> {
    let (cfg, data) = (config_dir(&app)?, data_dir(&app)?);
    blocking(move || {
        let mut emit = emitter(app);
        cache::extract(
            &data,
            Path::new(&path),
            settings::load(&cfg).use_cache,
            sheets.as_deref(),
            scope.unwrap_or_default(),
            vba_options.unwrap_or_default(),
            &mut emit,
        )
    })
    .await
}

/// Problemas e peças da planilha (erros, nomes, abas ocultas, tabelas, externas). Só leitura.
#[tauri::command]
pub async fn diagnose_workbook(app: AppHandle, path: String) -> Result<Diagnostics, AppError> {
    let (cfg, data) = (config_dir(&app)?, data_dir(&app)?);
    blocking(move || {
        // extração completa em cache evita reler as células
        let cached = settings::load(&cfg)
            .use_cache
            .then(|| cache::cached_full_sheets(&data, Path::new(&path)))
            .flatten();
        diagnostics::diagnose_with(Path::new(&path), cached.as_deref())
    })
    .await
}

/// Consultas M e conexões da planilha. Sem Power Query → listas vazias. Só leitura.
#[tauri::command]
pub async fn extract_power_query(path: String) -> Result<PowerQuery, AppError> {
    blocking(move || power_query::extract_file(Path::new(&path))).await
}

#[tauri::command]
pub async fn extract_vba(
    path: String,
    options: Option<VbaOptions>,
) -> Result<Vec<VbaComponent>, AppError> {
    blocking(move || vba::extract_file(Path::new(&path), &options.unwrap_or_default())).await
}

/// Grava `.bas`/`.cls`/`.frm` (ANSI, como o VBE importa) em `out_dir`; `names` omitido = todos.
/// Lê o código bruto (com `Attribute`), ignorando as opções de limpeza da exibição.
#[tauri::command]
pub async fn export_vba_files(
    path: String,
    out_dir: String,
    names: Option<Vec<String>>,
) -> Result<Vec<String>, AppError> {
    blocking(move || {
        let raw = VbaOptions {
            remove_blank_lines: false,
            hide_attributes: false,
        };
        let comps: Vec<VbaComponent> = vba::extract_file(Path::new(&path), &raw)?
            .into_iter()
            .filter(|c| names.as_ref().is_none_or(|n| n.contains(&c.name)))
            .collect();
        vba_files::export_files(&comps, Path::new(&out_dir))
    })
    .await
}

/// Grava os arquivos (reaproveita o cache se ele tiver as mesmas abas, escopo e opções);
/// devolve os caminhos gerados.
// Cada parâmetro é uma chave do `invoke`; agrupar quebraria o contrato com o frontend.
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn export_outputs(
    app: AppHandle,
    path: String,
    out_dir: String,
    targets: Option<ExportTargets>,
    sheets: Option<Vec<String>>,
    vba_options: Option<VbaOptions>,
    names: Option<FileNames>,
    template: Option<OutputTemplate>,
    on_exists: Option<OnExists>,
    scope: Option<ExtractScope>,
    split_tokens: Option<usize>,
    items: Option<ItemFilter>,
    zip: Option<bool>,
) -> Result<Vec<String>, AppError> {
    let (conf, data) = (config_dir(&app)?, data_dir(&app)?);
    blocking(move || {
        let mut emit = emitter(app);
        let (scope, vba_options) = (scope.unwrap_or_default(), vba_options.unwrap_or_default());
        let cached = settings::load(&conf)
            .use_cache
            .then(|| {
                cache::cached_result(
                    &data,
                    Path::new(&path),
                    sheets.as_deref(),
                    scope,
                    vba_options,
                )
            })
            .flatten();
        let cfg = OutputConfig {
            names: names.unwrap_or_default(),
            template: template.unwrap_or_default(),
            on_exists: on_exists.unwrap_or_default(),
            scope,
            split_tokens,
            cached,
            items: items.unwrap_or_default(),
            zip: zip.unwrap_or_default(),
        };
        export::export_outputs(
            Path::new(&path),
            &PathBuf::from(out_dir),
            targets.unwrap_or_default(),
            sheets.as_deref(),
            &vba_options,
            &cfg,
            &mut emit,
        )
    })
    .await
}

/// Tokens (estimados) e nº de partes do arquivo único. Usa o cache; sem ele, extrai (e guarda).
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn estimate_bundle(
    app: AppHandle,
    path: String,
    sheets: Option<Vec<String>>,
    scope: Option<ExtractScope>,
    vba_options: Option<VbaOptions>,
    template: Option<OutputTemplate>,
    split_tokens: Option<usize>,
    items: Option<ItemFilter>,
    diagnostics: Option<bool>,
) -> Result<BundleEstimate, AppError> {
    let (conf, data) = (config_dir(&app)?, data_dir(&app)?);
    blocking(move || {
        let (scope, vba_options) = (scope.unwrap_or_default(), vba_options.unwrap_or_default());
        let use_cache = settings::load(&conf).use_cache;
        let cached = use_cache
            .then(|| {
                cache::cached_result(
                    &data,
                    Path::new(&path),
                    sheets.as_deref(),
                    scope,
                    vba_options,
                )
            })
            .flatten();
        let result = match cached {
            Some(r) => r,
            None => {
                // Não sobrescreve o cache da extração ativa nem emite progresso global em prévia.
                let mut noop = |_| {};
                cache::extract(
                    &data,
                    Path::new(&path),
                    false,
                    sheets.as_deref(),
                    scope,
                    vba_options,
                    &mut noop,
                )?
            }
        };
        let file_name = Path::new(&path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let items = items.unwrap_or_default();
        let template = template.unwrap_or_default();
        let mut units = bundle_units(
            &file_name,
            &result.sheets,
            &items.vba(&result.vba),
            &items.power_query(&result.power_query),
            &template,
            &template::today(),
        );
        if diagnostics.unwrap_or_default() {
            let d = diagnostics::diagnose(Path::new(&path))?;
            add_section(&mut units, &markdown::diagnostics_block(&d), &template);
        }
        Ok(estimate_units(&units, split_tokens))
    })
    .await
}

#[tauri::command]
pub fn builtin_templates() -> Vec<TemplatePreset> {
    template::builtin_presets()
}

/// Markdown de exemplo com o modelo informado (dados fictícios).
#[tauri::command]
pub fn preview_template(template: OutputTemplate) -> String {
    markdown::preview(&template, &template::today())
}

#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Result<Settings, AppError> {
    let dir = config_dir(&app)?;
    blocking(move || Ok(settings::load(&dir))).await
}

#[tauri::command]
pub async fn save_settings(app: AppHandle, settings: Settings) -> Result<(), AppError> {
    let dir = config_dir(&app)?;
    blocking(move || settings::save(&dir, &settings)).await
}

#[tauri::command]
pub async fn get_recent(app: AppHandle) -> Result<Vec<RecentFile>, AppError> {
    let dir = config_dir(&app)?;
    blocking(move || Ok(recent::load(&dir))).await
}

#[tauri::command]
pub async fn remove_recent(app: AppHandle, path: String) -> Result<Vec<RecentFile>, AppError> {
    let dir = config_dir(&app)?;
    blocking(move || recent::remove(&dir, &path)).await
}

#[tauri::command]
pub async fn clear_recent(app: AppHandle) -> Result<Vec<RecentFile>, AppError> {
    let dir = config_dir(&app)?;
    blocking(move || recent::clear(&dir)).await
}

/// Situação de cada recente (só `metadata` + JSON do cache; não abre as planilhas).
#[tauri::command]
pub async fn check_recent(app: AppHandle) -> Result<Vec<RecentStatus>, AppError> {
    let (cfg, data) = (config_dir(&app)?, data_dir(&app)?);
    blocking(move || {
        Ok(recent::load(&cfg)
            .into_iter()
            .map(|r| RecentStatus {
                status: cache::status(&data, Path::new(&r.path)),
                path: r.path,
            })
            .collect())
    })
    .await
}

#[tauri::command]
pub async fn clear_cache(app: AppHandle) -> Result<(), AppError> {
    let dir = data_dir(&app)?;
    blocking(move || cache::clear(&dir)).await
}
