use std::fs;
use std::path::{Path, PathBuf};

use xlsm_inspector_lib::error::AppError;
use xlsm_inspector_lib::excel::cells::extract_cells;
use xlsm_inspector_lib::excel::diagnostics::diagnose;
use xlsm_inspector_lib::excel::vba::{VbaOptions, extract_file};
use xlsm_inspector_lib::excel::workbook::inspect;
use xlsm_inspector_lib::model::{
    CellContent, CellValue, ExtractScope, SheetCells, VbaComponentKind, Warning, WarningKind,
};
use xlsm_inspector_lib::output::export::{ExportTargets, ItemFilter, OutputConfig, export_outputs};
use xlsm_inspector_lib::output::json::cells_to_json;
use xlsm_inspector_lib::output::markdown::cells_to_markdown;
use xlsm_inspector_lib::output::naming::{FileNames, OnExists};
use xlsm_inspector_lib::output::template::OutputTemplate;
use xlsm_inspector_lib::store::cache;

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
    // dimensão declarada: as duas abas são blocos cheios
    let area: Vec<_> = summary.sheets.iter().map(|s| s.rows * s.cols).collect();
    assert_eq!(area, [20, 16]);
    assert!(!summary.has_vba && summary.vba.is_empty());
    assert!(summary.queries.is_empty() && !summary.pq_unreadable);

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
    assert!(
        summary
            .vba
            .iter()
            .any(|m| m.name == "modUtilitarios" && m.line_count > 0)
    );

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
        c.code
            .lines()
            .filter(|l| l.starts_with("Attribute"))
            .collect::<Vec<_>>()
            .join("\n")
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
        &OutputConfig::default(),
        &mut |_| {},
    )
    .unwrap();
    assert_eq!(written.len(), 3);
    for name in [
        "com_vba_celulas.json",
        "com_vba_celulas.md",
        "com_vba_codigo_vba.md",
    ] {
        assert!(
            fs::metadata(out.join(name)).unwrap().len() > 0,
            "{name} vazio"
        );
    }
    assert!(
        fs::read_to_string(out.join("com_vba_codigo_vba.md"))
            .unwrap()
            .contains("## COMPONENTE: modUtilitarios")
    );

    // sem VBA: o código VBA em .md é omitido, sem erro (sobrescreve os da rodada anterior)
    let cfg = OutputConfig {
        on_exists: OnExists::Overwrite,
        ..OutputConfig::default()
    };
    let written = export_outputs(
        &fx("simples.xlsx"),
        &out,
        ExportTargets::default(),
        None,
        &VbaOptions::default(),
        &cfg,
        &mut |_| {},
    )
    .unwrap();
    assert_eq!(written.len(), 2);

    fs::remove_dir_all(&out).ok();
}

fn export_to(dir: &Path, cfg: &OutputConfig) -> Result<Vec<String>, AppError> {
    fs::create_dir_all(dir).unwrap();
    export_outputs(
        &fx("simples.xlsx"),
        dir,
        ExportTargets::default(),
        None,
        &VbaOptions::default(),
        cfg,
        &mut |_| {},
    )
}

/// Modelo personalizado muda só o Markdown; o JSON continua idêntico.
#[test]
fn modelo_personalizado_nao_altera_json() {
    let base = std::env::temp_dir().join(format!("xlsm_inspector_tpl_{}", std::process::id()));
    let (a, b) = (base.join("padrao"), base.join("custom"));

    export_to(&a, &OutputConfig::default()).unwrap();
    let custom = OutputConfig {
        template: OutputTemplate {
            title: "# Outro título: {arquivo}".into(),
            intro: "Contexto.".into(),
            ..OutputTemplate::default()
        },
        ..OutputConfig::default()
    };
    export_to(&b, &custom).unwrap();

    let read = |d: &Path, n: &str| fs::read(d.join(n)).unwrap();
    assert_eq!(
        read(&a, "simples_celulas.json"),
        read(&b, "simples_celulas.json")
    );
    assert_ne!(
        read(&a, "simples_celulas.md"),
        read(&b, "simples_celulas.md")
    );
    assert!(
        fs::read_to_string(b.join("simples_celulas.md"))
            .unwrap()
            .starts_with("# Outro título: simples.xlsx\n\nContexto.\n")
    );
    fs::remove_dir_all(&base).ok();
}

