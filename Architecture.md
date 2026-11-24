# Arquitetura — XLSM Inspector

## 1. Visão geral

App desktop com **Tauri v2**: o frontend roda em uma WebView (React) e o backend em Rust, no mesmo processo do executável. Toda leitura de arquivos e processamento acontecem no Rust; o React só exibe dados e dispara comandos.

```text
┌──────────────────────────────────────────────────────────┐
│                     Janela Tauri                         │
│  ┌────────────────────────────────────────────────────┐  │
│  │            Frontend (React + TypeScript)           │  │
│  │  UI  ──►  hooks  ──►  api/ (invoke + listen)       │  │
│  └───────────────────────────┬────────────────────────┘  │
│                    IPC (comandos JSON + eventos)         │
│  ┌───────────────────────────▼────────────────────────┐  │
│  │                Backend (Rust)                      │  │
│  │  commands  ──►  excel/  ──►  output/               │  │
│  │                 (calamine)   (serde_json, md)      │  │
│  └───────────────────────────┬────────────────────────┘  │
└──────────────────────────────┼───────────────────────────┘
                               ▼
                    Sistema de arquivos local
              (.xlsx/.xlsm/.docx/.docm/.pptx/.pptm/.pdf de entrada, .json/.md de saída)
```

## 2. Stack

| Camada              | Tecnologia                                                                                            |
| ------------------- | ----------------------------------------------------------------------------------------------------- |
| Shell desktop       | Tauri v2                                                                                              |
| Frontend            | React 18+, TypeScript, Vite                                                                           |
| Estilo              | Tailwind CSS + tokens do `Design-System.md`                                                           |
| Componentes base    | Próprios, em `components/ui/` (Tailwind + tokens; sem shadcn)                                         |
| Fontes              | `@fontsource-variable/inter` e `jetbrains-mono` (no bundle)                                           |
| Estado              | Zustand (estado global leve)                                                                          |
| Tabelas grandes     | `@tanstack/react-virtual`                                                                             |
| Destaque de código  | Shiki (motor JS, gramática `vb`, tema por variáveis CSS)                                              |
| Leitura Excel + VBA | `calamine`                                                                                            |
| Power Query (RF13)  | `zip`, `quick-xml`, `base64` (o `calamine` os usa por dentro, mas não os reexporta; nenhum puxa rede) |
| Serialização        | `serde`, `serde_json` (com `preserve_order`)                                                          |
| Ordem estável       | `indexmap`                                                                                            |
| Erros               | `thiserror`                                                                                           |
| Plugins Tauri       | `dialog` (abrir pasta/arquivo), `opener` (revelar no Explorer)                                        |

## 3. Estrutura de pastas

