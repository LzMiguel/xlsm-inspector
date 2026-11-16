use std::path::Path;

use serde::{Deserialize, Serialize};

use super::template::fill;
use crate::error::AppError;

const MAX_STEM: usize = 100;
const INVALID_CHARS: &str = "<>:\"/\\|?*";

/// Padrões de nome (sem extensão, que é fixa). Placeholders: `{arquivo}` (sem extensão) e `{data}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileNames {
    pub json: String,
    pub cells_md: String,
    pub vba_md: String,
}

impl Default for FileNames {
    fn default() -> Self {
        Self {
            json: "mapeamento_planilha".into(),
            cells_md: "mapeamento_planilha".into(),
            vba_md: "vba_code".into(),
        }
    }
}

/// O que fazer quando o arquivo de destino já existe.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnExists {
    /// Não grava nada e devolve `FileExists` (o frontend pergunta ao usuário).
    #[default]
    Fail,
    Overwrite,
    /// `nome.ext` → `nome_2.ext`, `nome_3.ext`...
    Suffix,
}

fn reserved(base: &str) -> bool {
    let b = base.trim_end().to_ascii_uppercase();
    matches!(b.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || b.strip_prefix("COM")
            .or_else(|| b.strip_prefix("LPT"))
            .is_some_and(|n| matches!(n.as_bytes(), [b'1'..=b'9']))
}

/// Aplica os placeholders, valida para o Windows e acrescenta a extensão.
pub fn resolve(pattern: &str, ext: &str, vars: &[(&str, &str)]) -> Result<String, AppError> {
    let stem = fill(pattern, vars).trim().to_string();
    let bad = |why: &str| AppError::InvalidName(format!("\"{stem}\": {why}"));

    if stem.is_empty() {
        return Err(bad("o nome está vazio"));
    }
    if stem.chars().count() > MAX_STEM {
        return Err(bad(&format!("passa de {MAX_STEM} caracteres")));
    }
    if let Some(c) = stem
        .chars()
        .find(|c| c.is_control() || INVALID_CHARS.contains(*c))
    {
        return Err(bad(&format!("caractere não permitido ({c:?})")));
    }
    if stem.ends_with('.') {
        return Err(bad("não pode terminar com ponto"));
    }
    if reserved(stem.split('.').next().unwrap_or_default()) {
        return Err(bad("nome reservado do Windows"));
    }
    Ok(format!("{stem}.{ext}"))
}

/// `name` se estiver livre em `dir`; senão o primeiro `nome_N.ext` livre.
pub fn free_name(dir: &Path, name: &str) -> String {
    if !dir.join(name).exists() {
        return name.to_string();
    }
    let (stem, ext) = name.rsplit_once('.').unwrap_or((name, ""));
    (2..)
        .map(|n| format!("{stem}_{n}.{ext}"))
        .find(|c| !dir.join(c).exists())
        .unwrap_or_else(|| name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const V: [(&str, &str); 2] = [("arquivo", "vendas"), ("data", "2026-01-02")];

    fn invalid(p: &str) -> bool {
        matches!(resolve(p, "json", &V), Err(AppError::InvalidName(_)))
    }

    #[test]
    fn nome_valido_com_placeholders() {
        assert_eq!(
            resolve("mapeamento_planilha", "json", &V).unwrap(),
            "mapeamento_planilha.json"
        );
        assert_eq!(
            resolve("{arquivo}_{data}", "md", &V).unwrap(),
            "vendas_2026-01-02.md"
        );
        assert_eq!(
            resolve("  relatório 1 ", "md", &V).unwrap(),
            "relatório 1.md"
        );
    }

    #[test]
    fn nomes_invalidos() {
        for p in [
            "", "   ", "a/b", "a\\b", "a:b", "a?", "..", ".", "x.", "a\nb",
        ] {
            assert!(invalid(p), "{p:?} deveria ser inválido");
        }
        for p in ["CON", "con", "Nul", "COM1", "lpt9", "CON.backup", "aux "] {
            assert!(invalid(p), "{p:?} é reservado");
        }
        assert!(invalid(&"a".repeat(101)));
        assert!(!invalid(&"a".repeat(100)));
        assert!(!invalid("COM0") && !invalid("COM10") && !invalid("CONSOLE"));
    }

    #[test]
    fn valor_do_placeholder_tambem_e_validado() {
        let v = [("arquivo", "a/b")];
        assert!(resolve("{arquivo}", "json", &v).is_err());
    }

    #[test]
    fn sufixo_para_nome_livre() {
        let dir =
            std::env::temp_dir().join(format!("xlsm-inspector-{}-naming", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        assert_eq!(free_name(&dir, "a.json"), "a.json");
        fs::write(dir.join("a.json"), "").unwrap();
        assert_eq!(free_name(&dir, "a.json"), "a_2.json");
        fs::write(dir.join("a_2.json"), "").unwrap();
        assert_eq!(free_name(&dir, "a.json"), "a_3.json");
        let _ = fs::remove_dir_all(&dir);
    }
}
