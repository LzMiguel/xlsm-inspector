//! Exporta componentes VBA como arquivos importáveis no VBE (`.bas`, `.cls`, `.frm`).
//! O VBE lê ANSI (Windows-1252); UTF-8 viraria texto quebrado ao reimportar.

use std::path::Path;

use super::naming::{free_name, is_reserved};
use crate::error::AppError;
use crate::model::{VbaComponent, VbaComponentKind};

const INVALID_CHARS: &str = "<>:\"/\\|?*";
const MAX_STEM: usize = 100;

/// Garante um nome de arquivo seguro no Windows a partir do nome do módulo VBA (sem path traversal).
pub fn sanitize_module_name(raw: &str) -> String {
    let mut s: String = raw
        .chars()
        .map(|c| {
            if c.is_control() || INVALID_CHARS.contains(c) {
                '_'
            } else {
                c
            }
        })
        .collect();
    s = s.trim().trim_end_matches('.').to_string();
    if s.is_empty() || s.chars().all(|c| c == '.' || c == '_') {
        return "Modulo".to_string();
    }
    if is_reserved(s.split('.').next().unwrap_or_default()) {
        s.insert(0, '_');
    }
    s.chars().take(MAX_STEM).collect()
}

const CLASS_HEADER: &str = "VERSION 1.0 CLASS\nBEGIN\n  MultiUse = -1  'True\nEND\n";
const FORM_CLASS: &str = "C62A69F0-16DC-11CE-9E98-00AA00574A4F";

const HIGH: [(char, u8); 27] = [
    ('€', 0x80),
    ('‚', 0x82),
    ('ƒ', 0x83),
    ('„', 0x84),
    ('…', 0x85),
    ('†', 0x86),
    ('‡', 0x87),
    ('ˆ', 0x88),
    ('‰', 0x89),
    ('Š', 0x8A),
    ('‹', 0x8B),
    ('Œ', 0x8C),
    ('Ž', 0x8E),
    ('‘', 0x91),
    ('’', 0x92),
    ('“', 0x93),
    ('”', 0x94),
    ('•', 0x95),
    ('–', 0x96),
    ('—', 0x97),
    ('˜', 0x98),
    ('™', 0x99),
    ('š', 0x9A),
    ('›', 0x9B),
    ('œ', 0x9C),
    ('ž', 0x9E),
    ('Ÿ', 0x9F),
];

/// Windows-1252; o que não existe nela vira `?`.
pub fn encode_cp1252(s: &str) -> Vec<u8> {
    s.chars()
        .map(|c| match c as u32 {
            0..=0x7F | 0xA0..=0xFF => c as u32 as u8,
            _ => HIGH.iter().find(|(h, _)| *h == c).map_or(b'?', |(_, b)| *b),
        })
        .collect()
}

pub fn extension(kind: VbaComponentKind) -> &'static str {
    match kind {
        VbaComponentKind::UserForm => "frm",
        VbaComponentKind::Class | VbaComponentKind::Document | VbaComponentKind::Workbook => "cls",
        VbaComponentKind::Module | VbaComponentKind::Unknown => "bas",
    }
}

/// Texto do arquivo (CRLF). `.cls` e `.frm` ganham o cabeçalho que o VBE exige.
/// O `.frm` sai sem controles nem `.frx`: o layout do UserForm não é extraído.
pub fn file_text(c: &VbaComponent) -> String {
    let head = match c.kind {
        VbaComponentKind::Class | VbaComponentKind::Document | VbaComponentKind::Workbook => {
            CLASS_HEADER.to_string()
        }
        VbaComponentKind::UserForm => {
            let guid = c
                .code
                .split("VB_Base = \"0{")
                .nth(1)
                .and_then(|r| r.split('}').next())
                .unwrap_or(FORM_CLASS);
            let caption = c.name.replace('"', "\"\"");
            format!(
                "VERSION 5.00\nBegin {{{guid}}} {n}\n   Caption         =   \"{caption}\"\n   \
                 ClientHeight    =   3000\n   ClientWidth     =   4000\n   StartUpPosition =   1\nEnd\n",
                n = c.name
            )
        }
        _ => String::new(),
    };
    let text = format!("{head}{}", c.code.trim_end());
    text.lines().collect::<Vec<_>>().join("\r\n") + "\r\n"
}

/// Grava um arquivo por componente em `dir`, sem sobrescrever (usa `nome_2.ext`).
pub fn export_files(comps: &[VbaComponent], dir: &Path) -> Result<Vec<String>, AppError> {
    std::fs::create_dir_all(dir)
        .map_err(|e| AppError::WriteFailed(format!("{}: {e}", dir.display())))?;
    let mut out = Vec::new();
    for c in comps {
        let stem = sanitize_module_name(&c.name);
        let name = free_name(dir, &format!("{stem}.{}", extension(c.kind)));
        let path = dir.join(&name);
        if path.parent() != Some(dir) {
            return Err(AppError::InvalidName(name));
        }
        std::fs::write(&path, encode_cp1252(&file_text(c)))
            .map_err(|e| AppError::WriteFailed(format!("{}: {e}", path.display())))?;
        out.push(path.to_string_lossy().into_owned());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn comp(name: &str, kind: VbaComponentKind, code: &str) -> VbaComponent {
        VbaComponent {
            name: name.into(),
            kind,
            code: code.into(),
            line_count: 0,
            procedures: vec![],
        }
    }

    #[test]
    fn acentos_viram_ansi() {
        assert_eq!(encode_cp1252("Olá, ção €"), b"Ol\xE1, \xE7\xE3o \x80");
        assert_eq!(encode_cp1252("日"), b"?");
    }

    #[test]
    fn extensoes_e_cabecalhos() {
        use VbaComponentKind::*;
        assert_eq!(extension(Module), "bas");
        assert_eq!(extension(Workbook), "cls");
        assert_eq!(extension(UserForm), "frm");
        let t = file_text(&comp(
            "M",
            Module,
            "Attribute VB_Name = \"M\"\nSub A()\nEnd Sub",
        ));
        assert_eq!(t, "Attribute VB_Name = \"M\"\r\nSub A()\r\nEnd Sub\r\n");
        assert!(file_text(&comp("C", Class, "x")).starts_with("VERSION 1.0 CLASS\r\n"));
        let f = file_text(&comp(
            "F",
            UserForm,
            "Attribute VB_Base = \"0{AAAA}{BBBB}\"\nx",
        ));
        assert!(f.starts_with("VERSION 5.00\r\nBegin {AAAA} F\r\n"));
    }

    #[test]
    fn nao_sobrescreve() {
        let dir = std::env::temp_dir().join(format!("vba_files_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let c = [comp("Mod1", VbaComponentKind::Module, "Sub A()\nEnd Sub")];
        let a = export_files(&c, &dir).unwrap();
        let b = export_files(&c, &dir).unwrap();
        assert!(a[0].ends_with("Mod1.bas") && b[0].ends_with("Mod1_2.bas"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn sanitiza_path_traversal_e_nomes_reservados() {
        assert!(!sanitize_module_name("..\\..\\evil").contains('\\'));
        assert!(!sanitize_module_name("../evil").contains('/'));
        assert_eq!(sanitize_module_name("CON"), "_CON");
        assert_eq!(sanitize_module_name("..."), "Modulo");

        let dir = std::env::temp_dir().join(format!("vba_files_safe_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let c = [comp("..\\fora", VbaComponentKind::Module, "Sub A()\nEnd Sub")];
        let written = export_files(&c, &dir).unwrap();
        assert_eq!(Path::new(&written[0]).parent(), Some(dir.as_path()));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
