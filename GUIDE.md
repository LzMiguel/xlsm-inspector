# GUIDE — Passo a passo para desenvolver o XLSM Inspector

Guia para você desenvolver o app com o mínimo de intervenção de agentes. Estratégia: **backend Rust testável primeiro (via `cargo test`), UI depois**. Cada etapa termina com um critério de pronto verificável. Só avance quando ele passar.

Referências: `PRD.md` (o quê), `Architecture.md` (como), `Design-System.md` (visual), `AGENTS.md` (regras).

---

## Fase 0 — Pré-requisitos

- [ ] Node.js 20+ (`node -v`)
- [ ] Rust stable via rustup (`rustc -V`), toolchain `x86_64-pc-windows-msvc`
- [ ] Visual Studio Build Tools com "Desktop development with C++"
- [ ] WebView2 (já vem no Windows 10/11)
- [ ] Git (`git init`, `.gitignore` com `node_modules`, `target`, `dist`)
- [ ] 2–3 planilhas de teste **sintéticas** (sem dados reais)

> Dica: o repositório está no OneDrive. Isso pode travar `node_modules` e `target` (milhares de arquivos). Exclua essas pastas da sincronização ou mova o projeto para fora do OneDrive (ex.: `C:\dev\xlsm-inspector`).

---

## Fase 1 — Scaffold do projeto

1. Na pasta do projeto (mantendo os `.md` existentes), crie o app Tauri:

   ```bash
   npm create tauri-app@latest
   ```

   Escolha: TypeScript, npm, template **React**. Se ele não aceitar pasta não vazia, gere em pasta temporária e copie os arquivos para a raiz.

2. Confirme que `npm run tauri dev` abre a janela padrão.
3. Defina em `tauri.conf.json`: `productName` "XLSM Inspector", janela mínima 960×640, título, e CSP restritiva (sem origens externas).
4. Dependências do frontend:

   ```bash
   npm i zustand @tanstack/react-virtual lucide-react shiki
   npm i @tauri-apps/plugin-dialog @tauri-apps/plugin-opener
   npm i -D tailwindcss @tailwindcss/vite vitest @testing-library/react @testing-library/jest-dom jsdom
   ```

5. Dependências do backend (`src-tauri/Cargo.toml`), conferindo a versão do `calamine`:

   ```toml
   calamine = "0.38"          # confira a API de vba_project() na versão fixada
   serde = { version = "1", features = ["derive"] }
   serde_json = { version = "1", features = ["preserve_order"] }
   indexmap = { version = "2", features = ["serde"] }
   thiserror = "1"
   tauri-plugin-dialog = "2"
   tauri-plugin-opener = "2"
   ```

6. Crie as pastas vazias conforme `Architecture.md` §3:

```text
src/
  api/  components/ui/  hooks/  store/  types/  styles/
src-tauri/
  src/excel/  src/output/
  tests/fixtures/
  capabilities/
```

**Pronto quando:** `npm run tauri dev` abre a janela e `cargo build` (em `src-tauri/`) compila.

---

## Fase 2 — Backend: fundamentos (sem Tauri)

Ordem de arquivos em `src-tauri/src/`. Cada um com testes unitários no próprio arquivo (`#[cfg(test)]`).

1. **`error.rs`**: `AppError` com `thiserror` e `Serialize` (`code` + `message`): `FileNotFound`, `FileLocked`, `UnsupportedFormat`, `CorruptedFile`, `NoVbaProject`, `WriteFailed`.
2. **`model.rs`**: structs de `Architecture.md` §4 (`WorkbookSummary`, `SheetInfo`, `CellValue`, `CellContent`, `SheetCells`, `VbaComponentKind`, `VbaComponent`, `Warning`, `ExtractionResult`). `CellValue`/`CellContent` com `#[serde(untagged)]`; `formula` e `valor` literais.
3. **`excel/coords.rs`**: `(linha, coluna) → "A1"`. Testes: 0→A, 25→Z, 26→AA, 701→ZZ, 702→AAA.
4. **`excel/workbook.rs`**: abrir arquivo (somente leitura), validar extensão, mapear erros, retornar `WorkbookSummary` (nome, tamanho, abas, `has_vba`).
5. **`excel/cells.rs`**: extração valor + fórmula.
   - Unir `range` e `formula` por posição **absoluta**.
   - Adicionar `=` às fórmulas.
   - Ignorar células vazias.
   - Gerar `Warning` para fórmula sem cache.
6. **`excel/vba.rs`**: `vba_project()`, listar módulos, limpar (remover linhas em branco / ocultar `Attribute`, configurável), classificar o tipo.
7. **`output/json.rs`**: gerar `mapeamento_planilha.json` (ordem preservada).
8. **`output/markdown.rs`**: gerar `mapeamento_planilha.md` e `vba_code.md` (escapar `|`, `\n` → `|`, abas separadas por `---`), conforme `PRD.md` §8.