/// Escopo parcial: o `.md` ganha o aviso, o JSON só tem as linhas mantidas.
#[test]
fn exporta_amostra_com_aviso() {
    let dir = std::env::temp_dir().join(format!("xlsm_inspector_scope_{}", std::process::id()));
    let cfg = OutputConfig {
        scope: ExtractScope::Head { rows: 1 },
        ..OutputConfig::default()
    };
    export_to(&dir, &cfg).unwrap();

    let md = fs::read_to_string(dir.join("simples_celulas.md")).unwrap();
    assert!(md.contains("> Amostra: 1 de "), "aviso ausente:\n{md}");

    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.join("simples_celulas.json")).unwrap())
            .unwrap();
    let (full, _) = cells("simples.xlsx");
    let total: usize = full.iter().map(|s| s.cells.len()).sum();
    let kept: usize = json
        .as_object()
        .unwrap()
        .values()
        .map(|s| s.as_object().unwrap().len())
        .sum();
    assert!(kept < total);
    fs::remove_dir_all(&dir).ok();
}

/// Com resultado em cache a exportação usa ele e não relê o arquivo (o caminho nem precisa existir).
#[test]
fn exporta_do_cache_sem_reler_o_arquivo() {
    let dir = std::env::temp_dir().join(format!("xlsm_inspector_cached_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut cells = indexmap::IndexMap::new();
    cells.insert("A1".to_string(), CellContent::Plain(CellValue::Int(7)));
    let cached = xlsm_inspector_lib::model::ExtractionResult {
        sheets: vec![SheetCells {
            name: "SoNoCache".into(),
            cells,
            tables: vec![],
            sampled: None,
        }],
        vba: vec![],
        warnings: vec![],
        power_query: Default::default(),
    };
    let cfg = OutputConfig {
        cached: Some(cached),
        ..OutputConfig::default()
    };

    let written = export_outputs(
        &dir.join("nao_existe.xlsx"),
        &dir,
        ExportTargets::default(),
        None,
        &VbaOptions::default(),
        &cfg,
        &mut |_| {},
    )
    .unwrap();
    assert_eq!(written.len(), 2);
    let md = fs::read_to_string(dir.join("nao_existe_celulas.md")).unwrap();
    assert!(md.contains("## Aba: SoNoCache") && md.contains("| **A1** | 7 |"));
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn cache_ponta_a_ponta() {
    let tmp = std::env::temp_dir().join(format!("xlsm_inspector_cache_{}", std::process::id()));
    let data = tmp.join("data");
    fs::create_dir_all(&tmp).unwrap();
    let book = tmp.join("a.xlsx");
    fs::copy(fx("simples.xlsx"), &book).unwrap();
    let v = VbaOptions::default();

    assert!(
        cache::open(&data, &book, true, false, v)
            .unwrap()
            .cached
            .is_none()
    );
    let res = cache::extract(&data, &book, true, None, ExtractScope::Full, v, &mut |_| {}).unwrap();

    let hit = cache::open(&data, &book, true, false, v)
        .unwrap()
        .cached
        .expect("deveria vir do cache");
    assert_eq!(
        cells_to_json(&hit.result.sheets).unwrap(),
        cells_to_json(&res.sheets).unwrap()
    );
    assert_eq!(hit.scope, ExtractScope::Full);

    let outras = VbaOptions {
        remove_blank_lines: false,
        hide_attributes: true,
    };
    assert!(
        cache::open(&data, &book, true, false, outras)
            .unwrap()
            .cached
            .is_none()
    );
    assert!(
        cache::open(&data, &book, false, false, v)
            .unwrap()
            .cached
            .is_none()
    ); // cache desligado

    // arquivo "salvo de novo": mesma coisa, outra data
    let f = fs::OpenOptions::new().write(true).open(&book).unwrap();
    f.set_modified(std::time::SystemTime::now() + std::time::Duration::from_secs(120))
        .unwrap();
    drop(f);
    assert!(
        cache::open(&data, &book, true, false, v)
            .unwrap()
            .cached
            .is_none()
    );
    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn nomes_e_colisoes() {
    let dir = std::env::temp_dir().join(format!("xlsm_inspector_names_{}", std::process::id()));
    let names = FileNames {
        json: "{arquivo}_dados".into(),
        ..FileNames::default()
    };
    let cfg = OutputConfig {
        names,
        ..OutputConfig::default()
    };

    let written = export_to(&dir, &cfg).unwrap();
    assert!(dir.join("simples_dados.json").exists() && written.len() == 2);

    // já existem: por padrão não grava nada e avisa
    assert!(matches!(
        export_to(&dir, &cfg),
        Err(AppError::FileExists(_))
    ));
    // sufixo: cria _2 sem tocar nos originais
    let sufixo = OutputConfig {
        on_exists: OnExists::Suffix,
        ..cfg.clone()
    };
    export_to(&dir, &sufixo).unwrap();
    assert!(dir.join("simples_dados_2.json").exists() && dir.join("simples_celulas_2.md").exists());

    // nome inválido e nomes repetidos falham antes de gravar
    let ruim = OutputConfig {
        names: FileNames {
            json: "a/b".into(),
            ..FileNames::default()
        },
        ..OutputConfig::default()
    };
    assert!(matches!(
        export_to(&dir, &ruim),
        Err(AppError::InvalidName(_))
    ));
    let igual = OutputConfig {
        names: FileNames {
            vba_md: "{arquivo}_celulas".into(),
            ..FileNames::default()
        },
        ..OutputConfig::default()
    };
    assert!(matches!(
        export_to(&dir, &igual),
        Err(AppError::InvalidName(_))
    ));
    fs::remove_dir_all(&dir).ok();
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
            &OutputConfig {
                on_exists: OnExists::Overwrite,
                ..OutputConfig::default()
            },
            &mut |_| {},
        )
        .unwrap();

        assert_eq!(fs::read(&path).unwrap(), bytes, "{name} foi alterado");
        assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), mtime);
    }
    fs::remove_dir_all(&out).ok();
}

