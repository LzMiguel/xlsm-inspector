use std::fs;
use std::path::PathBuf;

use xlsm_inspector_lib::error::AppError;
use xlsm_inspector_lib::excel::cells::extract_cells;
use xlsm_inspector_lib::excel::vba::{VbaOptions, extract_file};
use xlsm_inspector_lib::excel::workbook::inspect;
use xlsm_inspector_lib::model::{
    CellContent, CellValue, SheetCells, VbaComponentKind, Warning, WarningKind,
};
use xlsm_inspector_lib::output::export::{ExportTargets, export_outputs};
use xlsm_inspector_lib::output::json::cells_to_json;
use xlsm_inspector_lib::output::markdown::cells_to_markdown;

fn fx(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn cells(name: &str) -> (Vec<SheetCells>, Vec<Warning>) {
    extract_cells(&fx(name), None, &mut |_| {}).expect("extração")
}

fn sheet<'a>(sheets: &'a [SheetCells], name: &str) -> &'a SheetCells {
    sheets
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("aba {name} ausente"))
}

fn plain(s: &SheetCells, coord: &str) -> CellValue {
    match &s.cells[coord] {
        CellContent::Plain(v) => v.clone(),
        other => panic!("{coord}: esperado valor simples, veio {other:?}"),
    }
}

fn formula(s: &SheetCells, coord: &str) -> (String, CellValue) {
    match &s.cells[coord] {
        CellContent::Formula { formula, valor } => (formula.clone(), valor.clone()),
        other => panic!("{coord}: esperada fórmula, veio {other:?}"),
    }
}

fn text(s: &str) -> CellValue {
    CellValue::String(s.into())
}

#[test]
fn simples() {
    let summary = inspect(&fx("simples.xlsx")).unwrap();
    let names: Vec<_> = summary.sheets.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["Clientes", "Produtos"]);
    assert_eq!(summary.sheets[0].cell_count, 20);
    assert_eq!(summary.sheets[1].cell_count, 16);
    assert!(!summary.has_vba);
    assert!(summary.sheets.iter().all(|s| s.formula_count == 0));

    let (sheets, warnings) = cells("simples.xlsx");
    assert!(warnings.is_empty());
    let c = sheet(&sheets, "Clientes");
    assert_eq!(plain(c, "A2"), CellValue::Int(1));
    assert_eq!(plain(c, "D2"), CellValue::DateTime("2024-01-15".into()));
    assert_eq!(plain(c, "E3"), CellValue::Bool(false));
    let p = sheet(&sheets, "Produtos");
    assert_eq!(plain(p, "C4"), CellValue::Float(129.99));
    assert_eq!(plain(p, "D3"), CellValue::Int(0));
    assert_eq!(plain(p, "D4"), CellValue::Int(-3));
}

#[test]
fn formulas() {
    let (sheets, warnings) = cells("formulas.xlsx");
    assert!(warnings.is_empty(), "{warnings:?}");
    let names: Vec<_> = sheets.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["Parâmetros", "Orçamento"]);

    let o = sheet(&sheets, "Orçamento");
    let n = o
        .cells
        .values()
        .filter(|c| matches!(c, CellContent::Formula { .. }))
        .count();
    assert_eq!(n, 10);

    assert_eq!(formula(o, "D2"), ("=B2*C2".into(), CellValue::Int(15)));
    assert_eq!(
        formula(o, "D6"),
        ("=SUM(D2:D4)".into(), CellValue::Float(34.5))
    );
    assert_eq!(formula(o, "D10").1, CellValue::Bool(true));
    assert_eq!(formula(o, "D11").1, CellValue::Error("#DIV/0!".into()));
    assert_eq!(formula(o, "D9").1, text("Entrega em São Paulo"));
    assert_eq!(formula(o, "D8").1, text("Atenção: acima do limite"));
    // referência entre abas (o Excel pode gravar com ou sem aspas)
    assert!(formula(o, "D7").0.contains("Parâmetros"));
}

