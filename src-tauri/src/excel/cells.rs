use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use calamine::{Data, Range, Reader};
use indexmap::IndexMap;

use super::coords::to_a1;
use super::tables;
use super::workbook::{Book, open, sheet_names, xlsx_err};
use crate::error::AppError;
use crate::model::{
    CellContent, CellValue, ExtractScope, Progress, SampleInfo, SheetCells, TableInfo, Warning,
    WarningKind,
};

pub fn read_sheet(book: &mut Book, name: &str) -> Result<(SheetCells, Option<Warning>), AppError> {
    let values = book.worksheet_range(name).map_err(xlsx_err)?;
    let formulas = book.worksheet_formula(name).map_err(xlsx_err)?;
    Ok(extract_sheet(name, &values, &formulas))
}

fn row_of(key: &str) -> u32 {
    key.trim_start_matches(|c: char| c.is_ascii_alphabetic())
        .parse()
        .unwrap_or(0)
}

fn table_row_bounds(t: &TableInfo) -> (u32, u32) {
    let (start_s, end_s) = t.range.split_once(':').unwrap_or((&t.range, &t.range));
    let start_row = row_of(start_s);
    let end_row = row_of(end_s);
    let first_body = end_row
        .saturating_add(1)
        .saturating_sub(u32::try_from(t.rows.len()).unwrap_or(u32::MAX))
        .max(start_row);
    (start_row, first_body)
}

pub fn apply_scope(sheet: &mut SheetCells, scope: ExtractScope) -> Option<Warning> {
    let want = match scope {
        ExtractScope::Full => return None,
        ExtractScope::Head { rows } | ExtractScope::Sample { rows } => rows.max(1),
    };
    let mut rows: Vec<u32> = sheet.cells.keys().map(|k| row_of(k)).collect();
    rows.dedup();
    let total = rows.len();
    if want >= total {
        return None;
    }

    let mut keep: HashSet<u32> = match scope {
        ExtractScope::Head { .. } => rows[..want].iter().copied().collect(),
        _ => (0..want)
            .map(|i| {
                let idx = (i as u128 * total as u128 / want as u128) as usize;
                rows[idx]
            })
            .collect(),
    };
    for t in &sheet.tables {
        let (start_row, first_body) = table_row_bounds(t);
        if first_body > start_row {
            keep.insert(start_row);
        }
    }

    let kept = rows.iter().filter(|r| keep.contains(r)).count();
    if kept >= total {
        return None;
    }

    sheet.cells.retain(|k, _| keep.contains(&row_of(k)));
    for t in &mut sheet.tables {
        let (_, first_body) = table_row_bounds(t);
        let body = std::mem::take(&mut t.rows);
        t.rows = body
            .into_iter()
            .enumerate()
            .filter(|(i, _)| {
                keep.contains(&first_body.saturating_add(u32::try_from(*i).unwrap_or(u32::MAX)))
            })
            .map(|(_, r)| r)
            .collect();
        t.cells.retain(|k| keep.contains(&row_of(k)));
    }

    sheet.sampled = Some(SampleInfo {
        total_rows: total,
        kept_rows: kept,
    });
    Some(Warning {
        kind: WarningKind::Sampled,
        sheet: sheet.name.clone(),
        count: kept,
        message: format!(
            "Planilha \"{}\": {kept} de {total} linhas (escopo parcial).",
            sheet.name
        ),
    })
}

pub fn extract_cells(
    path: &Path,
    sheets: Option<&[String]>,
    on_progress: &mut dyn FnMut(Progress),
) -> Result<(Vec<SheetCells>, Vec<Warning>), AppError> {
    extract_with_scope(path, sheets, ExtractScope::Full, on_progress)
}

pub fn extract_with_scope(
    path: &Path,
    sheets: Option<&[String]>,
    scope: ExtractScope,
    on_progress: &mut dyn FnMut(Progress),
) -> Result<(Vec<SheetCells>, Vec<Warning>), AppError> {
    let mut book = open(path)?;
    extract_with_scope_from_book(&mut book, sheets, scope, on_progress)
}

