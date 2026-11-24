// Estado local do app (configurações, recentes, cache). Não importa nada do Tauri.
pub mod cache;
pub mod recent;
pub mod settings;

use std::fs;
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

static TMP_SEQ: AtomicU64 = AtomicU64::new(0);

/// Grava em arquivo temporário único e renomeia, para nunca deixar o arquivo pela metade
/// nem disputar o mesmo `.tmp` sob chamadas concorrentes.
pub(crate) fn write_atomic(dir: &Path, name: &str, content: &str) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let seq = TMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let tmp = dir.join(format!("{name}.{}.{seq}.tmp", std::process::id()));
    fs::write(&tmp, content)?;
    if let Err(e) = fs::rename(&tmp, dir.join(name)) {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(())
}
