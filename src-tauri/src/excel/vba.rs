use std::path::Path;

use calamine::Reader;
use serde::{Deserialize, Serialize};

use super::workbook::{Book, open, xlsx_err};
use crate::error::AppError;
use crate::model::{VbaComponent, VbaComponentKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct VbaOptions {
    pub remove_blank_lines: bool,
    pub hide_attributes: bool,
}

impl Default for VbaOptions {
    fn default() -> Self {
        Self {
            remove_blank_lines: true,
            hide_attributes: false,
        }
    }
}

pub fn extract_file(path: &Path, opts: &VbaOptions) -> Result<Vec<VbaComponent>, AppError> {
    extract(&mut open(path)?, opts)
}

pub fn extract(book: &mut Book, opts: &VbaOptions) -> Result<Vec<VbaComponent>, AppError> {
    let project = book
        .vba_project()
        .map_err(xlsx_err)?
        .ok_or(AppError::NoVbaProject)?;

    let mut out = Vec::new();
    for name in project.get_module_names() {
        let raw = project
            .get_module(name)
            .map_err(|e| AppError::CorruptedFile(format!("módulo VBA \"{name}\": {e}")))?;
        if !has_code(&raw) {
            continue;
        }
        let code = clean_code(&raw, opts);
        out.push(VbaComponent {
            name: name.to_string(),
            kind: classify(name, &raw),
            line_count: code.lines().count(),
            procedures: super::vba_index::index(&code),
            code,
        });
    }
    Ok(out)
}

/// Falso quando só há linhas em branco, `Option` e `Attribute`. Comentários contam como código.
fn has_code(raw: &str) -> bool {
    raw.lines()
        .map(str::trim)
        .any(|l| !(l.is_empty() || l.starts_with("Option ") || l.starts_with("Attribute ")))
}

pub fn clean_code(code: &str, opts: &VbaOptions) -> String {
    code.lines()
        .filter(|l| !(opts.remove_blank_lines && l.trim().is_empty()))
        .filter(|l| !(opts.hide_attributes && l.trim_start().starts_with("Attribute VB_")))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn classify(name: &str, raw: &str) -> VbaComponentKind {
    use VbaComponentKind::*;

    if let Some(base) = raw
        .lines()
        .find(|l| l.trim_start().starts_with("Attribute VB_Base"))
    {
        let base = base.to_uppercase();

        if base.contains("}{") || base.contains("C62A69F0-16DC-11CE-9E98-00AA00574A4F") {
            return UserForm;
        }
        if base.contains("00020819-0000-0000-C000-000000000046") {
            return Workbook;
        }
        if base.contains("00020820-0000-0000-C000-000000000046")
            || base.contains("00020821-0000-0000-C000-000000000046")
        {
            return Document;
        }
    }

    let lower = name.to_lowercase();
    if matches!(
        lower.as_str(),
        "thisworkbook" | "estapasta_de_trabalho" | "estapastadetrabalho"
    ) {
        return Workbook;
    }
    if lower.starts_with("userform") {
        return UserForm;
    }
    let numbered = |prefix: &str| {
        lower
            .strip_prefix(prefix)
            .is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
    };
    if numbered("sheet") || numbered("planilha") || numbered("plan") {
        return Document;
    }

    if raw.contains("Attribute VB_Creatable") || raw.contains("Attribute VB_PredeclaredId") {
        return Class;
    }
    if raw.contains("Attribute VB_Name") {
        return Module;
    }
    Unknown
}

#[cfg(test)]
mod tests {
    use super::*;
    use VbaComponentKind::*;

    const CODE: &str = "Attribute VB_Name = \"M\"\r\n\r\nSub A()\r\n\r\n  ' olá\r\nEnd Sub\r\n";

    #[test]
    fn limpeza() {
        let all = VbaOptions::default();
        assert_eq!(
            clean_code(CODE, &all),
            "Attribute VB_Name = \"M\"\nSub A()\n  ' olá\nEnd Sub"
        );
        let hide = VbaOptions {
            hide_attributes: true,
            ..all
        };
        assert_eq!(clean_code(CODE, &hide), "Sub A()\n  ' olá\nEnd Sub");
        let keep = VbaOptions {
            remove_blank_lines: false,
            hide_attributes: false,
        };
        assert!(clean_code(CODE, &keep).contains("\n\n"));
    }

    #[test]
    fn modulo_sem_codigo() {
        assert!(!has_code(""));
        assert!(!has_code(
            "Attribute VB_Name = \"M\"\r\n\r\nOption Explicit\r\n  Option Base 1\r\n"
        ));
        assert!(has_code("Option Explicit\r\n' só comentário"));
        assert!(has_code(CODE));
    }

    #[test]
    fn classificacao() {
        assert_eq!(
            classify("Module1", "Attribute VB_Name = \"Module1\""),
            Module
        );
        assert_eq!(
            classify(
                "Cls",
                "Attribute VB_Name = \"Cls\"\nAttribute VB_Creatable = False"
            ),
            Class
        );
        assert_eq!(
            classify(
                "Form1",
                "Attribute VB_Base = \"0{C62A69F0-16DC-11CE-9E98-00AA00574A4F}{x}\""
            ),
            UserForm
        );
        assert_eq!(
            classify(
                "ThisWorkbook",
                "Attribute VB_Base = \"0{00020819-0000-0000-C000-000000000046}\""
            ),
            Workbook
        );
        assert_eq!(
            classify(
                "Plan1",
                "Attribute VB_Base = \"0{00020820-0000-0000-C000-000000000046}\""
            ),
            Document
        );
        assert_eq!(
            classify(
                "frmCadastro",
                "Attribute VB_Base = \"0{7253C676-8E53-474A-880C-94DEE222CD80}{D2204611-FAFA-4EAC-9CFC-C1AD0E881794}\"\nAttribute VB_Creatable = False"
            ),
            UserForm
        );
        assert_eq!(
            classify(
                "Cls",
                "Attribute VB_Base = \"0{FCFB3D2A-A0FA-1068-A738-08002B3371B5}\"\nAttribute VB_Creatable = False"
            ),
            Class
        );
        assert_eq!(classify("ThisWorkbook", ""), Workbook);
        assert_eq!(classify("EstaPastaDeTrabalho", ""), Workbook);
        assert_eq!(classify("Sheet12", ""), Document);
        assert_eq!(classify("Planilha1", ""), Document);
        assert_eq!(classify("UserForm1", ""), UserForm);
        assert_eq!(classify("Qualquer", ""), Unknown);
    }
}