#[test]
#[ignore = "desempenho: precisa de fixtures/grande.xlsx e de --release"]
fn desempenho_100k_celulas() {
    let path = fx("grande.xlsx");
    assert!(
        path.exists(),
        "gere grande.xlsx com tests/gerar_fixture_grande.py"
    );

    let t = std::time::Instant::now();
    let (sheets, _) = extract_cells(&path, None, &mut |_| {}).unwrap();
    let json = cells_to_json(&sheets).unwrap();
    let md = cells_to_markdown(
        "grande.xlsx",
        &sheets,
        &OutputTemplate::default(),
        "2026-01-01",
    );
    let elapsed = t.elapsed();

    let total: usize = sheets.iter().map(|s| s.cells.len()).sum();
    println!(
        "{total} células, JSON {} KB, MD {} KB, em {elapsed:?}",
        json.len() / 1024,
        md.len() / 1024
    );
    assert!(total >= 100_000);
    assert!(elapsed.as_secs_f64() < 5.0, "RNF02 estourou: {elapsed:?}");
}

#[test]
fn diagnostico() {
    let path = fx("diagnostico.xlsx");
    assert!(
        path.exists(),
        "gere diagnostico.xlsx com tests/gerar_fixture_diagnostico.py"
    );
    let d = diagnose(&path).expect("diagnóstico");

    assert_eq!(d.error_count, 1);
    assert_eq!(
        (
            d.errors[0].sheet.as_str(),
            d.errors[0].cell.as_str(),
            d.errors[0].detail.as_str()
        ),
        ("Dados", "D1", "#DIV/0!")
    );
    assert_eq!(
        (d.external_count, d.external_refs[0].cell.as_str()),
        (1, "D2")
    );
    assert!(
        d.names
            .iter()
            .any(|n| n.name == "Total" && n.refers_to.contains("Dados"))
    );
    let hidden: Vec<_> = d
        .hidden_sheets
        .iter()
        .map(|h| (h.name.as_str(), h.very_hidden))
        .collect();
    assert_eq!(hidden, [("Oculta", false), ("MuitoOculta", true)]);
    assert_eq!(d.tables.len(), 1);
    assert_eq!(
        (d.tables[0].name.as_str(), d.tables[0].range.as_deref()),
        ("tblItens", Some("A1:B3"))
    );
}

