use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::Path;

use calamine::{Reader, SheetType, Xlsx, XlsxError};

use super::cells;
use crate::error::AppError;
use crate::model::{CellContent, SheetInfo, WorkbookSummary};

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

fn check_extension(path: &Path) -> Result<(), AppError> {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "xlsx" | "xlsm" => Ok(()),
        _ => Err(AppError::UnsupportedFormat(format!(".{ext}"))),
    }
}

/// Abre o arquivo somente para leitura (`File::open`), nunca escreve.
pub fn open(path: &Path) -> Result<Book, AppError> {
    check_extension(path)?;
    let file = File::open(path).map_err(io_err)?;
    Xlsx::new(BufReader::new(file)).map_err(xlsx_err)
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
    let mut book = open(path)?;
    let size_bytes = fs::metadata(path).map_err(io_err)?.len();

    let mut sheets = Vec::new();
    for name in sheet_names(&book) {
        let (sheet, _) = cells::read_sheet(&mut book, &name)?;
        let formula_count = sheet
            .cells
            .values()
            .filter(|c| matches!(c, CellContent::Formula { .. }))
            .count();
        sheets.push(SheetInfo {
            name,
            cell_count: sheet.cells.len(),
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
