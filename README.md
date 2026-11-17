<div align="center">

# 🔍 XLSM Inspector

**Transforme planilhas Excel e macros VBA em arquivos legíveis, sem sair do seu computador.**

![Tauri](https://img.shields.io/badge/Tauri-v2-24C8DB?logo=tauri&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-backend-000000?logo=rust&logoColor=white)
![React](https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=black)
![TypeScript](https://img.shields.io/badge/TypeScript-estrito-3178C6?logo=typescript&logoColor=white)
![Offline](https://img.shields.io/badge/100%25-offline-2ea44f)

</div>

---

## Por que usar?

Entender uma planilha herdada, auditar fórmulas ou revisar macros normalmente exige abrir o Excel, clicar célula por célula e navegar pelo editor VBA. O **XLSM Inspector** lê o arquivo `.xlsx` / `.xlsm` e entrega tudo em texto: fácil de ler, comparar, versionar e enviar para uma IA analisar.

Ele substitui dois scripts Python (`openpyxl` + `oletools`) por um único app com interface gráfica.

## ✨ Funcionalidades

- 📊 **Mapeamento de células:** valor e fórmula de cada célula, aba por aba.
- 🧩 **Código VBA completo:** módulos, classes, formulários e código de planilhas, organizados por componente.
- 👀 **Pré-visualização:** tabelas virtualizadas (listas grandes sem travar) e VBA com realce de sintaxe.
- 🎯 **Escopo flexível:** todas as linhas, as primeiras N ou uma amostra de N linhas espaçadas por igual, ideal para planilhas enormes.
- 📝 **Modelos de saída:** personalize nomes de arquivo e o Markdown gerado.
- ⚡ **Cache e recentes:** reabrir uma planilha que não mudou é instantâneo.
- 🌗 **Tema claro e escuro.**

## 📦 O que ele gera

| Arquivo                    | Conteúdo                             |
| -------------------------- | ------------------------------------ |
| `mapeamento_planilha.json` | Células com `valor` e `formula`      |
| `mapeamento_planilha.md`   | Mesmo conteúdo em Markdown           |
| `vba_code.md`              | Código VBA organizado por componente |

## 🔒 Privacidade e segurança

- O arquivo de entrada **nunca é modificado** (somente leitura).
- Macros **nunca são executadas**; o VBA é tratado apenas como texto.
- **Sem rede, telemetria, CDN ou fontes externas.** Seus dados não saem da máquina.
- Todo o processamento de Excel roda em Rust; o frontend só exibe.

## 🧰 Tecnologias

| Camada       | Tecnologias                                                          |
| ------------ | -------------------------------------------------------------------- |
| **Backend**  | Tauri v2, Rust, `calamine`, `serde_json`, `thiserror`                |
| **Frontend** | React 19, TypeScript, Vite, Tailwind CSS 4, Zustand, TanStack Virtual, Shiki |
| **Testes**   | `cargo test` (Rust) e Vitest (frontend)                              |

## 🚀 Como rodar

### Pré-requisitos

- [Node.js](https://nodejs.org/) e npm
- [Rust](https://www.rust-lang.org/tools/install) (toolchain estável)
- [Dependências do Tauri v2](https://v2.tauri.app/start/prerequisites/) para o seu sistema (no Windows: WebView2 e Build Tools do Visual Studio)

### Comandos

```bash
npm install
npm run tauri dev      # app em modo desenvolvimento
npm run tauri build    # gera o instalador
```

### Qualidade

```bash
npm run lint                 # ESLint
npm run typecheck            # tsc --noEmit
npm test                     # Vitest
cd src-tauri && cargo test   # testes do backend
```

## 🗂️ Estrutura

```
src/             Frontend React (UI, store, api/commands.ts, types/)
src-tauri/src/   Backend Rust (commands.rs, excel/, output/, error.rs)
src-tauri/tests/ Testes e fixtures sintéticas
```

## 📚 Documentação

- [`PRD.md`](PRD.md): escopo, requisitos e formato de saída
- [`Architecture.md`](Architecture.md): pastas, modelo de dados, comandos IPC
- [`Design-System.md`](Design-System.md): tokens e componentes de UI
- [`GUIDE.md`](GUIDE.md): passo a passo de desenvolvimento
- [`AGENTS.md`](AGENTS.md) / [`CLAUDE.md`](CLAUDE.md): regras para agentes de IA

## 💡 IDE recomendada

[VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
