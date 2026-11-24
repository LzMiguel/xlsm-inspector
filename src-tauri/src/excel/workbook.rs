use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

use calamine::{Reader, SheetType, Xlsx, XlsxError};

use crate::error::AppError;
use crate::model::{SheetInfo, WorkbookSummary};

pub type Book = Xlsx<BufReader<File>>;

pub(crate) fn io_err(e: io::Error) -> AppError {
    // 32/33 = ERROR_SHARING_VIOLATION / ERROR_LOCK_VIOLATION (Windows).
    if e.kind() == io::ErrorKind::PermissionDenied || matches!(e.raw_os_error(), Some(32 | 33)) {
        AppError::FileLocked(e.to_string())
    } else if e.kind() == io::ErrorKind::NotFound {
        AppError::FileNotFound(e.to_string())
    } else {
        AppError::CorruptedFile(e.to_string())
    }
}

pub(crate) fn xlsx_err(e: XlsxError) -> AppError {
    match e {
        XlsxError::Io(e) => io_err(e),
        e => AppError::CorruptedFile(e.to_string()),
    }
}

pub(crate) fn check_extension(path: &Path) -> Result<(), AppError> {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "xlsx" | "xlsm" => Ok(()),
        "" => Err(AppError::UnsupportedFormat("arquivo sem extensão".into())),
        _ => Err(AppError::UnsupportedFormat(format!(".{ext}"))),
    }
}

/// Abre o arquivo somente para leitura (`File::open`), nunca escreve.
pub fn open(path: &Path) -> Result<Book, AppError> {
    Ok(open_with_size(path)?.0)
}

fn open_with_size(path: &Path) -> Result<(Book, u64), AppError> {
    check_extension(path)?;
    let file = File::open(path).map_err(io_err)?;
    let size_bytes = file.metadata().map_err(io_err)?.len();
    let book = Xlsx::new(BufReader::new(file)).map_err(xlsx_err)?;
    Ok((book, size_bytes))
}

/// Nomes das abas de dados (ignora gráficos, macros antigas etc.).
pub fn sheet_names(book: &Book) -> Vec<String> {
    book.sheets_metadata()
        .iter()
        .filter(|s| s.typ == SheetType::WorkSheet)
        .map(|s| s.name.clone())
        .collect()
}

pub fn inspect(path: &Path) -> Result<WorkbookSummary, AppError> {
    let (mut book, size_bytes) = open_with_size(path)?;

    let mut sheets = Vec::new();
    for name in sheet_names(&book) {
        let values = book.worksheet_range(&name).map_err(xlsx_err)?;
        let formulas = book.worksheet_formula(&name).map_err(xlsx_err)?;

        let mut formula_coords: HashSet<(u32, u32)> = HashSet::new();
        if let Some((r0, c0)) = formulas.start() {
            for (r, c, _) in formulas.used_cells() {
                let rr = r0.saturating_add(u32::try_from(r).unwrap_or(u32::MAX));
                let cc = c0.saturating_add(u32::try_from(c).unwrap_or(u32::MAX));
                formula_coords.insert((rr, cc));
            }
        }
        let formula_count = formula_coords.len();
        let mut cell_count = formula_count;
        if let Some((r0, c0)) = values.start() {
            if formula_coords.is_empty() {
                cell_count += values.used_cells().count();
            } else {
                for (r, c, _) in values.used_cells() {
                    let rr = r0.saturating_add(u32::try_from(r).unwrap_or(u32::MAX));
                    let cc = c0.saturating_add(u32::try_from(c).unwrap_or(u32::MAX));
                    if !formula_coords.contains(&(rr, cc)) {
                        cell_count += 1;
                    }
                }
            }
        }

        sheets.push(SheetInfo {
            name,
            cell_count,
            formula_count,
        });
    }

    let has_vba = matches!(book.vba_project(), Ok(Some(_)));
    Ok(WorkbookSummary {
        path: path.to_string_lossy().into_owned(),
        file_name: path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        size_bytes,
        sheets,
        has_vba,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejeita_extensao() {
        assert!(matches!(
            check_extension(Path::new("a.xls")),
            Err(AppError::UnsupportedFormat(_))
        ));
        assert!(check_extension(Path::new("a.XLSM")).is_ok());
        assert!(check_extension(Path::new("a.xlsx")).is_ok());
    }

    #[test]
    fn arquivo_inexistente() {
        assert!(matches!(
            open(Path::new("nao_existe_123.xlsx")),
            Err(AppError::FileNotFound(_))
        ));
    }
}
