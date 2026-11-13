use std::collections::BTreeMap;
use std::path::Path;

use calamine::{Data, Range, Reader};
use indexmap::IndexMap;

use super::coords::to_a1;
use super::tables;
use super::workbook::{open, sheet_names, xlsx_err, Book};
use crate::error::AppError;
use crate::model::{CellContent, CellValue, Progress, SheetCells, Warning, WarningKind};

pub fn read_sheet(
    book: &mut Book,
    name: &str,
) -> Result<(SheetCells, Option<Warning>), AppError> {
    let values = book.worksheet_range(name).map_err(xlsx_err)?;
    let formulas = book.worksheet_formula(name).map_err(xlsx_err)?;
    Ok(extract_sheet(name, &values, &formulas))
}

/// Abre o arquivo e extrai as abas pedidas (`None` = todas), na ordem da pasta de trabalho.
pub fn extract_cells(
    path: &Path,
    sheets: Option<&[String]>,
    on_progress: &mut dyn FnMut(Progress),
) -> Result<(Vec<SheetCells>, Vec<Warning>), AppError> {
    let mut book = open(path)?;
    book.load_tables().map_err(xlsx_err)?;
    let names: Vec<String> = sheet_names(&book)
        .into_iter()
        .filter(|n| sheets.is_none_or(|s| s.contains(n)))
        .collect();

    let total = names.len();
    let (mut out, mut warnings) = (Vec::with_capacity(total), Vec::new());
    for (i, name) in names.iter().enumerate() {
        on_progress(Progress::new("cells", i, total, name));
        let (mut sheet, warning) = read_sheet(&mut book, name)?;
        sheet.tables = tables::read_tables(&mut book, &sheet)?;
        out.push(sheet);
        warnings.extend(warning);
    }
    on_progress(Progress::new("cells", total, total, ""));
    Ok((out, warnings))
}

pub fn to_value(d: &Data) -> CellValue {
    match d {
        Data::Empty => CellValue::Empty,
        Data::Bool(b) => CellValue::Bool(*b),
        Data::Int(i) => CellValue::Int(*i),
        // openpyxl devolve 84, não 84.0.
        Data::Float(f) if f.fract() == 0.0 && f.abs() < 1e15 => CellValue::Int(*f as i64),
        Data::Float(f) => CellValue::Float(*f),
        Data::String(s) => CellValue::String(s.clone()),
        Data::DateTime(dt) => CellValue::DateTime(
            dt.as_datetime()
                .map(|d| {
                    // Meia-noite → só a data ("2024-01-15"); senão "2024-01-15 10:30:00".
                    let s = d.to_string();
                    s.strip_suffix(" 00:00:00").map(str::to_string).unwrap_or(s)
                })
                .unwrap_or_else(|| dt.to_string()),
        ),
        Data::DateTimeIso(s) => CellValue::DateTime(s.clone()),
        Data::DurationIso(s) => CellValue::String(s.clone()),
        Data::Error(e) => CellValue::Error(e.to_string()),
    }
}

/// Une valores e fórmulas pela posição ABSOLUTA (os ranges podem começar em
/// células diferentes). Ordem de saída: linha a linha.
pub fn extract_sheet(
    name: &str,
    values: &Range<Data>,
    formulas: &Range<String>,
) -> (SheetCells, Option<Warning>) {
    type Slot<'a> = (Option<&'a Data>, Option<&'a str>); // (valor, fórmula)
    let mut merged: BTreeMap<(u32, u32), Slot> = BTreeMap::new();

    if let Some((r0, c0)) = values.start() {
        for (r, c, v) in values.used_cells() {
            merged.entry((r0 + r as u32, c0 + c as u32)).or_default().0 = Some(v);
        }
    }
    if let Some((r0, c0)) = formulas.start() {
        for (r, c, f) in formulas.used_cells() {
            merged.entry((r0 + r as u32, c0 + c as u32)).or_default().1 = Some(f.as_str());
        }
    }

    let mut cells = IndexMap::with_capacity(merged.len());
    let mut missing = 0usize;
    for ((r, c), (v, f)) in merged {
        let valor = v.map(to_value).unwrap_or(CellValue::Empty);
        let content = match f {
            Some(f) => {
                if valor == CellValue::Empty {
                    missing += 1;
                }
                let formula = if f.starts_with('=') {
                    f.to_string()
                } else {
                    format!("={f}")
                };
                CellContent::Formula { formula, valor }
            }
            None => CellContent::Plain(valor),
        };
        cells.insert(to_a1(r, c), content);
    }

    let warning = (missing > 0).then(|| Warning {
        kind: WarningKind::MissingCache,
        sheet: name.to_string(),
        count: missing,
        message: format!(
            "{missing} fórmula(s) sem valor em cache na aba \"{name}\". \
             Abra o arquivo no Excel, salve e feche para gerar os valores."
        ),
    });

    (
        SheetCells {
            name: name.to_string(),
            cells,
            tables: Vec::new(),
        },
        warning,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ranges() -> (Range<Data>, Range<String>) {
        // valores: B2:C4 (origem em (1,1)); fórmulas: C3:D4 (origem em (2,2)).
        let mut v = Range::new((1, 1), (3, 2));
        v.set_value((1, 1), Data::String("texto".into())); // B2
        v.set_value((2, 2), Data::Float(84.0)); // C3 (fórmula com cache)
        let mut f = Range::new((2, 2), (3, 3));
        f.set_value((2, 2), "SUM(B1:B2)".to_string()); // C3
        f.set_value((3, 3), "A1+1".to_string()); // D4 (sem cache)
        (v, f)
    }

    #[test]
    fn une_por_posicao_absoluta() {
        let (v, f) = ranges();
        let (sheet, _) = extract_sheet("Aba", &v, &f);
        assert_eq!(
            sheet.cells["B2"],
            CellContent::Plain(CellValue::String("texto".into()))
        );
        assert_eq!(
            sheet.cells["C3"],
            CellContent::Formula {
                formula: "=SUM(B1:B2)".into(),
                valor: CellValue::Int(84)
            }
        );
        assert_eq!(sheet.cells.len(), 3); // B2, C3, D4
    }

    #[test]
    fn ordem_linha_a_linha_e_aviso_sem_cache() {
        let (v, f) = ranges();
        let (sheet, warning) = extract_sheet("Aba", &v, &f);
        let keys: Vec<_> = sheet.cells.keys().map(String::as_str).collect();
        assert_eq!(keys, ["B2", "C3", "D4"]);
        assert_eq!(
            sheet.cells["D4"],
            CellContent::Formula {
                formula: "=A1+1".into(),
                valor: CellValue::Empty
            }
        );
        let w = warning.unwrap();
        assert_eq!((w.kind, w.count), (WarningKind::MissingCache, 1));
    }

    #[test]
    fn aba_vazia() {
        let (sheet, warning) = extract_sheet("Vazia", &Range::empty(), &Range::empty());
        assert!(sheet.cells.is_empty() && warning.is_none());
    }

    #[test]
    fn conversao_de_tipos() {
        assert_eq!(to_value(&Data::Float(2.5)), CellValue::Float(2.5));
        assert_eq!(to_value(&Data::Float(3.0)), CellValue::Int(3));
        assert_eq!(to_value(&Data::Bool(true)), CellValue::Bool(true));
        assert_eq!(
            to_value(&Data::Error(calamine::CellErrorType::Div0)),
            CellValue::Error("#DIV/0!".into())
        );
    }
}
