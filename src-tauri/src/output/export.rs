use std::fs;
use std::path::Path;

use serde::Deserialize;

use super::naming::{FileNames, OnExists, free_name, resolve};
use super::template::{OutputTemplate, today};
use super::{bundle::bundle_parts, json::cells_to_json, markdown};
use crate::error::AppError;
use crate::excel::{cells::extract_with_scope, vba, vba::VbaOptions};
use crate::model::{ExtractScope, ExtractionResult, Progress, SheetCells, VbaComponent};

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(default)]
pub struct ExportTargets {
    pub json: bool,
    pub cells_md: bool,
    pub vba_md: bool,
    /// Markdown único para IA (resumo + células + VBA). Desligado por padrão.
    pub bundle: bool,
}

impl Default for ExportTargets {
    fn default() -> Self {
        Self {
            json: true,
            cells_md: true,
            vba_md: true,
            bundle: false,
        }
    }
}

/// Nomes, modelo do Markdown e política de colisão. O padrão é o comportamento da v1.0.
#[derive(Debug, Clone, Default)]
pub struct OutputConfig {
    pub names: FileNames,
    pub template: OutputTemplate,
    pub on_exists: OnExists,
    pub scope: ExtractScope,
    /// Arquivo único dividido em partes de até N tokens (estimados). `None` = uma só parte.
    pub split_tokens: Option<usize>,
    /// Resultado já extraído com as mesmas abas, escopo e opções de VBA (vem do cache).
    pub cached: Option<ExtractionResult>,
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
) -> Result<[Option<String>; 4], AppError> {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let vars = [("arquivo", stem.as_str()), ("data", date)];
    let name =
        |on: bool, pattern: &str, ext: &str| on.then(|| resolve(pattern, ext, &vars)).transpose();
    let mut names = [
        name(targets.json, &cfg.names.json, "json")?,
        name(targets.cells_md, &cfg.names.cells_md, "md")?,
        name(targets.vba_md, &cfg.names.vba_md, "md")?,
        name(targets.bundle, &cfg.names.bundle, "md")?,
    ];

    // o Windows não diferencia maiúsculas: "A.md" e "a.md" são o mesmo arquivo
    let lower: Vec<String> = names.iter().flatten().map(|n| n.to_lowercase()).collect();
    if let Some(dup) = lower
        .iter()
        .enumerate()
        .find(|(i, n)| lower[..*i].contains(n))
    {
        return Err(AppError::InvalidName(format!(
            "dois arquivos com o mesmo nome ({})",
            dup.1
        )));
    }

    // o arquivo único é conferido à parte: dividido, o nome-base nem chega a ser gravado
    let chosen: Vec<usize> = (0..3).filter(|&i| names[i].is_some()).collect();
    let plain: Vec<String> = chosen.iter().filter_map(|&i| names[i].clone()).collect();
    for (i, n) in chosen
        .into_iter()
        .zip(settle(out_dir, plain, cfg.on_exists)?)
    {
        names[i] = Some(n);
    }
    Ok(names)
}

/// Aplica a política de colisão: `Fail` recusa se algum já existe; `Suffix` troca por um nome livre.
fn settle(
    out_dir: &Path,
    names: Vec<String>,
    on_exists: OnExists,
) -> Result<Vec<String>, AppError> {
    match on_exists {
        OnExists::Fail => {
            let existing: Vec<&str> = names
                .iter()
                .filter(|n| out_dir.join(n).exists())
                .map(String::as_str)
                .collect();
            if existing.is_empty() {
                Ok(names)
            } else {
                Err(AppError::FileExists(existing.join(", ")))
            }
        }
        OnExists::Suffix => Ok(names.iter().map(|n| free_name(out_dir, n)).collect()),
        OnExists::Overwrite => Ok(names),
    }
}