```text
xlsm-inspector/
├── src/                         # Frontend
│   ├── api/
│   │   ├── commands.ts          # wrappers tipados de invoke()
│   │   └── events.ts            # listeners de progresso
│   ├── components/
│   │   ├── ui/                  # Alert, Badge, Button, Checkbox, Dialog, EmptyState, Fields, ProgressBar, Tabs, Toast...
│   │   ├── DropZone.tsx, RecentFiles.tsx, FileCard.tsx   # tela inicial e arquivo aberto
│   │   ├── SheetList.tsx, ScopeSelector.tsx, ExtractPanel.tsx
│   │   ├── Preview.tsx          # abas: Células, VBA, Diagnóstico, Power Query, Busca
│   │   ├── CellTable.tsx, TableView.tsx
│   │   ├── VbaViewer.tsx, ComponentList.tsx (índice de procedimentos)
│   │   ├── DiagnosticsView.tsx, PowerQueryView.tsx, SearchView.tsx (+ searchAll.ts)
│   │   ├── ExportPanel.tsx      # diálogo Exportar
│   │   ├── SettingsDialog.tsx, OutputSettings.tsx, TemplateSettings.tsx
│   │   └── ErrorAlert.tsx, ThemeToggle.tsx
│   ├── hooks/                   # useFileDrop, useHotkey, useTheme
│   ├── store/
│   │   ├── workbookStore.ts, workbookActions.ts
│   │   └── settingsStore.ts, settingsActions.ts
│   ├── types/
│   │   └── models.ts            # espelha os structs do Rust
│   ├── styles/
│   │   └── tokens.css
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/                   # Backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs               # registra plugins e comandos
│   │   ├── commands.rs          # camada fina: valida e delega
│   │   ├── error.rs             # AppError serializável
│   │   ├── model.rs             # structs compartilhados
│   │   ├── excel/
│   │   │   ├── mod.rs
│   │   │   ├── workbook.rs      # abrir arquivo, resumo
│   │   │   ├── cells.rs         # valores + fórmulas
│   │   │   ├── diagnostics.rs   # erros, nomes, abas ocultas, tabelas, refs. externas (RF14)
│   │   │   ├── tables.rs        # tabelas do Excel (ListObjects)
│   │   │   ├── vba.rs           # módulos VBA
│   │   │   ├── vba_index.rs     # índice de procedimentos (RF20)
│   │   │   ├── power_query.rs   # consultas M + conexões (RF13)
│   │   │   └── coords.rs        # (linha, coluna) → "A1"
│   │   ├── output/
│   │   │   ├── mod.rs
│   │   │   ├── json.rs
│   │   │   ├── markdown.rs      # recebe o OutputTemplate
│   │   │   ├── template.rs      # modelo do Markdown, predefinições, {placeholders}
│   │   │   ├── naming.rs        # nomes de arquivo: validação, colisão
│   │   │   ├── bundle.rs        # Markdown único para IA: resumo + células + VBA (RF19)
│   │   │   ├── tokens.rs        # estimativa de tokens e divisão em partes (RF19)
│   │   │   ├── vba_files.rs     # .bas/.cls/.frm em Windows-1252 (RF20)
│   │   │   └── export.rs        # grava os arquivos
│   │   └── store/
│   │       ├── mod.rs
│   │       ├── settings.rs      # configurações (settings.json)
│   │       ├── recent.rs        # histórico de arquivos (recent.json)
│   │       └── cache.rs         # cache em disco das últimas 5 extrações
│   ├── tests/
│   │   ├── integration.rs
│   │   └── fixtures/            # planilhas sintéticas (inclui `power_query.xlsx`, a ser adicionada)
│   ├── capabilities/
│   │   └── default.json
│   ├── Cargo.toml
│   └── tauri.conf.json
├── PRD.md
├── Design-System.md
├── Architecture.md
└── AGENTS.md
```

**Regra:** `excel/`, `output/` e `store/` não conhecem o Tauri. Recebem caminhos/dados e devolvem `Result`. Isso permite testá-los com `cargo test` e reaproveitá-los numa CLI no futuro.

## 4. Modelo de dados (Rust)

```rust
// model.rs (esboço)
pub struct WorkbookSummary {
    pub path: String,
    pub file_name: String,
    pub size_bytes: u64,
    pub sheets: Vec<SheetInfo>,
    pub has_vba: bool,
}

pub struct SheetInfo {
    pub name: String,
    pub cell_count: usize,
    pub formula_count: usize,
}

pub enum CellValue {           // serializado "untagged"
    Empty, Bool(bool), Int(i64), Float(f64),
    String(String), DateTime(String), Error(String),
}

pub enum CellContent {         // serializado "untagged"
    Plain(CellValue),
    Formula { formula: String, valor: CellValue },
}

pub struct SheetCells {
    pub name: String,
    pub cells: IndexMap<String, CellContent>, // "A1" -> conteúdo
    pub tables: Vec<TableInfo>,
    pub sampled: Option<SampleInfo>,          // Some só se o escopo cortou linhas
}

pub struct SampleInfo { pub total_rows: usize, pub kept_rows: usize }
pub enum ExtractScope { Full, Head { rows: usize }, Sample { rows: usize } } // JSON: {"kind":"head","rows":100}

pub enum VbaComponentKind { Module, Class, Document, Workbook, UserForm, Unknown }

pub struct VbaComponent {
    pub name: String,
    pub kind: VbaComponentKind,
    pub code: String,
    pub line_count: usize,
    pub procedures: Vec<VbaProcedure>, // excel/vba_index.rs; linhas (1-based) relativas a `code` já limpo
}

pub struct VbaProcedure { pub name: String, pub kind: String, pub visibility: String, pub start_line: usize, pub end_line: usize }

pub struct ExtractionResult {
    pub sheets: Vec<SheetCells>,
    pub vba: Vec<VbaComponent>,
    pub warnings: Vec<Warning>,
    pub power_query: PowerQuery,       // vazio sem Power Query; entra no cache (cache_version 3)
}

pub enum WarningKind { MissingCache, Sampled, TableTruncated, PowerQueryUnreadable }
pub struct Warning { pub kind: WarningKind, pub sheet: String, pub count: usize, pub message: String }

pub struct PowerQuery { pub queries: Vec<PowerQueryItem>, pub connections: Vec<ConnectionInfo> }
pub struct PowerQueryItem { pub name: String, pub code: String }            // code = M, sem o `;` final
pub struct ConnectionInfo { pub name: String, pub description: String, pub connection: String } // só exibida, nunca testada

pub struct Diagnostics { /* errors, external_refs, names, hidden_sheets, tables, pivots + contagens (RF14) */ }
```

