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
              (.xlsx/.xlsm de entrada, .json/.md de saída)
```

## 2. Stack

| Camada              | Tecnologia                                                     |
| ------------------- | -------------------------------------------------------------- |
| Shell desktop       | Tauri v2                                                       |
| Frontend            | React 18+, TypeScript, Vite                                    |
| Estilo              | Tailwind CSS + tokens do `Design-System.md`                    |
| Componentes base    | Próprios, em `components/ui/` (Tailwind + tokens; sem shadcn)  |
| Fontes              | `@fontsource-variable/inter` e `jetbrains-mono` (no bundle)    |
| Estado              | Zustand (estado global leve)                                   |
| Tabelas grandes     | `@tanstack/react-virtual`                                      |
| Destaque de código  | Shiki (motor JS, gramática `vb`, tema por variáveis CSS)       |
| Leitura Excel + VBA | `calamine`                                                     |
| Serialização        | `serde`, `serde_json` (com `preserve_order`)                   |
| Ordem estável       | `indexmap`                                                     |
| Erros               | `thiserror`                                                    |
| Plugins Tauri       | `dialog` (abrir pasta/arquivo), `opener` (revelar no Explorer) |

## 3. Estrutura de pastas

```text
xlsm-inspector/
├── src/                         # Frontend
│   ├── api/
│   │   ├── commands.ts          # wrappers tipados de invoke()
│   │   └── events.ts            # listeners de progresso
│   ├── components/
│   │   ├── ui/                  # Button, Tabs, Alert... (design system)
│   │   ├── DropZone.tsx
│   │   ├── FileCard.tsx
│   │   ├── SheetList.tsx
│   │   ├── CellTable.tsx
│   │   ├── VbaViewer.tsx
│   │   └── ExportPanel.tsx
│   ├── hooks/
│   ├── store/
│   │   └── workbookStore.ts
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
│   │   │   ├── diagnostics.rs   # erros, nomes, abas ocultas, tabelas, refs. externas (RF13)
│   │   │   ├── vba.rs           # módulos VBA
│   │   │   └── coords.rs        # (linha, coluna) → "A1"
│   │   ├── output/
│   │   │   ├── mod.rs
│   │   │   ├── json.rs
│   │   │   ├── markdown.rs      # recebe o OutputTemplate
│   │   │   ├── template.rs      # modelo do Markdown, predefinições, {placeholders}
│   │   │   ├── naming.rs        # nomes de arquivo: validação, colisão
│   │   │   ├── bundle.rs        # Markdown único para IA: resumo + células + VBA (RF14)
│   │   │   ├── tokens.rs        # estimativa de tokens e divisão em partes (RF14)
│   │   │   └── export.rs        # grava os arquivos
│   │   └── store/
│   │       ├── mod.rs
│   │       ├── settings.rs      # configurações (settings.json)
│   │       ├── recent.rs        # histórico de arquivos (recent.json)
│   │       └── cache.rs         # cache em disco das últimas 5 extrações
│   ├── tests/
│   │   └── fixtures/            # planilhas de teste
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
}

