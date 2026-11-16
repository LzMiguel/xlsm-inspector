# XLSM Inspector

App desktop que lê planilhas `.xlsx` / `.xlsm` e exporta seu conteúdo e código VBA em arquivos legíveis. Substitui dois scripts Python (`openpyxl` + `oletools`) por um único app, **100% offline**.

## O que gera

| Arquivo                  | Conteúdo                                   |
| ------------------------ | ------------------------------------------ |
| `mapeamento_planilha.json` | Células com `valor` e `formula`          |
| `mapeamento_planilha.md`   | Mesmo conteúdo em Markdown               |
| `vba_code.md`              | Código VBA organizado por componente     |

## Garantias

- O arquivo de entrada nunca é modificado (somente leitura).
- Macros nunca são executadas; o VBA é tratado apenas como texto.
- Sem rede, telemetria, CDN ou fontes externas.
- Todo o processamento de Excel roda no Rust; o frontend só exibe.

## Stack

- **Backend:** Tauri v2 + Rust (`calamine`, `serde_json`, `thiserror`)
- **Frontend:** React 19 + TypeScript + Vite, Tailwind CSS 4, Zustand, TanStack Virtual, Shiki
- **Testes:** `cargo test` (Rust) e Vitest (frontend)

## Pré-requisitos

- [Node.js](https://nodejs.org/) e npm
- [Rust](https://www.rust-lang.org/tools/install) (toolchain estável)
- [Dependências do Tauri v2](https://v2.tauri.app/start/prerequisites/) para o seu sistema (no Windows: WebView2 e Build Tools do Visual Studio)

## Desenvolvimento

```bash
npm install
npm run tauri dev      # app em modo desenvolvimento
npm run tauri build    # gera o instalador
```

Outros scripts:

```bash
npm run lint           # ESLint
npm run typecheck      # tsc --noEmit
npm test               # Vitest
cd src-tauri && cargo test   # testes do backend
```

## Estrutura

```
src/             Frontend React (UI, store, api/commands.ts, types/)
src-tauri/src/   Backend Rust (commands.rs, excel/, output/, error.rs)
src-tauri/tests/ Testes e fixtures sintéticas
```

## Documentação

- [`PRD.md`](PRD.md): escopo, requisitos e formato de saída
- [`Architecture.md`](Architecture.md): pastas, modelo de dados, comandos IPC
- [`Design-System.md`](Design-System.md): tokens e componentes de UI
- [`GUIDE.md`](GUIDE.md): passo a passo de desenvolvimento
- [`AGENTS.md`](AGENTS.md) / [`CLAUDE.md`](CLAUDE.md): regras para agentes de IA

## IDE recomendada

[VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