Os mesmos tipos ficam em `src/types/models.ts`. Mantê-los sincronizados manualmente ou gerar com `ts-rs` / `specta`.

Os campos `formula` e `valor` mantêm os nomes do script Python para que o JSON seja compatível com automações existentes.

## 5. Comandos IPC

| Comando                                         | Entrada                                                                                                                  | Saída                         | Observação                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| ----------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ | ----------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `open_workbook`                                 | `path`, `reload?`, `vba_options?`                                                                                        | `{ summary, cached }`         | Resumo + (cache válido e mesmas opções de VBA) a última extração `{sheets, scope, result}`, sem reler o arquivo. `reload: true` ignora o cache. Atualiza os recentes; `FileNotFound` remove o item.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| `extract_workbook`                              | `path`, `sheets?`, `scope?`, `vba_options?`                                                                              | `ExtractionResult`            | Células + VBA + Power Query em um passo; emite eventos de progresso. Sem `sheets` = todas. `scope`: `full` (padrão), `head` ou `sample` com `rows`; corta por linha com conteúdo, mantém cabeçalhos de tabela e gera aviso `Sampled`. Guarda no cache (se ligado). O arquivo ainda é lido inteiro. Power Query ilegível não derruba a extração: gera o aviso `PowerQueryUnreadable`. Tabela acima de 500 mil células é cortada com o aviso `TableTruncated`.                                                                                                                                                                                                                                                                                                                                                                              |
| `get_recent` / `remove_recent` / `clear_recent` | —, `path`, —                                                                                                             | `RecentFile[]`                | Histórico (até 10) em `recent.json`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| `check_recent`                                  | —                                                                                                                        | `{path, status}[]`            | `status`: `Cached`, `Modified`, `NotCached`, `Missing`, `Locked`. Só `metadata` + JSON do cache; não abre as planilhas.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `clear_cache`                                   | —                                                                                                                        | —                             | Apaga `cache/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| `diagnose_workbook`                             | `path`                                                                                                                   | `Diagnostics`                 | Lê o arquivo (somente leitura) e devolve erros de fórmula, referências externas (`[n]` na fórmula), nomes definidos, abas ocultas/muito ocultas, tabelas e tabelas dinâmicas. Listas de células cortadas em 500; `error_count`/`external_count` têm o total. Independe da extração, mas usa as células do cache quando há leitura completa válida (ver "Diagnóstico"); sem ela, lê o arquivo.                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `extract_power_query`                           | `path`                                                                                                                   | `PowerQuery`                  | Consultas M (`DataMashup`) e conexões (`xl/connections.xml`). Sem Power Query → listas vazias, sem erro. A aba usa este comando antes de haver extração; depois, usa o que veio no `ExtractionResult`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| `extract_vba`                                   | `path`, `options?`                                                                                                       | `VbaComponent[]`              | `options`: `remove_blank_lines`, `hide_attributes`. Sem VBA → erro `NoVbaProject`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `export_vba_files`                              | `path`, `out_dir`, `names?`                                                                                              | `string[]`                    | Grava `.bas`/`.cls`/`.frm` em Windows-1252 (CRLF) a partir do código bruto (com `Attribute`); nunca sobrescreve (`Nome_2.bas`). `.frm` sai sem controles e sem `.frx`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| `export_outputs`                                | `path`, `out_dir`, `targets?`, `sheets?`, `vba_options?`, `names?`, `template?`, `on_exists?`, `scope?`, `split_tokens?` | `string[]` (arquivos gerados) | Grava os alvos escolhidos, reaproveitando o cache quando abas, escopo e opções de VBA são iguais (senão re-extrai). `targets`: `{json, cells_md, vba_md, power_query, bundle}` (padrão: os quatro primeiros; `bundle` desligado). Sem VBA, omite `{arquivo}_codigo_vba.md`; sem Power Query, omite `{arquivo}_power_query.md`. `bundle` grava o arquivo único para IA (`{arquivo}_contexto_ia.md`, com resumo, células, VBA e Power Query); com `split_tokens` vira `{arquivo}_contexto_ia_parte_N.md`. `names`: padrões de nome (`{arquivo}`, `{data}`); `template`: textos do Markdown; `scope`: independente do da extração; `on_exists`: `fail` (padrão, erro `FileExists`), `overwrite` ou `suffix` (`_2`). Nomes e colisões são checados antes de extrair. O JSON nunca muda. |
| `estimate_bundle`                               | `path`, `sheets?`, `scope?`, `vba_options?`, `template?`, `split_tokens?`                                                | `{ tokens, parts }`           | Tamanho **aproximado** (≈ caracteres ÷ 4) do arquivo único e nº de partes. Usa o cache; sem ele, extrai e guarda.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `builtin_templates`                             | —                                                                                                                        | `TemplatePreset[]`            | Predefinições embutidas ("Padrão" = formato da v1.0, "Contexto para IA").                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `preview_template`                              | `template`                                                                                                               | `string`                      | Markdown de exemplo com dados fictícios.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `get_settings`                                  | —                                                                                                                        | `Settings`                    | Lê `settings.json` em `app_config_dir`. Ausente ou corrompido → padrões.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `save_settings`                                 | `settings`                                                                                                               | —                             | Grava via arquivo temporário + rename. Campos: `version`, `default_out_dir`, `vba_options`, `file_names` (inclui `power_query`), `template`, `presets` (do usuário), `use_cache`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |

> Argumentos do `invoke` em camelCase no JS (`outDir`, `vbaOptions`); campos dos structs em snake_case.

### Eventos

| Evento                  | Payload                                              |
| ----------------------- | ---------------------------------------------------- |
| `extraction://progress` | `{ stage: "cells" \| "vba", current, total, label }` |

