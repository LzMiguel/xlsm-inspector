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

/// Corta a aba conforme o escopo (por linha com conteúdo). Cabeçalhos de tabela ficam sempre;
/// o corpo das tabelas segue a mesma seleção. Devolve o aviso se cortou algo.
/// ponytail: conta linhas com conteúdo, não linhas físicas; o parsing já leu tudo.
pub fn apply_scope(sheet: &mut SheetCells, scope: ExtractScope) -> Option<Warning> {
    let want = match scope {
        ExtractScope::Full => return None,
        ExtractScope::Head { rows } | ExtractScope::Sample { rows } => rows.max(1),
    };
    let mut rows: Vec<u32> = sheet.cells.keys().map(|k| row_of(k)).collect();
    rows.dedup(); // as células vêm em ordem de linha
    let total = rows.len();
    if want >= total {
        return None;
    }

    let mut keep: HashSet<u32> = match scope {
        ExtractScope::Head { .. } => rows[..want].iter().copied().collect(),
        _ => (0..want).map(|i| rows[i * total / want]).collect(),
    };
    let header = |t: &TableInfo| row_of(t.range.split(':').next().unwrap_or_default());
    keep.extend(sheet.tables.iter().map(header));

    sheet.cells.retain(|k, _| keep.contains(&row_of(k)));
    for t in &mut sheet.tables {
        let first_body = header(t) + 1;
        let body = std::mem::take(&mut t.rows);
        t.rows = body
            .into_iter()
            .enumerate()
            .filter(|(i, _)| keep.contains(&(first_body + *i as u32)))
            .map(|(_, r)| r)
            .collect();
        t.cells.retain(|k| keep.contains(&row_of(k)));
    }

    let kept = rows.iter().filter(|r| keep.contains(r)).count();
    sheet.sampled = Some(SampleInfo {
        total_rows: total,
        kept_rows: kept,
    });
    Some(Warning {
        kind: WarningKind::Sampled,
        sheet: sheet.name.clone(),
        count: kept,
        message: format!(
            "Aba \"{}\": {kept} de {total} linhas (escopo parcial).",
            sheet.name
        ),
    })
}

/// Abre o arquivo e extrai as abas pedidas (`None` = todas), na ordem da pasta de trabalho.
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
        let sampled = apply_scope(&mut sheet, scope);
        out.push(sheet);
        warnings.extend(warning);
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
            sampled: None,
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

    /// Aba com uma célula por linha, A1..A10, e uma tabela A4:A10 (cabeçalho na linha 4).
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
        // A4 é o cabeçalho da tabela: fica mesmo fora das 3 primeiras
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
        // índices 0, 2, 5, 7 → linhas 1, 3, 6, 8; mais o cabeçalho (4)
        assert_eq!(keys(&a), ["A1", "A3", "A4", "A6", "A8"]);
        // corpo da tabela: só as linhas 6 e 8
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
}
