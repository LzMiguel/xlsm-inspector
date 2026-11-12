//! Camada fina: recebe do frontend, delega a `excel/` e `output/` em thread
//! separada e traduz progresso para eventos Tauri.

use std::path::{Path, PathBuf};

use tauri::{AppHandle, Emitter};

use crate::error::AppError;
use crate::excel::{cells, vba, vba::VbaOptions, workbook};
use crate::model::{CellsExtraction, Progress, VbaComponent, WorkbookSummary};
use crate::output::export::{self, ExportTargets};

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

#[tauri::command]
pub async fn inspect_workbook(path: String) -> Result<WorkbookSummary, AppError> {
    blocking(move || workbook::inspect(Path::new(&path))).await
}

/// `sheets: None` = todas as abas.
#[tauri::command]
pub async fn extract_cells(
    app: AppHandle,
    path: String,
    sheets: Option<Vec<String>>,
) -> Result<CellsExtraction, AppError> {
    blocking(move || {
        let mut emit = emitter(app);
        let (sheets, warnings) = cells::extract_cells(Path::new(&path), sheets.as_deref(), &mut emit)?;
        Ok(CellsExtraction { sheets, warnings })
    })
    .await
}

#[tauri::command]
pub async fn extract_vba(
    path: String,
    options: Option<VbaOptions>,
) -> Result<Vec<VbaComponent>, AppError> {
    blocking(move || vba::extract_file(Path::new(&path), &options.unwrap_or_default())).await
}

/// Re-extrai no Rust e grava os arquivos; devolve os caminhos gerados.
#[tauri::command]
pub async fn export_outputs(
    app: AppHandle,
    path: String,
    out_dir: String,
    targets: Option<ExportTargets>,
    sheets: Option<Vec<String>>,
    vba_options: Option<VbaOptions>,
) -> Result<Vec<String>, AppError> {
    blocking(move || {
        let mut emit = emitter(app);
        export::export_outputs(
            Path::new(&path),
            &PathBuf::from(out_dir),
            targets.unwrap_or_default(),
            sheets.as_deref(),
            &vba_options.unwrap_or_default(),
            &mut emit,
        )
    })
    .await
}