### Fixtures (`src-tauri/tests/fixtures/`)

Crie no Excel, à mão, arquivos sintéticos:

| Arquivo           | Conteúdo                                                                   |
| ----------------- | -------------------------------------------------------------------------- |
| `simples.xlsx`    | 2 abas, textos, números, datas                                             |
| `formulas.xlsx`   | fórmulas salvas no Excel (com cache) e acentuação                          |
| `sem_cache.xlsx`  | fórmulas geradas por script e nunca salvas (ex.: openpyxl)                 |
| `especiais.xlsx`  | células com `\|` e quebra de linha; aba vazia                              |
| `com_vba.xlsm`    | módulo, classe, `ThisWorkbook`, planilha, UserForm; comentários com acento |
| `corrompido.xlsx` | arquivo de texto renomeado                                                 |

**Pronto quando:** `cargo test` passa e `cargo clippy -- -D warnings` está limpo.

---

## Fase 3 — Backend: integração com Tauri

1. **Testes de integração** em `src-tauri/tests/` usando as fixtures (abrir, extrair, gerar saídas). Compare com a saída dos scripts Python atuais (paridade, métrica do PRD §10).
2. **`commands.rs`**: `inspect_workbook`, `extract_cells`, `extract_vba`, `export_outputs` (ver `Architecture.md` §5). Fina: valida e delega.
   - Trabalho pesado em `spawn_blocking`.
   - Emitir `extraction://progress` com `{ stage, current, total, label }`.
   - `export_outputs` re-extrai no Rust (não recebe dados do frontend).
3. **`lib.rs`**: registrar plugins `dialog` e `opener` e os comandos.
4. **`capabilities/default.json`**: só `dialog:allow-open`, `dialog:allow-save`, `opener` para pastas. Sem shell/HTTP/FS amplo.

**Pronto quando:** `cargo test` passa e `npm run tauri dev` abre sem erro de permissão.

---

## Fase 4 — Frontend: base

Ordem:

1. **`styles/tokens.css`**: variáveis CSS dos temas claro/escuro (`Design-System.md` §1), tipografia e espaçamento. Mapear no Tailwind (`@theme`).
2. **Fontes**: coloque Inter e JetBrains Mono em `src/assets/fonts/` e declare com `@font-face` (sem CDN).
3. **`types/models.ts`**: espelho exato de `model.rs`.
4. **`api/commands.ts`** e **`api/events.ts`**: wrappers tipados de `invoke` e `listen`. Único ponto de contato com o backend.
5. **`store/workbookStore.ts`** (Zustand): arquivo, resumo, abas selecionadas, resultado, progresso, avisos, status (`idle | loading | extracting | ready | error`).
6. **`components/ui/`**: `Button`, `Tabs`, `Alert`, `ProgressBar`, `Toast`, `EmptyState`, `Checkbox`, `Badge`. Siga `Design-System.md` §5.
7. Layout em `App.tsx`: barra superior 48px, barra lateral 240px, conteúdo.

**Pronto quando:** a tela renderiza nos dois temas com um `EmptyState`, sem hex em componentes.

---

## Fase 5 — MVP (RF01, RF03, RF04, RF06)

Ordem de funcionalidades. Depois de cada uma, teste no `npm run tauri dev`.

1. **RF01 Seleção de arquivo**: `DropZone` (diálogo nativo + arrastar) → `inspect_workbook` → `FileCard` (nome, tamanho, nº de abas, badge VBA). Valide extensão e mostre `Alert` nos erros.
2. **Extração** (botão "Extrair", todas as abas): `extract_cells` + `extract_vba` com `ProgressBar` ligada ao evento.
3. **RF06 Exportação**: `ExportPanel` com escolha de pasta, checkboxes dos 3 arquivos, `export_outputs`, `Toast` "Concluído" com "Abrir pasta".
4. **Validação de paridade**: rode o app e os scripts Python nas mesmas planilhas e compare as saídas (diff). Corrija diferenças no Rust.

**Pronto quando:** fluxo abrir → extrair → exportar gera os 3 arquivos equivalentes aos scripts Python. **Marco: MVP.**

---

## Fase 6 — v1.0 (RF02, RF05, RF07, tema escuro)

