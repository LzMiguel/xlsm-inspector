use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::write_atomic;
use crate::error::AppError;
use crate::excel::vba::VbaOptions;
use crate::output::naming::FileNames;
use crate::output::template::{OutputTemplate, TemplatePreset};

const FILE: &str = "settings.json";
pub const VERSION: u32 = 1;

/// Campos ausentes ou desconhecidos no JSON caem nos padrões (`#[serde(default)]`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub version: u32,
    /// Pasta onde o diálogo de exportação abre.
    pub default_out_dir: Option<String>,
    pub vba_options: VbaOptions,
    pub file_names: FileNames,
    /// Modelo ativo do Markdown.
    pub template: OutputTemplate,
    /// Predefinições do usuário (as embutidas vêm de `template::builtin_presets`).
    pub presets: Vec<TemplatePreset>,
    /// Guarda no disco o resultado das últimas 5 planilhas (store/cache.rs).
    pub use_cache: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: VERSION,
            default_out_dir: None,
            vba_options: VbaOptions::default(),
            file_names: FileNames::default(),
            template: OutputTemplate::default(),
            presets: Vec::new(),
            use_cache: true,
        }
    }
}

/// Arquivo ausente ou corrompido nunca é erro: volta aos padrões.
pub fn load(dir: &Path) -> Settings {
    fs::read_to_string(dir.join(FILE))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Grava em arquivo temporário e renomeia, para nunca deixar o JSON pela metade.
pub fn save(dir: &Path, settings: &Settings) -> Result<(), AppError> {
    let json = serde_json::to_string_pretty(&Settings {
        version: VERSION,
        ..settings.clone()
    })
    .map_err(|e| AppError::WriteFailed(e.to_string()))?;
    write_atomic(dir, FILE, &json).map_err(|e| AppError::WriteFailed(format!("configurações: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("xlsm-inspector-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn arquivo_ausente_usa_padroes() {
        assert_eq!(load(&temp("ausente")), Settings::default());
    }

    #[test]
    fn grava_e_le_de_volta() {
        let dir = temp("roundtrip");
        let s = Settings {
            default_out_dir: Some("C:\\saida".into()),
            vba_options: VbaOptions {
                remove_blank_lines: false,
                hide_attributes: true,
            },
            file_names: FileNames {
                json: "{arquivo}_{data}".into(),
                ..FileNames::default()
            },
            template: OutputTemplate {
                intro: "Olá".into(),
                ..OutputTemplate::default()
            },
            presets: vec![TemplatePreset {
                name: "Meu".into(),
                template: OutputTemplate {
                    footer: "fim".into(),
                    ..OutputTemplate::default()
                },
            }],
            ..Settings::default()
        };
        save(&dir, &s).unwrap();
        assert_eq!(load(&dir), s);
        let has_tmp = fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .any(|e| e.path().extension().is_some_and(|x| x == "tmp"));
        assert!(!has_tmp);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn json_invalido_usa_padroes() {
        let dir = temp("invalido");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(FILE), "{ isto não é json").unwrap();
        assert_eq!(load(&dir), Settings::default());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn campos_ausentes_e_desconhecidos() {
        let dir = temp("parcial");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(FILE),
            r#"{ "default_out_dir": "D:\\x", "futuro": 1, "vba_options": { "hide_attributes": true } }"#,
        )
        .unwrap();
        let s = load(&dir);
        assert_eq!(s.default_out_dir.as_deref(), Some("D:\\x"));
        assert!(s.vba_options.hide_attributes && s.vba_options.remove_blank_lines);
        assert!(s.use_cache, "o cache vem ligado por padrão");
        let _ = fs::remove_dir_all(&dir);
    }
}