pub struct ExtractionResult {
    pub sheets: Vec<SheetCells>,
    pub vba: Vec<VbaComponent>,
    pub warnings: Vec<Warning>,
}
```

Os mesmos tipos ficam em `src/types/models.ts`. Mantê-los sincronizados manualmente ou gerar com `ts-rs` / `specta`.

Os campos `formula` e `valor` mantêm os nomes do script Python para que o JSON seja compatível com automações existentes.

## 5. Comandos IPC

| Comando                                         | Entrada                                                                                       | Saída                         | Observação                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| ----------------------------------------------- | --------------------------------------------------------------------------------------------- | ----------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `open_workbook`                                 | `path`, `reload?`, `vba_options?`                                                             | `{ summary, cached }`         | Resumo + (cache válido e mesmas opções de VBA) a última extração `{sheets, scope, result}`, sem reler o arquivo. `reload: true` ignora o cache. Atualiza os recentes; `FileNotFound` remove o item.                                                                                                                                                                                                                                                                                                                                                                                          |
| `extract_workbook`                              | `path`, `sheets?`, `scope?`, `vba_options?`                                                   | `ExtractionResult`            | Células + VBA em um passo; emite eventos de progresso. Sem `sheets` = todas. `scope`: `full` (padrão), `head` ou `sample` com `rows`; corta por linha com conteúdo, mantém cabeçalhos de tabela e gera aviso `Sampled`. Guarda no cache (se ligado). O arquivo ainda é lido inteiro.                                                                                                                                                                                                                                                                                                         |
| `get_recent` / `remove_recent` / `clear_recent` | —, `path`, —                                                                                  | `RecentFile[]`                | Histórico (até 10) em `recent.json`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `check_recent`                                  | —                                                                                             | `{path, status}[]`            | `status`: `Cached`, `Modified`, `NotCached`, `Missing`, `Locked`. Só `metadata` + JSON do cache; não abre as planilhas.                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| `clear_cache`                                   | —                                                                                             | —                             | Apaga `cache/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| `diagnose_workbook`                             | `path`                                                                                        | `Diagnostics`                 | Lê o arquivo (somente leitura) e devolve erros de fórmula, referências externas (`[n]` na fórmula), nomes definidos, abas ocultas/muito ocultas, tabelas e tabelas dinâmicas. Listas de células cortadas em 500; `error_count`/`external_count` têm o total. Independe da extração e do cache.                                                                                                                                                                                                                                                                                               |
| `extract_vba`                                   | `path`, `options?`                                                                            | `VbaComponent[]`              | `options`: `remove_blank_lines`, `hide_attributes`. Sem VBA → erro `NoVbaProject`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| `export_outputs`                                | `path`, `out_dir`, `targets?`, `sheets?`, `vba_options?`, `names?`, `template?`, `on_exists?` | `string[]` (arquivos gerados) | Re-extrai no Rust e escreve. `targets`: `{json, cells_md, vba_md, bundle}` (padrão: os três primeiros; `bundle` desligado). Sem VBA, omite `vba_code.md`. `bundle` grava o arquivo único para IA (`contexto_ia.md`); com `split_tokens` vira `contexto_ia_parte_N.md`. `names`: padrões de nome (`{arquivo}`, `{data}`); `template`: textos do Markdown; `scope`: igual ao de `extract_cells` (a exportação é independente da extração); `on_exists`: `fail` (padrão, erro `FileExists`), `overwrite` ou `suffix` (`_2`). Nomes e colisões são checados antes de extrair. O JSON nunca muda. |
| `estimate_bundle`                               | `path`, `sheets?`, `scope?`, `vba_options?`, `template?`, `split_tokens?`                     | `{ tokens, parts }`           | Tamanho **aproximado** (≈ caracteres ÷ 4) do arquivo único e nº de partes. Usa o cache; sem ele, extrai e guarda.                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `builtin_templates`                             | —                                                                                             | `TemplatePreset[]`            | Predefinições embutidas ("Padrão" = formato da v1.0, "Contexto para IA").                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| `preview_template`                              | `template`                                                                                    | `string`                      | Markdown de exemplo com dados fictícios.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |

| `get_settings` | — | `Settings` | Lê `settings.json` em `app_config_dir`. Ausente ou corrompido → padrões. |
| `save_settings` | `settings` | — | Grava via arquivo temporário + rename. Campos: `version`, `default_out_dir`, `vba_options`, `file_names`, `template`, `presets` (do usuário). |

> Argumentos do `invoke` em camelCase no JS (`outDir`, `vbaOptions`); campos dos structs em snake_case.

### Eventos

| Evento                  | Payload                                              |
| ----------------------- | ---------------------------------------------------- |
| `extraction://progress` | `{ stage: "cells" \| "vba", current, total, label }` |

### Erros

Todos os comandos retornam `Result<T, AppError>`. `AppError` é um enum serializável com `code` e `message`:

- `FileNotFound`, `FileLocked`, `UnsupportedFormat`, `CorruptedFile`, `NoVbaProject`, `WriteFailed`.

O frontend mapeia `code` para a mensagem e o componente `Alert` corretos.

## 6. Fluxos

### Abrir arquivo

```text
DropZone/diálogo ─► invoke("open_workbook", path)  ─► cache válido? devolve a última extração
                  ─► Rust abre com calamine, lê nomes das abas e checa VBA
                  ─► WorkbookSummary ─► store ─► FileCard + SheetList
```

### Extrair

```text
Botão "Extrair" ─► invoke("extract_workbook", path, abas, escopo)
                    ├─ spawn_blocking: para cada aba
                    │    worksheet_range (valores) + worksheet_formula (fórmulas)
                    │    une por posição absoluta, converte para "A1"
                    │    emit("extraction://progress")
                    └─ SheetCells[] ─► store ─► CellTable
               ─► invoke("extract_vba", path, opções)
                    └─ vba_project() ─► módulos ─► limpeza ─► VbaComponent[]
```