#[test]
fn sem_cache() {
    let (sheets, warnings) = cells("sem_cache.xlsx");
    let s = sheet(&sheets, "Calculos");
    assert_eq!(formula(s, "B3"), ("=SUM(B1:B2)".into(), CellValue::Empty));
    assert_eq!(plain(s, "B1"), CellValue::Int(10));
    assert_eq!(warnings.len(), 1);
    assert_eq!(
        (warnings[0].kind, warnings[0].count),
        (WarningKind::MissingCache, 3)
    );
}

#[test]
fn especiais() {
    let (sheets, _) = cells("especiais.xlsx");
    let e = sheet(&sheets, "Especiais");
    assert_eq!(plain(e, "A1"), text("valor|com|pipe"));
    assert_eq!(plain(e, "A2"), text("Linha 1\nLinha 2\nLinha 3"));
    assert_eq!(plain(e, "A5"), text("00123"));
    assert_eq!(plain(e, "C10"), text("célula isolada"));
    match plain(e, "A4") {
        CellValue::String(s) => assert!(s != s.trim() && s.trim() == "espaços nas pontas"),
        v => panic!("A4: {v:?}"),
    }

    let d = sheet(&sheets, "Deslocada");
    assert_eq!(plain(d, "D5"), text("Chave"));
    assert_eq!(plain(d, "E7"), CellValue::Int(2));
    assert!(!d.cells.contains_key("A1") && !d.cells.contains_key("B1"));

    assert!(sheet(&sheets, "Vazia").cells.is_empty());

    let json = cells_to_json(&sheets).unwrap();
    assert!(json.contains("Linha 1\\nLinha 2"));
}

#[test]
fn com_vba() {
    let summary = inspect(&fx("com_vba.xlsm")).unwrap();
    assert!(summary.has_vba);

    let comps = extract_file(&fx("com_vba.xlsm"), &VbaOptions::default()).unwrap();
    let kind = |name: &str| {
        comps
            .iter()
            .find(|c| c.name == name)
            .unwrap_or_else(|| panic!("{name} ausente"))
            .kind
    };
    let attrs = |name: &str| {
        let c = comps.iter().find(|c| c.name == name).unwrap();
        c.code.lines().filter(|l| l.starts_with("Attribute")).collect::<Vec<_>>().join("\n")
    };
    assert_eq!(kind("modUtilitarios"), VbaComponentKind::Module);
    assert_eq!(
        kind("clsFuncionario"),
        VbaComponentKind::Class,
        "atributos:\n{}",
        attrs("clsFuncionario")
    );
    // o nome interno pode ser "ThisWorkbook" mesmo no Excel pt-BR
    let names: Vec<_> = comps.iter().map(|c| (c.name.as_str(), c.kind)).collect();
    assert!(
        comps.iter().any(|c| c.kind == VbaComponentKind::Workbook
            && ["ThisWorkbook", "EstaPastaDeTrabalho"].contains(&c.name.as_str())),
        "componentes: {names:?}"
    );
    assert_eq!(kind("Planilha1"), VbaComponentKind::Document);
    assert_eq!(
        kind("frmCadastro"),
        VbaComponentKind::UserForm,
        "atributos:\n{}",
        attrs("frmCadastro")
    );

    let util = comps.iter().find(|c| c.name == "modUtilitarios").unwrap();
    assert!(
        util.code.contains("Operação concluída com êxito!"),
        "codepage: {}",
        util.code
    );
    assert!(util.code.lines().all(|l| !l.trim().is_empty()));
    assert!(util.code.contains("Attribute VB_"));

    let hidden = VbaOptions {
        hide_attributes: true,
        ..VbaOptions::default()
    };
    let comps = extract_file(&fx("com_vba.xlsm"), &hidden).unwrap();
    assert!(comps.iter().all(|c| !c.code.contains("Attribute VB_")));
}

#[test]
fn sem_vba_nao_quebra() {
    let r = extract_file(&fx("sem_vba.xlsm"), &VbaOptions::default());
    assert!(r.is_ok() || matches!(r, Err(AppError::NoVbaProject)));
    // xlsx puro: sempre NoVbaProject
    assert!(matches!(
        extract_file(&fx("simples.xlsx"), &VbaOptions::default()),
        Err(AppError::NoVbaProject)
    ));
}

