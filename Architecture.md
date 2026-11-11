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

| Camada              | Tecnologia                                                       |
| ------------------- | ---------------------------------------------------------------- |
| Shell desktop       | Tauri v2                                                         |
| Frontend            | React 18+, TypeScript, Vite                                      |
| Estilo              | Tailwind CSS + tokens do `Design-System.md`                      |
| Componentes base    | shadcn/ui (Radix)                                                |
| Estado              | Zustand (estado global leve)                                     |
| Tabelas grandes     | `@tanstack/react-virtual`                                        |
| Destaque de código  | Shiki (gramática VBA)                                            |
| Leitura Excel + VBA | `calamine`                                                       |
| Serialização        | `serde`, `serde_json` (com `preserve_order`)                     |
| Ordem estável       | `indexmap`                                                       |
| Erros               | `thiserror`                                                      |
| Plugins Tauri       | `dialog` (abrir/salvar), `fs` (escrever), `opener` (abrir pasta) |

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
│   │   │   ├── vba.rs           # módulos VBA
│   │   │   └── coords.rs        # (linha, coluna) → "A1"
│   │   └── output/
│   │       ├── mod.rs
│   │       ├── json.rs
│   │       └── markdown.rs
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

**Regra:** `excel/` e `output/` não conhecem o Tauri. Recebem caminhos/dados e devolvem `Result`. Isso permite testá-los com `cargo test` e reaproveitá-los numa CLI no futuro.

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
}

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

| Comando            | Entrada                      | Saída                         | Observação                                                           |
| ------------------ | ---------------------------- | ----------------------------- | -------------------------------------------------------------------- |
| `inspect_workbook` | `path`                       | `WorkbookSummary`             | Rápido; só metadados.                                                |
| `extract_cells`    | `path`, `sheets: string[]`   | `SheetCells[]`                | Emite eventos de progresso.                                          |
| `extract_vba`      | `path`, `options`            | `VbaComponent[]`              | `options`: remover linhas vazias, ocultar `Attribute`.               |
| `export_outputs`   | `path`, `out_dir`, `targets` | `string[]` (arquivos gerados) | Re-extrai no Rust e escreve; não depende do frontend reenviar dados. |

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
DropZone/diálogo ─► invoke("inspect_workbook", path)
                  ─► Rust abre com calamine, lê nomes das abas e checa VBA
                  ─► WorkbookSummary ─► store ─► FileCard + SheetList
```

### Extrair

```text
Botão "Extrair" ─► invoke("extract_cells", path, abas)
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

### Desempenho

- Extração em `tauri::async_runtime::spawn_blocking` para não travar a UI.
- A tabela do frontend é virtualizada.
- Para arquivos muito grandes, considerar enviar ao frontend apenas uma prévia (ex.: primeiras 50 mil células por aba) e exportar o conteúdo completo direto do Rust.

## 8. Segurança

- Sem acesso à rede: não habilitar plugins HTTP; CSP restritiva em `tauri.conf.json`.
- **Capabilities** mínimas: `dialog:allow-open`, `dialog:allow-save`, escrita restrita à pasta escolhida pelo usuário, `opener` apenas para pastas.
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