### Erros

Todos os comandos retornam `Result<T, AppError>`. `AppError` é um enum serializável com `code` e `message`:

- `FileNotFound`, `FileLocked`, `UnsupportedFormat`, `CorruptedFile`, `NoVbaProject`, `WriteFailed`, `InvalidName`, `FileExists`, `Internal`.

O frontend mapeia `code` para a mensagem e o componente `Alert` corretos.

## 6. Fluxos

### Abrir arquivo

```text
DropZone/diálogo ─► invoke("open_workbook", path)  ─► cache válido? devolve a última extração
                  ─► Rust abre com calamine, lê nomes das abas e checa VBA
                  ─► WorkbookSummary ─► store ─► FileCard + SheetList
```

### Ler arquivo

```text
Botão "Ler arquivo" ─► invoke("extract_workbook", path, abas, escopo)
                    ├─ spawn_blocking: para cada aba
                    │    worksheet_range (valores) + worksheet_formula (fórmulas)
                    │    une por posição absoluta, converte para "A1"
                    │    emit("extraction://progress")
                    └─ SheetCells[] ─► store ─► CellTable
                    ├─ vba_project() ─► módulos ─► limpeza ─► VbaComponent[]
                    └─ power_query::extract_file ─► PowerQuery (vazio se não houver)
                    ExtractionResult { sheets, vba, warnings, power_query } ─► cache
```

### Exportar

```text
Botão "Exportar" ─► diálogo de pasta ─► invoke("export_outputs")
                 ─► resultado do cache (mesmas abas/escopo/VBA) ou re-extração
                 ─► output::json + markdown (células, VBA, Power Query) + bundle ─► escreve arquivos
                 ─► Toast "Concluído" + ação "Abrir pasta" (plugin opener)
```

## 7. Detalhes de implementação

### Valores + fórmulas

- `worksheet_range(nome)` e `worksheet_formula(nome)` podem ter **origens diferentes**. Sempre converter para posição absoluta com `range.start()` + offset antes de unir.
- Uma célula entra na saída se tiver valor não vazio **ou** fórmula não vazia (equivale ao `if cell_f.value is None and cell_v.value is None: continue`).
- Fórmulas vêm sem o `=` inicial no calamine; adicionar para manter compatibilidade com a saída do openpyxl.
- Fórmula com valor `Empty` gera um `Warning` de cache ausente.

### Coordenadas

- Coluna 0 → `A`, 25 → `Z`, 26 → `AA`. Função pura em `coords.rs`, com testes.

### VBA

