use std::collections::HashSet;
use std::fs;
use std::io::{Cursor, Write};
use std::path::Path;

use serde::Deserialize;
use zip::{ZipWriter, write::SimpleFileOptions};

use super::bundle::{add_section, bundle_units, parts_of};
use super::naming::{FileNames, OnExists, free_name_avoiding, resolve};
use super::template::{OutputTemplate, today};
use super::{json::cells_to_json, markdown};
use crate::error::AppError;
use crate::excel::{
    cells::extract_with_scope_from_book, diagnostics, power_query, vba, vba::VbaOptions, workbook,
};
use crate::model::{
    ExtractScope, ExtractionResult, PowerQuery, Progress, SheetCells, VbaComponent,
};

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(default)]
pub struct ExportTargets {
    pub json: bool,
    pub cells_md: bool,
    pub vba_md: bool,
    /// Consultas M e conexões; omitido quando o arquivo não tem Power Query.
    pub power_query: bool,
    /// Markdown único para IA (resumo + células + VBA). Desligado por padrão.
    pub bundle: bool,
    /// Entra no arquivo único; sem ele, vira um arquivo próprio. Desligado por padrão.
    pub diagnostics: bool,
}

impl Default for ExportTargets {
    fn default() -> Self {
        Self {
            json: true,
            cells_md: true,
            vba_md: true,
            power_query: true,
            bundle: false,
            diagnostics: false,
        }
    }
}

/// Módulos VBA, consultas e conexões a exportar, por nome. `None` = todos.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ItemFilter {
    pub modules: Option<Vec<String>>,
    pub queries: Option<Vec<String>>,
    pub connections: Option<Vec<String>>,
}

fn keep(list: &Option<Vec<String>>, name: &str) -> bool {
    list.as_ref().is_none_or(|l| l.iter().any(|n| n == name))
}

impl ItemFilter {
    pub fn vba(&self, all: &[VbaComponent]) -> Vec<VbaComponent> {
        all.iter()
            .filter(|c| keep(&self.modules, &c.name))
            .cloned()
            .collect()
    }

    pub fn power_query(&self, all: &PowerQuery) -> PowerQuery {
        let mut pq = all.clone();
        pq.queries.retain(|q| keep(&self.queries, &q.name));
        pq.connections.retain(|c| keep(&self.connections, &c.name));
        pq
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
    pub items: ItemFilter,
    /// Grava todos os arquivos dentro de um único `.zip`.
    pub zip: bool,
}

fn write(
    dir: &Path,
    name: &str,
    content: impl AsRef<[u8]>,
    written: &mut Vec<String>,
) -> Result<(), AppError> {
    let p = dir.join(name);
    fs::write(&p, content).map_err(|e| AppError::WriteFailed(format!("{}: {e}", p.display())))?;
    written.push(p.to_string_lossy().into_owned());
    Ok(())
}

/// Monta o `.zip` em memória: o arquivo só é gravado se tudo der certo.
fn zip_bytes(files: &[(String, String)]) -> Result<Vec<u8>, AppError> {
    let fail = |e: String| AppError::WriteFailed(format!("zip: {e}"));
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, content) in files {
        zip.start_file(name.as_str(), SimpleFileOptions::default())
            .map_err(|e| fail(e.to_string()))?;
        zip.write_all(content.as_bytes())
            .map_err(|e| fail(e.to_string()))?;
    }
    let done = zip.finish().map_err(|e| fail(e.to_string()))?;
    Ok(done.into_inner())
}