#[test]
fn resumo_lista_as_tabelas_da_aba() {
    let summary = inspect(&fx("diagnostico.xlsx")).unwrap();
    let tables: Vec<_> = summary.sheets.iter().flat_map(|s| &s.tables).collect();
    assert_eq!(tables, ["tblItens"]);
}

#[test]
fn diagnostico_sem_problemas() {
    let d = diagnose(&fx("simples.xlsx")).expect("diagnóstico");
    assert_eq!((d.error_count, d.external_count), (0, 0));
    assert!(d.hidden_sheets.is_empty());
}

/// RF19 (saída para IA): o arquivo único junta resumo, células e VBA; dividido, as partes reproduzem o único.
#[test]
fn arquivo_unico_para_ia() {
    let base = std::env::temp_dir().join(format!("xlsm_inspector_bundle_{}", std::process::id()));
    let only_bundle = ExportTargets {
        json: false,
        cells_md: false,
        vba_md: false,
        power_query: false,
        bundle: true,
        diagnostics: false,
    };
    let run = |dir: &Path, file: &str, split_tokens: Option<usize>| {
        fs::create_dir_all(dir).unwrap();
        export_outputs(
            &fx(file),
            dir,
            only_bundle,
            None,
            &VbaOptions::default(),
            &OutputConfig {
                split_tokens,
                ..OutputConfig::default()
            },
            &mut |_| {},
        )
        .unwrap()
    };

    // uma parte, com VBA
    let (a, b) = (base.join("unico"), base.join("partes"));
    let written = run(&a, "com_vba.xlsm", None);
    assert_eq!(written.len(), 1);
    let unico = fs::read_to_string(&written[0]).unwrap();
    assert!(written[0].ends_with("com_vba_contexto_ia.md"));
    assert!(unico.contains("## Resumo") && unico.contains("## COMPONENTE:"));

    // limite mínimo: um bloco por parte; juntas, idênticas ao único
    let partes = run(&b, "com_vba.xlsm", Some(1));
    assert!(partes.len() > 1);
    assert!(partes[0].ends_with("com_vba_contexto_ia_parte_1.md"));
    let juntas: String = partes
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect();
    assert_eq!(juntas, unico);
    for p in &partes {
        let txt = fs::read_to_string(p).unwrap();
        assert_eq!(txt.matches("```").count() % 2, 0, "{p}");
    }

    // sem VBA: sem seção de componentes
    let sem = run(&base.join("sem_vba"), "simples.xlsx", None);
    assert!(!fs::read_to_string(&sem[0]).unwrap().contains("COMPONENTE"));

    fs::remove_dir_all(&base).ok();
}

#[test]
fn indice_de_procedimentos_na_fixture() {
    let comps = extract_file(&fx("com_vba.xlsm"), &VbaOptions::default()).unwrap();
    let procs: Vec<_> = comps.iter().flat_map(|c| &c.procedures).collect();
    assert!(!procs.is_empty());
    for c in &comps {
        for p in &c.procedures {
            assert!(1 <= p.start_line && p.start_line <= p.end_line && p.end_line <= c.line_count);
            assert!(
                c.code
                    .lines()
                    .nth(p.start_line - 1)
                    .unwrap()
                    .contains(&p.name)
            );
        }
    }
}

