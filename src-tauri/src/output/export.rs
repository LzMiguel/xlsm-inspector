use std::fs;
use std::path::Path;

use serde::Deserialize;

use super::naming::{FileNames, OnExists, free_name, resolve};
use super::template::{OutputTemplate, today};
use super::{json::cells_to_json, markdown};
use crate::error::AppError;
use crate::excel::{cells::extract_cells, vba, vba::VbaOptions};
use crate::model::Progress;

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

/// Nomes, modelo do Markdown e política de colisão. O padrão é o comportamento da v1.0.
#[derive(Debug, Clone, Default)]
pub struct OutputConfig {
    pub names: FileNames,
    pub template: OutputTemplate,
    pub on_exists: OnExists,
}

fn write(dir: &Path, name: &str, content: &str, written: &mut Vec<String>) -> Result<(), AppError> {
    let p = dir.join(name);
    fs::write(&p, content).map_err(|e| AppError::WriteFailed(format!("{}: {e}", p.display())))?;
    written.push(p.to_string_lossy().into_owned());
    Ok(())
}

/// Nomes finais dos alvos escolhidos, validados e sem colisão (entre si e com a pasta).
/// Falha antes de qualquer extração.
fn plan(
    path: &Path,
    out_dir: &Path,
    targets: ExportTargets,
    cfg: &OutputConfig,
    date: &str,
) -> Result<[Option<String>; 3], AppError> {
    let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let vars = [("arquivo", stem.as_str()), ("data", date)];
    let name = |on: bool, pattern: &str, ext: &str| {
        on.then(|| resolve(pattern, ext, &vars)).transpose()
    };
    let mut names = [
        name(targets.json, &cfg.names.json, "json")?,
        name(targets.cells_md, &cfg.names.cells_md, "md")?,
        name(targets.vba_md, &cfg.names.vba_md, "md")?,
    ];

    // o Windows não diferencia maiúsculas: "A.md" e "a.md" são o mesmo arquivo
    let lower: Vec<String> = names.iter().flatten().map(|n| n.to_lowercase()).collect();
    if let Some(dup) = lower.iter().enumerate().find(|(i, n)| lower[..*i].contains(n)) {
        return Err(AppError::InvalidName(format!("dois arquivos com o mesmo nome ({})", dup.1)));
    }

    let existing: Vec<&str> = names
        .iter()
        .flatten()
        .filter(|n| out_dir.join(n).exists())
        .map(String::as_str)
        .collect();
    match cfg.on_exists {
        OnExists::Fail if !existing.is_empty() => {
            return Err(AppError::FileExists(existing.join(", ")));
        }
        OnExists::Suffix => {
            for n in names.iter_mut().flatten() {
                *n = free_name(out_dir, n);
            }
        }
        _ => {}
    }
    Ok(names)
}

/// Re-extrai do arquivo e grava os alvos escolhidos em `out_dir`.
/// Sem projeto VBA, o Markdown do VBA é simplesmente omitido. Devolve os caminhos gravados.
pub fn export_outputs(
    path: &Path,
    out_dir: &Path,
    targets: ExportTargets,
    sheets: Option<&[String]>,
    vba_opts: &VbaOptions,
    cfg: &OutputConfig,
    on_progress: &mut dyn FnMut(Progress),
) -> Result<Vec<String>, AppError> {
    if !out_dir.is_dir() {
        return Err(AppError::WriteFailed(format!(
            "pasta de saída inválida: {}",
            out_dir.display()
        )));
    }
    let date = today();
    let [json_name, md_name, vba_name] = plan(path, out_dir, targets, cfg, &date)?;
    let mut written = Vec::new();

    if json_name.is_some() || md_name.is_some() {
        let (cells, _) = extract_cells(path, sheets, on_progress)?;
        if let Some(name) = &json_name {
            write(out_dir, name, &cells_to_json(&cells)?, &mut written)?;
        }
        if let Some(name) = &md_name {
            let file_name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
            let md = markdown::cells_to_markdown(&file_name, &cells, &cfg.template, &date);
            write(out_dir, name, &md, &mut written)?;
        }
    }

    if let Some(name) = &vba_name {
        on_progress(Progress::new("vba", 0, 1, "VBA"));
        match vba::extract_file(path, vba_opts) {
            Ok(components) => {
                let file_name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
                let md = markdown::vba_to_markdown(&components, &file_name, &cfg.template, &date);
                write(out_dir, name, &md, &mut written)?
            }
            Err(AppError::NoVbaProject) => {}
            Err(e) => return Err(e),
        }
        on_progress(Progress::new("vba", 1, 1, "VBA"));
    }

    Ok(written)
}
