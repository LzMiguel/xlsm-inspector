# CLAUDE.md

Contexto para o Claude (e outros agentes) neste repositório. As regras completas estão em `AGENTS.md`; este arquivo é o resumo operacional. Em caso de conflito, `AGENTS.md` e `PRD.md` prevalecem.

## Projeto

**XLSM Inspector**: app desktop (Tauri v2 + React + TypeScript) que lê planilhas (`.xlsx`/`.xlsm`), Word (`.docx`/`.docm`), PowerPoint (`.pptx`/`.pptm`) e PDF, e exporta:

- `{arquivo}_celulas.json` (células: valor + fórmula);
- `{arquivo}_celulas.md` (mesmo conteúdo em Markdown);
- `{arquivo}_codigo_vba.md` (código VBA por componente);
- `{arquivo}_contexto_ia.md` (arquivo único para IA, opcionalmente em partes);
- conforme o `GUIDE.md`: Power Query, diagnóstico, Word, PowerPoint e PDF.

Substitui dois scripts Python (`openpyxl` + `oletools`). 100% offline.

## Documentos (ler antes de mudanças relevantes)

| Arquivo            | Conteúdo                                             |
| ------------------ | ---------------------------------------------------- |
| `PRD.md`           | Escopo, requisitos RF/RNF, formato de saída, roadmap |
| `Architecture.md`  | Pastas, modelo de dados, comandos IPC, fluxos        |
| `Design-System.md` | Tokens, tipografia, componentes de UI                |
| `GUIDE.md`         | Passo a passo de desenvolvimento (ordem das etapas)  |
| `AGENTS.md`        | Regras para agentes                                  |

Se uma tarefa conflitar com esses documentos, **pare e pergunte**.

## Regra mais importante

**Não execute comandos (npm, cargo, build, testes, lint) sem pedido explícito do usuário.** O usuário roda testes e corrige erros. Limite-se a ler/criar arquivos e a sugerir o comando a rodar.

## Regras inegociáveis

1. Nunca modificar o arquivo de entrada (somente leitura).
2. Nunca executar macros; VBA é só texto.
3. Sem rede, telemetria, CDN ou fontes externas.
4. Todo processamento de arquivos (Excel, Word, PowerPoint, PDF) é no Rust; o frontend só exibe.
5. JSON usa as chaves `formula` e `valor`. Markdown segue o `PRD.md`.
6. Mudanças pequenas e focadas; sem refatorar o que não foi pedido.

## Backend (`src-tauri/src/`)

- `commands.rs` fino: valida e delega a `excel/` e `output/`.
- `excel/` e `output/` **não importam Tauri**.
- Erros via `thiserror` (`AppError` em `error.rs`); sem `unwrap()`/`expect()` fora de testes.
- Trabalho pesado em `spawn_blocking`; progresso via evento `extraction://progress`.
- `IndexMap` + `serde_json` com `preserve_order`.
- Conferir a API do `calamine` na versão do `Cargo.toml` (`vba_project()` mudou entre versões).
- Fórmulas no calamine vêm sem `=`; adicionar. Unir `worksheet_range` e `worksheet_formula` por posição **absoluta** (`range.start()` + offset).
- Toda função pura nova tem teste unitário.

## Frontend (`src/`)

- TypeScript estrito, sem `any`.
- Backend só via `src/api/commands.ts`; nunca `invoke` direto em componentes.
- `src/types/models.ts` espelha `model.rs`; alterar os dois juntos.
- Só tokens do design system (sem hex em componentes). Base em `components/ui/`.
- Listas grandes sempre virtualizadas.
- Textos da UI em **português do Brasil**.

## Tauri

- Capabilities mínimas em `src-tauri/capabilities/`; sem plugins de shell/HTTP nem FS amplo.

## Testes

- Fixtures sintéticas em `src-tauri/tests/fixtures/`, sem dados reais.
- Cobrir: fórmula com/sem cache, acentos, quebra de linha e `|`, aba vazia, todos os tipos de VBA, arquivo sem VBA, arquivo corrompido.

## Commits

Conventional Commits (`feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`), idioma consistente.

## Definição de pronto

- [ ] Tipos Rust e TS sincronizados.
- [ ] Nenhuma permissão/dependência de rede nova sem justificativa.
- [ ] `PRD.md`, `Architecture.md`, `Design-System.md` atualizados se o comportamento mudou.

## Plano de condução

O desenvolvimento segue as fases do `GUIDE.md` (backend testável primeiro, UI depois). Ao receber uma tarefa, identifique a fase/etapa correspondente, faça só ela e indique ao usuário qual comando rodar para validar.