/// Nomes dos alvos escolhidos, validados e sem duplicata entre si.
/// Falha antes de qualquer extração.
fn plan(
    path: &Path,
    targets: ExportTargets,
    cfg: &OutputConfig,
    date: &str,
) -> Result<[Option<String>; 7], AppError> {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let vars = [("arquivo", stem.as_str()), ("data", date)];
    let name =
        |on: bool, pattern: &str, ext: &str| on.then(|| resolve(pattern, ext, &vars)).transpose();
    let names = [
        name(targets.json, &cfg.names.json, "json")?,
        name(targets.cells_md, &cfg.names.cells_md, "md")?,
        name(targets.vba_md, &cfg.names.vba_md, "md")?,
        name(targets.power_query, &cfg.names.power_query, "md")?,
        name(targets.bundle, &cfg.names.bundle, "md")?,
        name(
            targets.diagnostics && !targets.bundle,
            &cfg.names.diagnostics,
            "md",
        )?,
        name(cfg.zip, &cfg.names.zip, "zip")?,
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

    Ok(names)
}

/// Aplica a política de colisão respeitando também `taken` (nomes já reservados no mesmo lote).
fn settle(
    out_dir: &Path,
    names: Vec<String>,
    on_exists: OnExists,
    taken: &mut HashSet<String>,
) -> Result<Vec<String>, AppError> {
    if on_exists != OnExists::Suffix {
        let mut batch = HashSet::new();
        for n in &names {
            let low = n.to_lowercase();
            if taken.contains(&low) || !batch.insert(low) {
                return Err(AppError::InvalidName(format!(
                    "dois arquivos com o mesmo nome ({n})"
                )));
            }
        }
    }

    match on_exists {
        OnExists::Fail => {
            let existing: Vec<&str> = names
                .iter()
                .filter(|n| out_dir.join(n).exists())
                .map(String::as_str)
                .collect();
            if existing.is_empty() {
                for n in &names {
                    taken.insert(n.to_lowercase());
                }
                Ok(names)
            } else {
                Err(AppError::FileExists(existing.join(", ")))
            }
        }
        OnExists::Suffix => {
            let mut out = Vec::with_capacity(names.len());
            for n in &names {
                let chosen = free_name_avoiding(out_dir, n, taken);
                taken.insert(chosen.to_lowercase());
                out.push(chosen);
            }
            Ok(out)
        }
        OnExists::Overwrite => {
            for n in &names {
                taken.insert(n.to_lowercase());
            }
            Ok(names)
        }
    }
}

/// `nome.md` com uma parte; `nome_parte_1.md`, `nome_parte_2.md`... com várias.
fn part_names(base: &str, count: usize) -> Vec<String> {
    if count <= 1 {
        return vec![base.to_string()];
    }
    let split = base.rsplit_once('.');
    (1..=count)
        .map(|i| match split {
            Some((stem, ext)) => format!("{stem}_parte_{i}.{ext}"),
            None => format!("{base}_parte_{i}"),
        })
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
    let mut planned = plan(path, targets, cfg, &date)?;
    let mut taken = HashSet::new();

    // dentro do zip não há colisão com o disco, só entre as próprias entradas
    let policy = if cfg.zip {
        OnExists::Overwrite
    } else {
        cfg.on_exists
    };
    if let Some(name) = planned[6].take() {
        planned[6] = settle(out_dir, vec![name], cfg.on_exists, &mut HashSet::new())?.pop();
    }

    // Alvos garantidos são conferidos antes de extrair (vba/power_query podem ser omitidos;
    // bundle dividido só conhece o número de partes depois de montar o texto).
    let mut pre_idx = Vec::with_capacity(4);
    for i in [0usize, 1, 5] {
        if planned[i].is_some() {
            pre_idx.push(i);
        }
    }
    if cfg.split_tokens.is_none() && planned[4].is_some() {
        pre_idx.push(4);
    }
    let pre_names: Vec<String> = pre_idx.iter().filter_map(|&i| planned[i].clone()).collect();
    for (i, n) in pre_idx
        .into_iter()
        .zip(settle(out_dir, pre_names, policy, &mut taken)?)
    {
        planned[i] = Some(n);
    }

    let [
        json_name,
        md_name,
        mut vba_name,
        mut pq_name,
        bundle_name,
        diag_name,
        zip_name,
    ] = planned;

    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();

    let need_cells = json_name.is_some() || md_name.is_some() || bundle_name.is_some();
    let need_vba = vba_name.is_some() || bundle_name.is_some();
    let need_pq = pq_name.is_some() || bundle_name.is_some();

    // com resultado em cache (mesmas abas, escopo e opções) o arquivo não é relido nem clonado
    let fresh_cells: Vec<SheetCells>;
    let fresh_vba: Option<Vec<VbaComponent>>;
    let fresh_pq: PowerQuery;
    let empty_pq = PowerQuery::default();

    let (cells, components, pq): (&[SheetCells], Option<&[VbaComponent]>, &PowerQuery) =
        if let Some(c) = &cfg.cached {
            let cells = if need_cells { c.sheets.as_slice() } else { &[] };
            let comps = if need_vba {
                on_progress(Progress::new("vba", 0, 1, "VBA"));
                on_progress(Progress::new("vba", 1, 1, "VBA"));
                (!c.vba.is_empty()).then_some(c.vba.as_slice())
            } else {
                None
            };
            let pq = if need_pq { &c.power_query } else { &empty_pq };
            (cells, comps, pq)
        } else {
            let mut book = if need_cells || need_vba {
                Some(workbook::open(path)?)
            } else {
                None
            };
            let cells = if let (true, Some(b)) = (need_cells, book.as_mut()) {
                fresh_cells = extract_with_scope_from_book(b, sheets, cfg.scope, on_progress)?.0;
                fresh_cells.as_slice()
            } else {
                &[]
            };
            let comps = if let (true, Some(b)) = (need_vba, book.as_mut()) {
                on_progress(Progress::new("vba", 0, 1, "VBA"));
                let found = vba::extract(b, vba_opts);
                on_progress(Progress::new("vba", 1, 1, "VBA"));
                fresh_vba = match found {
                    Ok(c) => (!c.is_empty()).then_some(c),
                    Err(AppError::NoVbaProject) => None,
                    Err(e) => return Err(e),
                };
                fresh_vba.as_deref()
            } else {
                None
            };
            let pq = if need_pq {
                // Erro de leitura é propagado: nada é omitido em silêncio no export.
                fresh_pq = power_query::extract_file(path)?;
                &fresh_pq
            } else {
                &empty_pq
            };
            (cells, comps, pq)
        };

    let kept_vba = cfg.items.vba(components.unwrap_or_default());
    let components = (!kept_vba.is_empty()).then_some(kept_vba.as_slice());
    let pq = &cfg.items.power_query(pq);
    // shortcut: o diagnóstico é do arquivo inteiro e relê as células; reaproveitar o cache se pesar
    let diag = if targets.diagnostics {
        Some(markdown::diagnostics_block(&diagnostics::diagnose(path)?))
    } else {
        None
    };

    // VBA e Power Query só passam por `settle` se realmente forem gravados (evita FileExists falso)
    vba_name = match (vba_name, components) {
        (Some(name), Some(_)) => settle(out_dir, vec![name], policy, &mut taken)?
            .into_iter()
            .next(),
        _ => None,
    };
    let has_pq = !pq.queries.is_empty() || !pq.connections.is_empty();
    pq_name = match (pq_name, has_pq) {
        (Some(name), true) => settle(out_dir, vec![name], policy, &mut taken)?
            .into_iter()
            .next(),
        _ => None,
    };

    // as partes do arquivo único e seus nomes são decididos antes de gravar qualquer coisa
    let bundle_files: Vec<(String, String)> = match &bundle_name {
        Some(base) => {
            let mut units = bundle_units(
                &file_name,
                cells,
                components.unwrap_or_default(),
                pq,
                &cfg.template,
                &date,
            );
            if let Some(section) = &diag {
                add_section(&mut units, section, &cfg.template);
            }
            let parts = parts_of(&units, cfg.split_tokens);
            let names = if cfg.split_tokens.is_some() {
                settle(out_dir, part_names(base, parts.len()), policy, &mut taken)?
            } else {
                vec![base.clone()]
            };
            names.into_iter().zip(parts).collect()
        }
        None => Vec::new(),
    };

    let mut files: Vec<(String, String)> = Vec::new();
    if let Some(name) = json_name {
        files.push((name, cells_to_json(cells)?));
    }
    if let Some(name) = md_name {
        let md = markdown::cells_to_markdown(&file_name, cells, &cfg.template, &date);
        files.push((name, md));
    }
    if let (Some(name), Some(components)) = (vba_name, components) {
        let md = markdown::vba_to_markdown(components, &file_name, &cfg.template, &date);
        files.push((name, md));
    }
    if let Some(name) = pq_name {
        files.push((name, markdown::power_query_to_markdown(pq, &file_name)));
    }
    files.extend(bundle_files);
    if let (Some(name), Some(section)) = (diag_name, &diag) {
        files.push((name, format!("# {file_name}\n\n{section}")));
    }

    let mut written = Vec::new();
    match zip_name {
        Some(name) if !files.is_empty() => {
            write(out_dir, &name, zip_bytes(&files)?, &mut written)?;
        }
        _ => {
            for (name, content) in &files {
                write(out_dir, name, content, &mut written)?;
            }
        }
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
            settle(&dir, names.clone(), OnExists::Fail, &mut HashSet::new()),
            Err(AppError::FileExists(m)) if m == "ctx_parte_2.md"
        ));
        assert_eq!(
            settle(&dir, names.clone(), OnExists::Suffix, &mut HashSet::new()).unwrap(),
            ["ctx_parte_1.md", "ctx_parte_2_2.md"]
        );
        assert_eq!(
            settle(
                &dir,
                names.clone(),
                OnExists::Overwrite,
                &mut HashSet::new()
            )
            .unwrap(),
            names
        );

        // colisão entre parte dividida e nome já reservado no mesmo lote
        let mut taken: HashSet<String> = ["ctx_parte_1.md".into()].into_iter().collect();
        assert!(matches!(
            settle(&dir, names.clone(), OnExists::Fail, &mut taken),
            Err(AppError::InvalidName(_))
        ));
        assert_eq!(
            settle(&dir, names, OnExists::Suffix, &mut taken).unwrap(),
            ["ctx_parte_1_2.md", "ctx_parte_2_2.md"]
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
