use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::write_atomic;
use crate::error::AppError;

const FILE: &str = "recent.json";
pub const MAX: usize = 10;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecentFile {
    pub path: String,
    pub name: String,
    /// Epoch em ms da última abertura.
    pub opened_at: u64,
}

/// Arquivo ausente ou corrompido = lista vazia.
pub fn load(dir: &Path) -> Vec<RecentFile> {
    let mut list: Vec<RecentFile> = fs::read_to_string(dir.join(FILE))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    list.truncate(MAX);
    list
}

fn save(dir: &Path, list: &[RecentFile]) -> Result<(), AppError> {
    let json = serde_json::to_string_pretty(list).map_err(|e| AppError::WriteFailed(e.to_string()))?;
    write_atomic(dir, FILE, &json).map_err(|e| AppError::WriteFailed(format!("recentes: {e}")))
}

/// Move/insere o arquivo no topo, sem duplicar, respeitando o limite.
pub fn with_recent(list: Vec<RecentFile>, file: RecentFile) -> Vec<RecentFile> {
    let mut out = vec![file.clone()];
    out.extend(list.into_iter().filter(|r| r.path != file.path));
    out.truncate(MAX);
    out
}

pub fn add(dir: &Path, file: RecentFile) -> Result<Vec<RecentFile>, AppError> {
    let list = with_recent(load(dir), file);
    save(dir, &list)?;
    Ok(list)
}

pub fn remove(dir: &Path, path: &str) -> Result<Vec<RecentFile>, AppError> {
    let mut list = load(dir);
    list.retain(|r| r.path != path);
    save(dir, &list)?;
    Ok(list)
}

pub fn clear(dir: &Path) -> Result<Vec<RecentFile>, AppError> {
    save(dir, &[])?;
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(path: &str, at: u64) -> RecentFile {
        RecentFile { path: path.into(), name: path.into(), opened_at: at }
    }

    fn temp(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("xlsm-inspector-{}-recent-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn move_para_o_topo_sem_duplicar() {
        let out = with_recent(vec![f("a", 1), f("b", 2)], f("b", 5));
        let paths: Vec<_> = out.iter().map(|r| r.path.as_str()).collect();
        assert_eq!(paths, ["b", "a"]);
        assert_eq!(out[0].opened_at, 5);
    }

    #[test]
    fn respeita_o_limite() {
        let list: Vec<_> = (0..MAX).map(|i| f(&i.to_string(), 0)).collect();
        let out = with_recent(list, f("novo", 0));
        assert_eq!(out.len(), MAX);
        assert_eq!(out[0].path, "novo");
        assert!(!out.iter().any(|r| r.path == (MAX - 1).to_string()));
    }

    #[test]
    fn grava_remove_e_limpa() {
        let dir = temp("crud");
        assert!(load(&dir).is_empty());
        add(&dir, f("a", 1)).unwrap();
        add(&dir, f("b", 2)).unwrap();
        assert_eq!(load(&dir).len(), 2);
        assert_eq!(remove(&dir, "a").unwrap().len(), 1);
        assert!(clear(&dir).unwrap().is_empty() && load(&dir).is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn arquivo_corrompido_vira_lista_vazia() {
        let dir = temp("ruim");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(FILE), "[ não é json").unwrap();
        assert!(load(&dir).is_empty());
        let _ = fs::remove_dir_all(&dir);
    }
}
