use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

use calamine::{Reader, SheetType, Xlsx, XlsxError};

use super::power_query;
use super::vba::{self, VbaOptions};
use crate::error::AppError;
use crate::model::{SheetInfo, VbaModuleInfo, WorkbookSummary};

pub type Book = Xlsx<BufReader<File>>;

pub(crate) fn io_err(e: io::Error) -> AppError {
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

pub fn sheet_names(book: &Book) -> Vec<String> {
    book.sheets_metadata()
        .iter()
        .filter(|s| s.typ == SheetType::WorkSheet)
        .map(|s| s.name.clone())
        .collect()
}

/// Visão geral barata: não lê as células, só a dimensão declarada de cada aba.
pub fn inspect(path: &Path) -> Result<WorkbookSummary, AppError> {
    let (mut book, size_bytes) = open_with_size(path)?;
    book.load_tables().map_err(xlsx_err)?;

    let mut sheets = Vec::new();
    for name in sheet_names(&book) {
        let dim = book
            .worksheet_cells_reader(&name)
            .map_err(xlsx_err)?
            .dimensions();
        let tables = book
            .table_names_in_sheet(&name)
            .into_iter()
            .cloned()
            .collect();
        sheets.push(SheetInfo {
            name,
            rows: dim.end.0.saturating_sub(dim.start.0).saturating_add(1),
            cols: dim.end.1.saturating_sub(dim.start.1).saturating_add(1),
            tables,
        });
    }

    // VBA ou Power Query ilegíveis não impedem abrir.
    let has_vba = matches!(book.vba_project(), Ok(Some(_)));
    let vba = vba::extract(&mut book, &VbaOptions::default())
        .unwrap_or_default()
        .into_iter()
        .map(|c| VbaModuleInfo {
            name: c.name,
            kind: c.kind,
            line_count: c.line_count,
        })
        .collect();
    let pq = power_query::extract_file(path);
    let pq_unreadable = pq.is_err();
    let pq = pq.unwrap_or_default();

    Ok(WorkbookSummary {
        path: path.to_string_lossy().into_owned(),
        file_name: path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        size_bytes,
        sheets,
        has_vba,
        vba,
        queries: pq.queries.into_iter().map(|q| q.name).collect(),
        connections: pq.connections.into_iter().map(|c| c.name).collect(),
        pq_unreadable,
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