- Obter o projeto com `vba_project()` do `calamine` (conferir a assinatura exata na versão usada; ela mudou entre versões).
- Listar nomes e pegar o código de cada módulo.
- Inferir o tipo pelas linhas `Attribute VB_...` e pelo nome (`ThisWorkbook`, `Sheet*`/`Planilha*`, `UserForm*`). Default: `Unknown`.
- O `calamine` resolveu a leitura do projeto; o plano B (`zip` + `cfb` + descompressão MS-OVBA, isolado em `excel/vba.rs`) fica de reserva se algum arquivo falhar.
- `vba_index.rs` indexa `Sub`/`Function`/`Property` com linhas relativas ao código já limpo; `vba_files.rs` exporta `.bas`/`.cls`/`.frm` em Windows-1252.

### Markdown

- Escapar `|` e trocar `\n` por `|` nas células, como no script original.
- Uma seção por aba, separadas por `---`.

### Diagnóstico (RF14)

- `excel/diagnostics.rs` reaproveita `read_sheet` e `read_tables`; erros vêm de `CellValue::Error` (com ou sem fórmula). Nomes internos `_xlnm.*` são filtrados.
- Planilhas ocultas: `sheets_metadata()` (`Hidden`/`VeryHidden`). Tabelas dinâmicas: `pivot_tables()` do calamine; se falhar, a lista fica vazia (não derruba o diagnóstico).
- Referência externa: `[n]` solto na fórmula (não conta `Tabela[Col]`). Limite conhecido: um texto literal `"[1]"` dentro da fórmula também conta.
- `diagnose_with(path, cached)` aceita as abas de uma extração completa em cache (`cache::cached_full_sheets`: todas as abas, escopo `Full`, opções de VBA irrelevantes) e só escaneia essas células; o arquivo continua sendo aberto para nomes, abas ocultas, tabelas e tabelas dinâmicas. Aba ausente do cache é lida do arquivo. `diagnose(path)` = sem cache.
- O resultado do diagnóstico em si não é guardado em disco; é recalculado ao abrir a aba (e guardado só na memória da sessão).

### Power Query (RF13)

- `excel/power_query.rs` abre o arquivo como ZIP (`zip`), só leitura. O `DataMashup` está em `customXml/item*.xml`, normalmente em **UTF-16 com BOM**; o texto é base64.
- Layout do `DataMashup`: 4 bytes de versão, 4 bytes (little-endian) com o tamanho do pacote e então um ZIP (OPC) com `Formulas/Section1.m`. Bytes depois do pacote (permissões, metadados) são ignorados. Truncado → `CorruptedFile`.
- Uma consulta por `shared <nome> = ...;` no início da linha; `#"Relatório Mensal"` vira o nome sem o `#""`. Limite conhecido: um `shared` no começo de linha dentro de texto multilinha vira consulta falsa.
- Conexões: `xl/connections.xml` (`name`, `description` e o `connection`/`url`/`sourceFile` do `dbPr`/`webPr`/`textPr`). A string pode ter caminho de servidor ou usuário: **só exibir, nunca testar a conexão** (RNF01).
- Entra no `ExtractionResult` (e no cache, `cache_version` 3), em `{arquivo}_power_query.md`, no resumo e nos blocos do arquivo único (uma consulta = um bloco indivisível; as conexões formam outro).
- Fixture real: `tests/fixtures/power_query.xlsx` (duas consultas, uma com espaço e acento no nome, e uma conexão). `power_query_fixture_real`, em `tests/integration.rs`, roda assim que o arquivo existir; enquanto isso, os testes unitários montam o ZIP em memória.

### Saída para IA (RF19)

- `bundle.rs` monta o `.md` único com o modelo ativo: abertura (título + texto de abertura) + `## Resumo` (células/fórmulas por aba, componentes VBA, consultas Power Query) + abas + componentes VBA + consultas Power Query + conexões + rodapé. O "Contexto para IA" é uma predefinição do modelo; o arquivo único usa o modelo ativo.
- Blocos indivisíveis: abertura+resumo, cada aba, cada componente VBA, cada consulta Power Query, conexões, rodapé. `tokens::split_units` agrupa blocos até N tokens e só corta entre eles, então nenhuma parte quebra bloco de código ou linha de tabela, e a junção das partes é idêntica ao arquivo único. Um bloco maior que N fica sozinho na parte (passa do limite).
- Tokens são estimativa (≈ caracteres ÷ 4), sempre rotulada como tal na UI.

