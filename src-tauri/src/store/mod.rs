// Estado local do app (configurações, recentes, cache). Não importa nada do Tauri.
pub mod cache;
pub mod recent;
pub mod settings;

use std::fs;
use std::io;
use std::path::Path;

/// Grava em arquivo temporário e renomeia, para nunca deixar o arquivo pela metade.
pub(crate) fn write_atomic(dir: &Path, name: &str, content: &str) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let tmp = dir.join(format!("{name}.tmp"));
    fs::write(&tmp, content)?;
    fs::rename(&tmp, dir.join(name))
}
