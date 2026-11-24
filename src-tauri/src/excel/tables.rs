use super::coords::to_a1;
use super::workbook::{Book, xlsx_err};
use crate::error::AppError;
use crate::model::{CellContent, CellValue, SheetCells, SheetObject, TableInfo};

/// Limite de células materializadas por tabela (evita OOM em intervalos gigantes como A1:XFD1048576).
pub const MAX_TABLE_CELLS: usize = 500_000;

fn clamp_end_row(r0: u32, r1: u32, c0: u32, c1: u32) -> u32 {
    let cols = u64::from(c1.saturating_sub(c0)) + 1;
    let rows = u64::from(r1.saturating_sub(r0)) + 1;
    if rows.saturating_mul(cols) <= MAX_TABLE_CELLS as u64 {
        return r1;
    }
    let max_rows = (MAX_TABLE_CELLS as u64 / cols).max(1);
    let max_offset = (max_rows - 1).min(u64::from(u32::MAX)) as u32;
    r0.saturating_add(max_offset).min(r1)
}

/// Só metadados (nome e intervalo) das tabelas da aba, sem materializar o corpo. Exige `book.load_tables()`.
pub fn read_table_objects(book: &mut Book, sheet_name: &str) -> Result<Vec<SheetObject>, AppError> {
    let names: Vec<String> = book
        .table_names_in_sheet(sheet_name)
        .into_iter()
        .cloned()
        .collect();
    let mut out = Vec::with_capacity(names.len());
    for name in names {
        let table = book.table_by_name_ref(&name).map_err(xlsx_err)?;
        let (Some((r0, c0)), Some((r1, _))) = (table.data().start(), table.data().end()) else {
            continue;
        };
        let top = r0.saturating_sub(1);
        let col_span = table.columns().len().max(1).saturating_sub(1);
        let c1 = c0.saturating_add(u32::try_from(col_span).unwrap_or(u32::MAX));
        out.push(SheetObject {
            sheet: sheet_name.to_string(),
            name,
            range: Some(format!("{}:{}", to_a1(top, c0), to_a1(r1, c1))),
        });
    }
    Ok(out)
}

/// Tabelas (ListObjects) da aba. Exige `book.load_tables()` antes.
/// Assume 1 linha de cabeçalho quando `r0 > 0` (padrão do Excel); tabela sem corpo é ignorada.
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
        let top = r0.saturating_sub(1);
        let col_span = table.columns().len().max(1).saturating_sub(1);
        let c1 = c0.saturating_add(u32::try_from(col_span).unwrap_or(u32::MAX));
        let end_r = clamp_end_row(r0, r1, c0, c1);
        let cells = (top..=end_r)
            .flat_map(|r| (c0..=c1).map(move |c| to_a1(r, c)))
            .filter(|k| sheet.cells.contains_key(k))
            .collect();
        let rows = (r0..=end_r)
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
            range: format!("{}:{}", to_a1(top, c0), to_a1(end_r, c1)),
            cells,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limita_intervalo_gigante_sem_oom() {
        // 16.384 colunas x 1.048.576 linhas -> limitado a no máximo MAX_TABLE_CELLS
        let end = clamp_end_row(1, 1_048_575, 0, 16_383);
        let total = u64::from(end - 1 + 1) * 16_384;
        assert!(total <= MAX_TABLE_CELLS as u64);
        assert_eq!(clamp_end_row(1, 100, 0, 10), 100);
    }
}