### Exportar

```text
Botão "Exportar" ─► diálogo de pasta ─► invoke("export_outputs")
                 ─► output::json + output::markdown ─► escreve arquivos
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
- Se o calamine falhar em algum caso, plano B: `zip` + `cfb` + descompressão MS-OVBA, isolado em `excel/vba.rs`.

### Markdown

- Escapar `|` e trocar `\n` por `|` nas células, como no script original.
- Uma seção por aba, separadas por `---`.

### Diagnóstico (RF13)

- `excel/diagnostics.rs` reaproveita `read_sheet` e `read_tables`; erros vêm de `CellValue::Error` (com ou sem fórmula). Nomes internos `_xlnm.*` são filtrados.
- Abas ocultas: `sheets_metadata()` (`Hidden`/`VeryHidden`). Tabelas dinâmicas: `pivot_tables()` do calamine; se falhar, a lista fica vazia (não derruba o diagnóstico).
- Referência externa: `[n]` solto na fórmula (não conta `Tabela[Col]`). Limite conhecido: um texto literal `"[1]"` dentro da fórmula também conta.
- Fora do cache de propósito: o resultado é recalculado ao abrir a aba.

### Saída para IA (RF14)

- `bundle.rs` monta o `.md` único com o modelo ativo: abertura (título + texto de abertura) + `## Resumo` (células/fórmulas por aba, componentes VBA) + abas + componentes VBA + rodapé. O "Contexto para IA" é uma predefinição do modelo; o arquivo único usa o modelo ativo.
- Blocos indivisíveis: abertura+resumo, cada aba, cada componente VBA, rodapé. `tokens::split_units` agrupa blocos até N tokens e só corta entre eles, então nenhuma parte quebra bloco de código ou linha de tabela, e a junção das partes é idêntica ao arquivo único. Um bloco maior que N fica sozinho na parte (passa do limite).
- Tokens são estimativa (≈ caracteres ÷ 4), sempre rotulada como tal na UI.

### Cache e recentes (RF12, RNF08)

- Dados só locais, em disco: `settings.json` e `recent.json` em `app_config_dir`; `cache/<hash>.json` em `app_data_dir`. Nenhuma rede (RNF01); a planilha nunca é escrita (RNF07).
- Entrada do cache = resumo + última extração (`abas`, `escopo`, opções de VBA, `ExtractionResult`). Vale enquanto **tamanho + data de modificação** do arquivo forem os mesmos e `cache_version` bater; JSON corrompido ou de outra versão é ignorado.
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
- **Pré-visualização:** sem limite de células; a tabela e o código são virtualizados.

## 8. Segurança

- Sem acesso à rede: não habilitar plugins HTTP; CSP restritiva em `tauri.conf.json`.
- **Capabilities** mínimas: `dialog:allow-open`, `dialog:allow-save`, `opener:allow-reveal-item-in-dir`. Sem `fs`, shell ou HTTP.
- **CSP:** `default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'`. O Vite não inlina assets (`assetsInlineLimit: 0`) para as fontes saírem como arquivos.
- O arquivo de entrada é aberto só para leitura. Macros nunca são executadas.
- Fontes e assets embutidos no bundle.

## 9. Testes

| Tipo            | Ferramenta                       | Escopo                                                                       |
| --------------- | -------------------------------- | ---------------------------------------------------------------------------- |
| Unitário Rust   | `cargo test`                     | `coords`, limpeza de VBA, geração de Markdown/JSON                           |
| Integração Rust | `cargo test` + `tests/fixtures/` | Planilhas reais: fórmulas, acentos, VBA de todos os tipos, arquivo sem cache |
| Paridade        | script de comparação             | Saída do Rust vs. saída dos scripts Python para os mesmos arquivos           |
| Frontend        | Vitest + Testing Library         | Componentes e store                                                          |
| E2E (opcional)  | WebdriverIO + tauri-driver       | Fluxo abrir → extrair → exportar                                             |

## 10. Build e distribuição

- `npm run tauri build` gera instalador `.msi`/NSIS para Windows.
- WebView2 já vem no Windows 10/11; configurar bootstrapper para máquinas sem ele.
- Versão definida em `tauri.conf.json` e `Cargo.toml` (manter iguais).
