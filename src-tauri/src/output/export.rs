use std::fs;
use std::path::Path;

use serde::Deserialize;

use super::{json::cells_to_json, markdown};
use crate::error::AppError;
use crate::excel::{cells::extract_cells, vba, vba::VbaOptions};
use crate::model::Progress;

pub const JSON_FILE: &str = "mapeamento_planilha.json";
pub const CELLS_MD_FILE: &str = "mapeamento_planilha.md";
pub const VBA_MD_FILE: &str = "vba_code.md";

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(default)]
pub struct ExportTargets {
    pub json: bool,
    pub cells_md: bool,
    pub vba_md: bool,
}

impl Default for ExportTargets {
    fn default() -> Self {
        Self { json: true, cells_md: true, vba_md: true }
    }
}

fn write(dir: &Path, name: &str, content: &str, written: &mut Vec<String>) -> Result<(), AppError> {
    let p = dir.join(name);
    fs::write(&p, content).map_err(|e| AppError::WriteFailed(format!("{}: {e}", p.display())))?;
    written.push(p.to_string_lossy().into_owned());
    Ok(())
}

/// Re-extrai do arquivo e grava os alvos escolhidos em `out_dir`.
/// Sem projeto VBA, `vba_code.md` é simplesmente omitido. Devolve os caminhos gravados.
pub fn export_outputs(
    path: &Path,
    out_dir: &Path,
    targets: ExportTargets,
    sheets: Option<&[String]>,
    vba_opts: &VbaOptions,
    on_progress: &mut dyn FnMut(Progress),
) -> Result<Vec<String>, AppError> {
    if !out_dir.is_dir() {
        return Err(AppError::WriteFailed(format!(
            "pasta de saída inválida: {}",
            out_dir.display()
        )));
    }
    let mut written = Vec::new();

    if targets.json || targets.cells_md {
        let (cells, _) = extract_cells(path, sheets, on_progress)?;
        if targets.json {
            write(out_dir, JSON_FILE, &cells_to_json(&cells)?, &mut written)?;
        }
        if targets.cells_md {
            let file_name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
            let md = markdown::cells_to_markdown(&file_name, &cells);
            write(out_dir, CELLS_MD_FILE, &md, &mut written)?;
        }
    }

    if targets.vba_md {
        on_progress(Progress::new("vba", 0, 1, "VBA"));
        match vba::extract_file(path, vba_opts) {
            Ok(components) => {
                write(out_dir, VBA_MD_FILE, &markdown::vba_to_markdown(&components), &mut written)?
            }
            Err(AppError::NoVbaProject) => {}
            Err(e) => return Err(e),
        }
        on_progress(Progress::new("vba", 1, 1, "VBA"));
    }

    Ok(written)
}
