use std::path::Path;

use calamine::{Reader, SheetType, SheetVisible};

use super::workbook::{open, sheet_names, xlsx_err};
use super::{cells, tables};
use crate::error::AppError;
use crate::model::{
    CellContent, CellIssue, CellValue, DefinedName, Diagnostics, HiddenSheet, SheetCells,
    SheetObject,
};

/// Máximo de células listadas por categoria; o total real vai em `*_count`.
pub const MAX_LISTED: usize = 500;

/// Fórmula com `[n]` solto (`[1]Plan1!A1`, `'[2]Dados'!B2`). `Tabela[[...]]` e `Tabela[Col]`
/// não contam: o `[` vem colado em letra/dígito.
/// ponytail: um texto literal `"[1]"` dentro da fórmula também dispara; não vale um parser.
fn has_external_ref(formula: &str) -> bool {
    let b = formula.as_bytes();
    formula.match_indices('[').any(|(i, _)| {
        let rest = &b[i + 1..];
        let digits = rest.iter().take_while(|c| c.is_ascii_digit()).count();
        let attached = i > 0 && (b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_');
        digits > 0 && rest.get(digits) == Some(&b']') && !attached
    })
}

fn push(list: &mut Vec<CellIssue>, count: &mut usize, sheet: &str, cell: &str, detail: &str) {
    *count += 1;
    if list.len() < MAX_LISTED {
        list.push(CellIssue {
            sheet: sheet.to_string(),
            cell: cell.to_string(),
            detail: detail.to_string(),
        });
    }
}

/// Erros (valor `#DIV/0!` etc., com ou sem fórmula) e referências externas da aba.
fn scan_sheet(sheet: &SheetCells, d: &mut Diagnostics) {
    for (cell, c) in &sheet.cells {
        let (formula, value) = match c {
            CellContent::Plain(v) => (None, v),
            CellContent::Formula { formula, valor } => (Some(formula), valor),
        };
        if let CellValue::Error(e) = value {
            push(&mut d.errors, &mut d.error_count, &sheet.name, cell, e);
        }
        if let Some(f) = formula.filter(|f| has_external_ref(f)) {
            push(&mut d.external_refs, &mut d.external_count, &sheet.name, cell, f);
        }
    }
}

/// Lê o arquivo (somente leitura) e monta o diagnóstico.
pub fn diagnose(path: &Path) -> Result<Diagnostics, AppError> {
    let mut book = open(path)?;
    book.load_tables().map_err(xlsx_err)?;

    let mut d = Diagnostics {
        // `_xlnm.*` são nomes internos do Excel (área de impressão, filtros)
        names: book
            .defined_names()
            .iter()
            .filter(|(n, _)| !n.starts_with("_xlnm."))
            .map(|(n, r)| DefinedName {
                name: n.clone(),
                refers_to: r.clone(),
            })
            .collect(),
        hidden_sheets: book
            .sheets_metadata()
            .iter()
            .filter(|s| s.typ == SheetType::WorkSheet && s.visible != SheetVisible::Visible)
            .map(|s| HiddenSheet {
                name: s.name.clone(),
                very_hidden: s.visible == SheetVisible::VeryHidden,
            })
            .collect(),
        ..Diagnostics::default()
    };

    for name in sheet_names(&book) {
        let (mut sheet, _) = cells::read_sheet(&mut book, &name)?;
        sheet.tables = tables::read_tables(&mut book, &sheet)?;
        d.tables.extend(sheet.tables.iter().map(|t| SheetObject {
            sheet: name.clone(),
            name: t.name.clone(),
            range: Some(t.range.clone()),
        }));
        scan_sheet(&sheet, &mut d);
    }

    // tabela dinâmica ilegível não derruba o diagnóstico: só some da lista
    if let Ok(p) = book.pivot_tables() {
        d.pivots = p
            .get_pivot_tables_by_name_and_sheet()
            .into_iter()
            .map(|(sheet, name)| SheetObject {
                sheet: sheet.to_string(),
                name: name.to_string(),
                range: None,
            })
            .collect();
    }
    Ok(d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    fn sheet(cells: Vec<(&str, CellContent)>) -> SheetCells {
        SheetCells {
            name: "A".into(),
            cells: cells
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect::<IndexMap<_, _>>(),
            tables: vec![],
            sampled: None,
        }
    }

    fn err(s: &str) -> CellValue {
        CellValue::Error(s.into())
    }

    fn f(formula: &str, valor: CellValue) -> CellContent {
        CellContent::Formula {
            formula: formula.into(),
            valor,
        }
    }

    #[test]
    fn referencia_externa() {
        assert!(has_external_ref("=[1]Plan1!A1"));
        assert!(has_external_ref("=SOMA('[12]Dados'!A1:A3)"));
        assert!(!has_external_ref("=SOMA(Tabela1[Valor])"));
        assert!(!has_external_ref("=Tabela1[[#Esta Linha],[1]]"));
        assert!(!has_external_ref("=Tabela1[1]"));
        assert!(!has_external_ref("=A1+B1"));
        assert!(!has_external_ref("=[x]A1"));
        assert!(!has_external_ref("=[1"));
    }

    #[test]
    fn conta_erros_com_e_sem_formula() {
        let s = sheet(vec![
            ("A1", CellContent::Plain(err("#REF!"))),
            ("A2", f("=1/0", err("#DIV/0!"))),
            ("A3", f("=1+1", CellValue::Int(2))),
            ("A4", CellContent::Plain(CellValue::String("#DIV/0!".into()))), // texto, não erro
            ("A5", f("=[1]X!A1", CellValue::Empty)),
        ]);
        let mut d = Diagnostics::default();
        scan_sheet(&s, &mut d);

        assert_eq!(d.error_count, 2);
        let cells: Vec<_> = d.errors.iter().map(|e| (e.cell.as_str(), e.detail.as_str())).collect();
        assert_eq!(cells, [("A1", "#REF!"), ("A2", "#DIV/0!")]);
        assert_eq!((d.external_count, d.external_refs[0].cell.as_str()), (1, "A5"));
    }

    #[test]
    fn lista_cortada_mas_total_real() {
        let many: Vec<(String, CellContent)> = (1..=MAX_LISTED + 10)
            .map(|i| (format!("A{i}"), CellContent::Plain(err("#N/A"))))
            .collect();
        let s = sheet(many.iter().map(|(k, v)| (k.as_str(), v.clone())).collect());
        let mut d = Diagnostics::default();
        scan_sheet(&s, &mut d);
        assert_eq!((d.errors.len(), d.error_count), (MAX_LISTED, MAX_LISTED + 10));
    }
}