/// RF13: arquivo sem Power Query devolve listas vazias, sem erro.
#[test]
fn power_query_vazio_nas_fixtures_sem_consultas() {
    for f in ["simples.xlsx", "com_vba.xlsm"] {
        let pq = xlsm_inspector_lib::excel::power_query::extract_file(&fx(f)).unwrap();
        assert!(pq.queries.is_empty() && pq.connections.is_empty(), "{f}");
    }
}

/// RF13: fixture `power_query.xlsx` (a ser adicionada): duas consultas (uma com espaço e acento no
/// nome) e uma conexão. Enquanto o arquivo não existir, o teste é ignorado.
#[test]
fn power_query_fixture_real() {
    let path = fx("power_query.xlsx");
    if !path.exists() {
        eprintln!("fixture power_query.xlsx ausente: teste ignorado");
        return;
    }
    let pq = xlsm_inspector_lib::excel::power_query::extract_file(&path).unwrap();
    assert_eq!(pq.queries.len(), 2, "consultas: {:?}", pq.queries);
    assert!(
        pq.queries
            .iter()
            .any(|q| q.name.contains(' ') && !q.name.is_ascii()),
        "esperava uma consulta com espaço e acento no nome"
    );
    assert!(pq.queries.iter().all(|q| !q.code.is_empty()));
    assert_eq!(pq.connections.len(), 1, "conexões: {:?}", pq.connections);

    // sai separado na exportação e no arquivo único
    let dir = std::env::temp_dir().join(format!("xlsm_inspector_pq_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let targets = ExportTargets {
        json: false,
        cells_md: false,
        vba_md: false,
        power_query: true,
        bundle: true,
        diagnostics: false,
    };
    export_outputs(
        &path,
        &dir,
        targets,
        None,
        &VbaOptions::default(),
        &OutputConfig::default(),
        &mut |_| {},
    )
    .unwrap();
    let md = fs::read_to_string(dir.join("power_query_power_query.md")).unwrap();
    assert!(md.contains("## Consulta: ") && md.contains("## Conexões"));
    let bundle = fs::read_to_string(dir.join("power_query_contexto_ia.md")).unwrap();
    assert!(bundle.contains("Consultas Power Query (2)"));
    fs::remove_dir_all(&dir).ok();
}

/// Itens filtrados e diagnóstico saem juntos em um único `.zip`.
#[test]
fn zip_com_filtro_e_diagnostico() {
    use std::io::Read;

    let out = std::env::temp_dir().join(format!("xlsm_inspector_zip_{}", std::process::id()));
    fs::create_dir_all(&out).unwrap();
    let targets = ExportTargets {
        json: false,
        cells_md: false,
        vba_md: true,
        power_query: false,
        bundle: false,
        diagnostics: true,
    };
    let cfg = OutputConfig {
        zip: true,
        items: ItemFilter {
            modules: Some(vec!["modUtilitarios".into()]),
            ..Default::default()
        },
        ..OutputConfig::default()
    };
    let written = export_outputs(
        &fx("com_vba.xlsm"),
        &out,
        targets,
        None,
        &VbaOptions::default(),
        &cfg,
        &mut |_| {},
    )
    .unwrap();
    assert_eq!(written.len(), 1);
    assert!(written[0].ends_with("com_vba_exportacao.zip"));
    assert_eq!(fs::read_dir(&out).unwrap().count(), 1);

    let mut zip = zip::ZipArchive::new(fs::File::open(&written[0]).unwrap()).unwrap();
    assert_eq!(zip.len(), 2);
    let mut read = |name: &str| {
        let mut text = String::new();
        zip.by_name(name)
            .unwrap()
            .read_to_string(&mut text)
            .unwrap();
        text
    };
    let vba = read("com_vba_codigo_vba.md");
    assert!(vba.contains("## COMPONENTE: modUtilitarios"));
    assert!(!vba.contains("## COMPONENTE: clsFuncionario"));
    assert!(read("com_vba_diagnostico.md").contains("## Diagnóstico"));

    fs::remove_dir_all(&out).ok();
}