#[test]
fn arquivos_invalidos() {
    for f in ["corrompido.xlsx", "truncado.xlsx"] {
        assert!(
            matches!(inspect(&fx(f)), Err(AppError::CorruptedFile(_))),
            "{f} deveria dar CorruptedFile"
        );
    }
    assert!(matches!(
        inspect(&fx("nao_existe.xlsx")),
        Err(AppError::FileNotFound(_))
    ));
}

#[test]
fn exporta_tres_arquivos() {
    let out = std::env::temp_dir().join(format!("xlsm_inspector_it_{}", std::process::id()));
    fs::create_dir_all(&out).unwrap();

    let written = export_outputs(
        &fx("com_vba.xlsm"),
        &out,
        ExportTargets::default(),
        None,
        &VbaOptions::default(),
        &mut |_| {},
    )
    .unwrap();
    assert_eq!(written.len(), 3);
    for name in [
        "mapeamento_planilha.json",
        "mapeamento_planilha.md",
        "vba_code.md",
    ] {
        assert!(
            fs::metadata(out.join(name)).unwrap().len() > 0,
            "{name} vazio"
        );
    }
    assert!(
        fs::read_to_string(out.join("vba_code.md"))
            .unwrap()
            .contains("## COMPONENTE: modUtilitarios")
    );

    // sem VBA: vba_code.md é omitido, sem erro
    let written = export_outputs(
        &fx("simples.xlsx"),
        &out,
        ExportTargets::default(),
        None,
        &VbaOptions::default(),
        &mut |_| {},
    )
    .unwrap();
    assert_eq!(written.len(), 2);

    fs::remove_dir_all(&out).ok();
}

/// RNF07: abrir, extrair e exportar nunca alteram o arquivo de entrada.
#[test]
fn entrada_nunca_modificada() {
    let out = std::env::temp_dir().join(format!("xlsm_inspector_rnf07_{}", std::process::id()));
    fs::create_dir_all(&out).unwrap();

    for name in ["simples.xlsx", "com_vba.xlsm"] {
        let path = fx(name);
        let bytes = fs::read(&path).unwrap();
        let mtime = fs::metadata(&path).unwrap().modified().unwrap();

        inspect(&path).unwrap();
        extract_cells(&path, None, &mut |_| {}).unwrap();
        let _ = extract_file(&path, &VbaOptions::default());
        export_outputs(
            &path,
            &out,
            ExportTargets::default(),
            None,
            &VbaOptions::default(),
            &mut |_| {},
        )
        .unwrap();

        assert_eq!(fs::read(&path).unwrap(), bytes, "{name} foi alterado");
        assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), mtime);
    }
    fs::remove_dir_all(&out).ok();
}

/// RNF02: ~100 mil células (extração + JSON + Markdown) em menos de 5 s.
/// Gere o arquivo com `python src-tauri/tests/gerar_fixture_grande.py` e rode:
/// `cargo test --release desempenho -- --ignored --nocapture`
#[test]
#[ignore = "desempenho: precisa de fixtures/grande.xlsx e de --release"]
fn desempenho_100k_celulas() {
    let path = fx("grande.xlsx");
    assert!(path.exists(), "gere grande.xlsx com tests/gerar_fixture_grande.py");

    let t = std::time::Instant::now();
    let (sheets, _) = extract_cells(&path, None, &mut |_| {}).unwrap();
    let json = cells_to_json(&sheets).unwrap();
    let md = cells_to_markdown("grande.xlsx", &sheets);
    let elapsed = t.elapsed();

    let total: usize = sheets.iter().map(|s| s.cells.len()).sum();
    println!("{total} células, JSON {} KB, MD {} KB, em {elapsed:?}", json.len() / 1024, md.len() / 1024);
    assert!(total >= 100_000);
    assert!(elapsed.as_secs_f64() < 5.0, "RNF02 estourou: {elapsed:?}");
}