pub fn extract_with_scope_from_book(
    book: &mut Book,
    sheets: Option<&[String]>,
    scope: ExtractScope,
    on_progress: &mut dyn FnMut(Progress),
) -> Result<(Vec<SheetCells>, Vec<Warning>), AppError> {
    book.load_tables().map_err(xlsx_err)?;
    let names: Vec<String> = sheet_names(book)
        .into_iter()
        .filter(|n| sheets.is_none_or(|s| s.contains(n)))
        .collect();

    let total = names.len();
    let (mut out, mut warnings) = (Vec::with_capacity(total), Vec::new());
    for (i, name) in names.iter().enumerate() {
        on_progress(Progress::new("cells", i, total, name));
        let (mut sheet, warning) = read_sheet(book, name)?;
        let (tbls, table_warnings) = tables::read_tables(book, &sheet)?;
        sheet.tables = tbls;
        let sampled = apply_scope(&mut sheet, scope);
        out.push(sheet);
        warnings.extend(warning);
        warnings.extend(table_warnings);
        warnings.extend(sampled);
    }
    on_progress(Progress::new("cells", total, total, ""));
    Ok((out, warnings))
}

pub fn to_value(d: &Data) -> CellValue {
    match d {
        Data::Empty => CellValue::Empty,
        Data::Bool(b) => CellValue::Bool(*b),
        Data::Int(i) => CellValue::Int(*i),
        Data::Float(f) if f.fract() == 0.0 && f.abs() < 1e15 => CellValue::Int(*f as i64),
        Data::Float(f) => CellValue::Float(*f),
        Data::String(s) => CellValue::String(s.clone()),
        Data::DateTime(dt) => CellValue::DateTime(
            dt.as_datetime()
                .map(|d| {
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

pub fn extract_sheet(
    name: &str,
    values: &Range<Data>,
    formulas: &Range<String>,
) -> (SheetCells, Option<Warning>) {
    type Slot<'a> = (Option<&'a Data>, Option<&'a str>);
    let mut merged: BTreeMap<(u32, u32), Slot> = BTreeMap::new();

    if let Some((r0, c0)) = values.start() {
        for (r, c, v) in values.used_cells() {
            let rr = r0.saturating_add(u32::try_from(r).unwrap_or(u32::MAX));
            let cc = c0.saturating_add(u32::try_from(c).unwrap_or(u32::MAX));
            merged.entry((rr, cc)).or_default().0 = Some(v);
        }
    }
    if let Some((r0, c0)) = formulas.start() {
        for (r, c, f) in formulas.used_cells() {
            let rr = r0.saturating_add(u32::try_from(r).unwrap_or(u32::MAX));
            let cc = c0.saturating_add(u32::try_from(c).unwrap_or(u32::MAX));
            merged.entry((rr, cc)).or_default().1 = Some(f.as_str());
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
            "{missing} fórmula(s) sem valor em cache na planilha \"{name}\". \
             Abra o arquivo no Excel, salve e feche para gerar os valores."
        ),
    });

    (
        SheetCells {
            name: name.to_string(),
            cells,
            tables: Vec::new(),
            sampled: None,
        },
        warning,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ranges() -> (Range<Data>, Range<String>) {
        let mut v = Range::new((1, 1), (3, 2));
        v.set_value((1, 1), Data::String("texto".into()));
        v.set_value((2, 2), Data::Float(84.0));
        let mut f = Range::new((2, 2), (3, 3));
        f.set_value((2, 2), "SUM(B1:B2)".to_string());
        f.set_value((3, 3), "A1+1".to_string());
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
        assert_eq!(sheet.cells.len(), 3);
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

    fn sheet10() -> SheetCells {
        let cells: IndexMap<String, CellContent> = (1..=10)
            .map(|r| (format!("A{r}"), CellContent::Plain(CellValue::Int(r))))
            .collect();
        let table = TableInfo {
            name: "T".into(),
            range: "A4:A10".into(),
            columns: vec!["c".into()],
            rows: (5..=10)
                .map(|r| vec![CellContent::Plain(CellValue::Int(r))])
                .collect(),
            cells: (4..=10).map(|r| format!("A{r}")).collect(),
        };
        SheetCells {
            name: "S".into(),
            cells,
            tables: vec![table],
            sampled: None,
        }
    }

    fn keys(s: &SheetCells) -> Vec<&str> {
        s.cells.keys().map(String::as_str).collect()
    }

    #[test]
    fn escopo_completo_ou_maior_que_o_total_nao_corta() {
        for scope in [
            ExtractScope::Full,
            ExtractScope::Head { rows: 10 },
            ExtractScope::Sample { rows: 99 },
        ] {
            let mut s = sheet10();
            assert!(apply_scope(&mut s, scope).is_none());
            assert!(s.sampled.is_none() && s.cells.len() == 10);
        }
    }

    #[test]
    fn head_pega_as_primeiras_linhas() {
        let mut s = sheet10();
        let w = apply_scope(&mut s, ExtractScope::Head { rows: 3 }).unwrap();
        assert_eq!(keys(&s), ["A1", "A2", "A3", "A4"]);
        assert_eq!(
            s.sampled,
            Some(SampleInfo {
                total_rows: 10,
                kept_rows: 4
            })
        );
        assert_eq!(w.kind, WarningKind::Sampled);
        assert!(s.tables[0].rows.is_empty() && s.tables[0].cells == ["A4"]);
    }

    #[test]
    fn amostra_e_deterministica_e_inclui_a_primeira() {
        let (mut a, mut b) = (sheet10(), sheet10());
        apply_scope(&mut a, ExtractScope::Sample { rows: 4 });
        apply_scope(&mut b, ExtractScope::Sample { rows: 4 });
        assert_eq!(keys(&a), keys(&b));
        assert_eq!(keys(&a), ["A1", "A3", "A4", "A6", "A8"]);
        assert_eq!(a.tables[0].rows.len(), 2);
        assert_eq!(a.tables[0].cells, ["A4", "A6", "A8"]);
    }

    #[test]
    fn aba_vazia_e_rows_zero() {
        let mut vazia = SheetCells {
            name: "V".into(),
            cells: IndexMap::new(),
            tables: vec![],
            sampled: None,
        };
        assert!(apply_scope(&mut vazia, ExtractScope::Sample { rows: 5 }).is_none());
        let mut s = sheet10();
        apply_scope(&mut s, ExtractScope::Head { rows: 0 }); // 0 vira 1
        assert!(s.cells.contains_key("A1") && s.cells.len() == 2);
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

    #[test]
    fn cabecalho_que_completa_todas_as_linhas_nao_marca_sampled() {
        let mut s = SheetCells {
            name: "S".into(),
            cells: (1..=4)
                .map(|r| (format!("A{r}"), CellContent::Plain(CellValue::Int(r))))
                .collect(),
            tables: vec![TableInfo {
                name: "T".into(),
                range: "A4:A5".into(),
                columns: vec!["c".into()],
                rows: vec![vec![CellContent::Plain(CellValue::Int(5))]],
                cells: vec!["A4".into()],
            }],
            sampled: None,
        };
        assert!(apply_scope(&mut s, ExtractScope::Head { rows: 3 }).is_none());
        assert!(s.sampled.is_none());
        assert_eq!(s.cells.len(), 4);
    }

    #[test]
    fn tabela_sem_cabecalho_na_linha_1_mantem_alinhamento() {
        let cells: IndexMap<String, CellContent> = (1..=5)
            .map(|r| (format!("A{r}"), CellContent::Plain(CellValue::Int(r))))
            .collect();
        let mut s = SheetCells {
            name: "S".into(),
            cells,
            tables: vec![TableInfo {
                name: "SemCab".into(),
                range: "A1:A5".into(),
                columns: vec!["c".into()],
                rows: (1..=5)
                    .map(|r| vec![CellContent::Plain(CellValue::Int(r))])
                    .collect(),
                cells: (1..=5).map(|r| format!("A{r}")).collect(),
            }],
            sampled: None,
        };
        let w = apply_scope(&mut s, ExtractScope::Head { rows: 2 }).unwrap();
        assert_eq!(w.kind, WarningKind::Sampled);
        assert_eq!(s.tables[0].rows.len(), 2);
        assert_eq!(
            s.tables[0].rows[0][0],
            CellContent::Plain(CellValue::Int(1))
        );
        assert_eq!(
            s.tables[0].rows[1][0],
            CellContent::Plain(CellValue::Int(2))
        );
    }
}