### Cache e recentes (RF12, RNF08)

- Dados só locais, em disco: `settings.json` e `recent.json` em `app_config_dir`; `cache/<hash>.json` em `app_data_dir`. Nenhuma rede (RNF01); a planilha nunca é escrita (RNF07).
- Entrada do cache = resumo + última extração (`abas`, `escopo`, opções de VBA, `ExtractionResult` com `power_query`). Vale enquanto **tamanho + data de modificação** do arquivo forem os mesmos e `cache_version` bater; JSON corrompido ou de outra versão é ignorado.
- Impressão digital lida **antes** de abrir o arquivo; uma extração só é guardada se o arquivo ainda for o mesmo.
- LRU de 5 arquivos (o uso renova a data do JSON). Planilha acima de 50 MB não entra. `use_cache` desliga tudo; `clear_cache` apaga a pasta.
- Reabrir com cache válido restaura abas, escopo e resultado; se as opções de VBA mudaram, só o resumo vem do cache. `export_outputs` reaproveita o resultado se abas, escopo e opções forem as mesmas.
- `check_recent` usa só `metadata` e roda em segundo plano ao abrir a tela inicial. Limite: tamanho + mtime pode errar em pastas sincronizadas (OneDrive); "Recarregar" ignora o cache.

### Desempenho

- Extração em `tauri::async_runtime::spawn_blocking` para não travar a UI.
- A tabela do frontend é virtualizada.
- Para arquivos muito grandes, considerar enviar ao frontend apenas uma prévia (ex.: primeiras 50 mil células por aba) e exportar o conteúdo completo direto do Rust.

### Decisões que divergem do plano original

- **Sem plugin `fs`:** a escrita dos arquivos exportados é feita pelo Rust (`output/export.rs`), então o frontend não precisa de permissão de escrita.
- **Datas:** meia-noite vira `"2024-01-15"`; com hora, `"2024-01-15 10:30:00"`. Números inteiros guardados como `Float` saem como inteiros (como no openpyxl).
- **Aviso de cache ausente:** um `Warning` por aba, com a contagem (não um por célula).
- **VBA:** módulos em ordem alfabética (o `calamine` não preserva a ordem do projeto). Tipo inferido por `Attribute VB_Base` (UserForm = dois GUIDs), depois pelo nome.
- **Markdown:** `|` vira `\|` e quebra de linha vira `|` (comportamento do script original).
- **Limites de segurança:** entrada ZIP do Power Query até 32 MB descomprimidos (erro `CorruptedFile` acima); tabela até 500 mil células (corte com aviso). Nomes de saída: duplicatas no mesmo lote são recusadas, ou ganham sufixo `_N` com a política "sufixo". Nomes de módulos VBA são sanitizados (sem path traversal nem nomes reservados do Windows). No export direto (sem cache), erro ao ler Power Query é propagado.
- **Pré-visualização:** sem limite de células; a tabela e o código são virtualizados.

## 8. Segurança

- Sem acesso à rede: não habilitar plugins HTTP; CSP restritiva em `tauri.conf.json`.
- **Capabilities** mínimas: `dialog:allow-open`, `dialog:allow-save`, `opener:allow-reveal-item-in-dir`. Sem `fs`, shell ou HTTP.
- **CSP:** `default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'`. O Vite não inlina assets (`assetsInlineLimit: 0`) para as fontes saírem como arquivos.
- O arquivo de entrada é aberto só para leitura. Macros nunca são executadas.
- Fontes e assets embutidos no bundle.

## 9. Testes

| Tipo            | Ferramenta                       | Escopo                                                                                                               |
| --------------- | -------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| Unitário Rust   | `cargo test`                     | `coords`, limpeza de VBA, geração de Markdown/JSON                                                                   |
| Integração Rust | `cargo test` + `tests/fixtures/` | Planilhas sintéticas: fórmulas, acentos, VBA de todos os tipos, arquivo sem cache, Power Query (fixture a adicionar) |
| Frontend        | Vitest + Testing Library         | Componentes e store                                                                                                  |
| E2E (opcional)  | WebdriverIO + tauri-driver       | Fluxo abrir → extrair → exportar                                                                                     |

## 10. Build e distribuição

- `npm run tauri build` gera instalador `.msi`/NSIS para Windows.
- WebView2 já vem no Windows 10/11; configurar bootstrapper para máquinas sem ele.
- Versão definida em `tauri.conf.json` e `Cargo.toml` (manter iguais).
