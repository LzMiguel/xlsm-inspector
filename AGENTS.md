# AGENTS.md

Instruções para agentes que trabalham neste repositório.

## Sobre o projeto

**XLSM Inspector** é um app desktop (Tauri v2 + React + TypeScript) que lê planilhas (`.xlsx`/`.xlsm`), documentos Word (`.docx`/`.docm`), apresentações PowerPoint (`.pptx`/`.pptm`) e PDFs, e exporta:

- mapeamento de células (valor + fórmula) em JSON e Markdown;
- código VBA (módulos, classes, planilhas, `ThisWorkbook`, UserForms) em Markdown;
- arquivo único para IA, com estimativa de tokens e divisão em partes;
- conforme o `GUIDE.md`: Power Query, diagnóstico, texto de Word, PowerPoint e PDF.

Antes de qualquer mudança relevante, leia:

- `PRD.md`: o que está sendo construído e o que está fora de escopo.
- `Architecture.md`: estrutura de pastas, comandos IPC, modelo de dados.
- `Design-System.md`: tokens e componentes de interface.

Se uma tarefa conflitar com esses documentos, pare e pergunte em vez de improvisar.

## Comandos

**Comandos só devem ser executados quando forem estritamente solicitados.**
Testes e correções de erros eu mesmo irei fazer, então não execute nada sem minha autorização.

```bash
# instalar dependências
npm install

# desenvolvimento (frontend + backend com hot reload)
npm run tauri dev

# build de produção
npm run tauri build

# frontend
npm run lint
npm run typecheck
npm run test

# backend (rodar dentro de src-tauri/)
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

## Regras gerais

1. **Nunca modificar o arquivo de entrada.** Abrir planilhas só para leitura.
2. **Nunca executar macros** nem interpretar código VBA; apenas ler como texto.
3. **Sem rede.** Não adicionar dependências que façam chamadas HTTP, telemetria ou carreguem fontes/CDNs.
4. **Processamento no Rust.** O frontend não lê nem interpreta arquivos Excel.
5. **Compatibilidade de saída.** O JSON usa as chaves `formula` e `valor`; o Markdown segue o formato do `PRD.md`. Não mudar sem atualizar o PRD.
6. Mudanças pequenas e focadas. Não refatorar código não relacionado à tarefa.

## Backend (Rust)

- Código em `src-tauri/src/`.
- `commands.rs` é uma camada fina: valida entrada e delega para `excel/` e `output/`.
- `excel/` e `output/` **não importam nada do Tauri**, para serem testáveis isoladamente.
- Erros com `thiserror` no enum `AppError` (`error.rs`). Nada de `unwrap()`/`expect()` fora de testes.
- Trabalho pesado dentro de `spawn_blocking`; emitir progresso via evento `extraction://progress`.
- Usar `IndexMap` / `serde_json` com `preserve_order` para manter a ordem de abas e células.
- Conferir a API do `calamine` na versão fixada no `Cargo.toml` antes de usar (`vba_project()` mudou de assinatura entre versões).
- Toda função pura nova (ex.: conversão de coordenadas, limpeza de VBA, geração de Markdown) vem com teste unitário.

## Frontend (React)

- TypeScript estrito; sem `any`.
- Chamadas ao backend apenas via `src/api/commands.ts` (nunca `invoke` direto nos componentes).
- Tipos em `src/types/models.ts` devem espelhar `model.rs`. Ao mudar um, mude o outro no mesmo commit.
- Estilos só com tokens do design system (classes Tailwind mapeadas para variáveis CSS). Sem cores hexadecimais nos componentes.
- Componentes base em `src/components/ui/`; reutilizar antes de criar novos.
- Listas grandes (células) sempre virtualizadas.
- Textos da interface em **português do Brasil**.

## Permissões do Tauri

- Capabilities ficam em `src-tauri/capabilities/`. Adicionar apenas o mínimo necessário e justificar no PR.
- Não habilitar plugins de shell, HTTP ou acesso amplo ao sistema de arquivos.

## Testes e fixtures

- Planilhas de teste em `src-tauri/tests/fixtures/`. Devem ser arquivos sintéticos, **sem dados reais de empresa ou pessoas**.
- Cobrir: fórmulas com e sem cache, acentuação, células com quebra de linha e `|`, abas vazias, VBA de todos os tipos de componente, arquivo sem VBA, arquivo corrompido.

## Commits

- Formato: Conventional Commits (`feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`).
- Mensagens em português ou inglês, mas consistentes no repositório.

## Definição de pronto

Uma tarefa só está concluída quando:

- [ ] Tipos Rust e TypeScript estão sincronizados.
- [ ] Nenhuma nova permissão ou dependência de rede foi adicionada sem justificativa.
- [ ] Documentação (`PRD.md`, `Architecture.md`, `Design-System.md`) atualizada se o comportamento mudou.