1. **RF02 `SheetList`**: checkbox por aba, "Marcar/Desmarcar todas", contador de células, ícone de fórmula. Envie só as abas marcadas.
2. **RF07 Avisos**: `Alert` para fórmulas sem cache (texto sugerindo abrir/salvar/fechar no Excel), arquivo bloqueado, corrompido, sem VBA (`EmptyState`).
3. **RF05 Pré-visualização**:
   - `Tabs` Células / VBA.
   - `CellTable` virtualizada (`@tanstack/react-virtual`), busca por coordenada/fórmula/valor, cabeçalho fixo. Se o volume for grande, prévia limitada (ex.: 50 mil células por aba), com exportação completa pelo Rust.
   - `VbaViewer`: `ComponentList` agrupada por tipo + código com Shiki (gramática VBA), números de linha, botão "Copiar".
4. **Opções de VBA** na UI: remover linhas em branco, ocultar `Attribute`.
5. **Tema escuro**: segue o SO; alternância manual.
6. **Acessibilidade e atalhos**: `Ctrl+O`, `Ctrl+E`, `Ctrl+F`; foco visível; `aria-label` em ícones.

**Pronto quando:** todos os RFs do PRD até v1.0 passam no checklist abaixo. **Marco: v1.0.**

---

## Fase 7 — Qualidade e distribuição

1. `npm run lint`, `npm run typecheck`, `npm run test` (Vitest: store e componentes principais).
2. `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`.
3. Teste de desempenho: ~100 mil células em < 5 s (RNF02); UI responsiva durante o processamento (RNF03).
4. Teste manual com arquivos de OneDrive (bloqueado/placeholder) e `.xlsm` com acentos no VBA (risco de codepage, PRD §12).
5. Alinhar versão em `tauri.conf.json` e `Cargo.toml`.
6. `npm run tauri build`: gera `.msi`/NSIS. Teste a instalação numa máquina limpa, sem admin se possível.
7. Atualize `PRD.md`, `Architecture.md` e `Design-System.md` se algo mudou.

---

## Fase 8 — v1.1 e futuro (depois)

- v1.1: configurações persistentes (pasta padrão, opções de limpeza) e arquivos recentes.
- Futuro: lote de pastas, mapa de dependências de fórmulas, `.md` único para IA.

---

## Checklist de requisitos

| Req   | Descrição                        | Fase | OK  |
| ----- | -------------------------------- | ---- | --- |
| RF01  | Seleção de arquivo               | 5    | [ ] |
| RF02  | Seleção de abas                  | 6    | [ ] |
| RF03  | Extração de células              | 2, 5 | [ ] |
| RF04  | Extração de VBA                  | 2, 5 | [ ] |
| RF05  | Pré-visualização                 | 6    | [ ] |
| RF06  | Exportação                       | 5    | [ ] |
| RF07  | Avisos                           | 6    | [ ] |
| RNF01 | Sem rede (CSP, sem plugins HTTP) | 3, 7 | [ ] |
| RNF02 | 100 mil células < 5 s            | 7    | [ ] |
| RNF03 | UI não trava                     | 3, 7 | [ ] |
| RNF04 | UTF-8 e acentos                  | 2, 7 | [ ] |
| RNF07 | Entrada nunca modificada         | 2    | [ ] |

---

## Armadilhas conhecidas

- **`calamine`**: `range` e `formula` podem começar em células diferentes; use posição absoluta. `vba_project()` muda de assinatura entre versões. Se falhar, plano B em `excel/vba.rs`: `zip` + `cfb` + descompressão MS-OVBA.
- **Fórmulas**: vêm sem `=`; adicione.
- **Sem cache**: fórmula com valor vazio gera aviso, não erro.
- **Tipos**: mudou `model.rs`? Mude `models.ts` no mesmo commit.
- **OneDrive**: pode bloquear build e arquivos de entrada.
- **Windows/Tauri**: se `tauri dev` falhar na primeira compilação, é só demora do Rust (vários minutos).

---

## Quando chamar um agente (intervenção mínima)

Use agente apenas para:

- gerar a gramática VBA do Shiki ou resolver o plano B de VBA (MS-OVBA);
- depurar diferença de paridade que você não achou;
- revisar capabilities/CSP antes do release.

Ao pedir, cite a fase e a etapa deste guia (ex.: "Fase 2, passo 6"). O agente seguirá `CLAUDE.md`/`AGENTS.md` e não rodará comandos sem você pedir.

---

## Commits sugeridos (Conventional Commits)

| Marco                | Mensagem                                 |
| -------------------- | ---------------------------------------- |
| Fase 1               | `chore: scaffold tauri + react`          |
| Fase 2 (por arquivo) | `feat(excel): extração de células`       |
| Fase 3               | `feat: comandos IPC e capabilities`      |
| Fase 4               | `feat(ui): design system base`           |
| Fase 5               | `feat: fluxo MVP abrir-extrair-exportar` |
| Fase 6               | `feat: pré-visualização, avisos e tema`  |
| Fase 7               | `chore: release v1.0.0`                  |