/// `nome.md` com uma parte; `nome_parte_1.md`, `nome_parte_2.md`... com várias.
fn part_names(base: &str, count: usize) -> Vec<String> {
    if count <= 1 {
        return vec![base.to_string()];
    }
    let (stem, ext) = base.rsplit_once('.').unwrap_or((base, ""));
    (1..=count)
        .map(|i| format!("{stem}_parte_{i}.{ext}"))
        .collect()
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
    let [json_name, md_name, vba_name, bundle_name] = plan(path, out_dir, targets, cfg, &date)?;
    let mut written = Vec::new();

    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();

    // com resultado em cache (mesmas abas, escopo e opções) o arquivo não é relido
    let fresh;
    let cells: &[SheetCells] = if json_name.is_some() || md_name.is_some() || bundle_name.is_some()
    {
        match &cfg.cached {
            Some(c) => &c.sheets,
            None => {
                fresh = extract_with_scope(path, sheets, cfg.scope, on_progress)?.0;
                &fresh
            }
        }
    } else {
        &[]
    };

    // sem projeto VBA, o VBA é simplesmente omitido (`None`)
    let components: Option<Vec<VbaComponent>> = if vba_name.is_some() || bundle_name.is_some() {
        on_progress(Progress::new("vba", 0, 1, "VBA"));
        let found = match &cfg.cached {
            Some(c) if c.vba.is_empty() => Err(AppError::NoVbaProject),
            Some(c) => Ok(c.vba.clone()),
            None => vba::extract_file(path, vba_opts),
        };
        on_progress(Progress::new("vba", 1, 1, "VBA"));
        match found {
            Ok(c) => Some(c),
            Err(AppError::NoVbaProject) => None,
            Err(e) => return Err(e),
        }
    } else {
        None
    };

    // as partes do arquivo único e seus nomes são decididos antes de gravar qualquer coisa
    let bundle_files: Vec<(String, String)> = match &bundle_name {
        Some(base) => {
            let parts = bundle_parts(
                &file_name,
                cells,
                components.as_deref().unwrap_or_default(),
                &cfg.template,
                &date,
                cfg.split_tokens,
            );
            let names = settle(out_dir, part_names(base, parts.len()), cfg.on_exists)?;
            names.into_iter().zip(parts).collect()
        }
        None => Vec::new(),
    };

    if let Some(name) = &json_name {
        write(out_dir, name, &cells_to_json(cells)?, &mut written)?;
    }
    if let Some(name) = &md_name {
        let md = markdown::cells_to_markdown(&file_name, cells, &cfg.template, &date);
        write(out_dir, name, &md, &mut written)?;
    }
    if let (Some(name), Some(components)) = (&vba_name, &components) {
        let md = markdown::vba_to_markdown(components, &file_name, &cfg.template, &date);
        write(out_dir, name, &md, &mut written)?;
    }
    for (name, part) in &bundle_files {
        write(out_dir, name, part, &mut written)?;
    }

    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nomes_das_partes() {
        assert_eq!(part_names("ctx.md", 0), ["ctx.md"]);
        assert_eq!(part_names("ctx.md", 1), ["ctx.md"]);
        assert_eq!(
            part_names("ctx.md", 3),
            ["ctx_parte_1.md", "ctx_parte_2.md", "ctx_parte_3.md"]
        );
    }

    #[test]
    fn colisao_das_partes() {
        let dir =
            std::env::temp_dir().join(format!("xlsm-inspector-{}-partes", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("ctx_parte_2.md"), "").unwrap();

        let names = part_names("ctx.md", 2);
        assert!(matches!(
            settle(&dir, names.clone(), OnExists::Fail),
            Err(AppError::FileExists(m)) if m == "ctx_parte_2.md"
        ));
        assert_eq!(
            settle(&dir, names.clone(), OnExists::Suffix).unwrap(),
            ["ctx_parte_1.md", "ctx_parte_2_2.md"]
        );
        assert_eq!(
            settle(&dir, names.clone(), OnExists::Overwrite).unwrap(),
            names
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
