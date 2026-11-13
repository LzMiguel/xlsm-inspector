use super::coords::to_a1;
use super::workbook::{Book, xlsx_err};
use crate::error::AppError;
use crate::model::{CellContent, CellValue, SheetCells, TableInfo};

/// Tabelas (ListObjects) da aba. Exige `book.load_tables()` antes.
/// ponytail: assume 1 linha de cabeçalho (padrão do Excel); tabela sem corpo é ignorada.
pub fn read_tables(book: &mut Book, sheet: &SheetCells) -> Result<Vec<TableInfo>, AppError> {
    let names: Vec<String> = book
        .table_names_in_sheet(&sheet.name)
        .into_iter()
        .cloned()
        .collect();
    let mut out = Vec::with_capacity(names.len());
    for name in names {
        let table = book.table_by_name_ref(&name).map_err(xlsx_err)?;
        let (Some((r0, c0)), Some((r1, _))) = (table.data().start(), table.data().end()) else {
            continue;
        };
        let (top, c1) = (
            r0.saturating_sub(1),
            c0 + table.columns().len().max(1) as u32 - 1,
        );
        let cells = (top..=r1)
            .flat_map(|r| (c0..=c1).map(move |c| to_a1(r, c)))
            .filter(|k| sheet.cells.contains_key(k))
            .collect();
        let rows = (r0..=r1)
            .map(|r| {
                (c0..=c1)
                    .map(|c| {
                        sheet
                            .cells
                            .get(&to_a1(r, c))
                            .cloned()
                            .unwrap_or(CellContent::Plain(CellValue::Empty))
                    })
                    .collect()
            })
            .collect();
        out.push(TableInfo {
            columns: table.columns().to_vec(),
            rows,
            name,
            range: format!("{}:{}", to_a1(top, c0), to_a1(r1, c1)),
            cells,
        });
    }
    Ok(out)
}
