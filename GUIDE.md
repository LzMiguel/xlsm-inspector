# GUIDE — Roadmap v1.1+ do XLSM Inspector

Guia das próximas funcionalidades, em fases. A v1.0 está concluída (o guia anterior, Fases 0-8, ficou no histórico do git). Estratégia mantida: **backend Rust testável primeiro (via `cargo test`), UI depois**. Cada fase termina com um critério de pronto verificável. Só avance quando ele passar.

Referências: `PRD.md` (o quê), `Architecture.md` (como), `Design-System.md` (visual), `AGENTS.md` (regras).

## Decisões de arquitetura (valem para todas as fases)

1. **Configurações vivem no Rust** (`settings.json` em `app_config_dir`, via `get_settings`/`save_settings`), porque `export_outputs` roda no Rust e precisa delas. Esquema com `#[serde(default)]` e campo `version`. Sem plugin `fs` novo: só `std::fs`.
2. **Módulo `store/` novo, sem Tauri** (recebe o diretório por parâmetro). Só `commands.rs` resolve caminhos com `app.path()`. Mantém a regra "`excel/` e `output/` não importam Tauri".
3. **Template padrão = formato do PRD §8, byte a byte.** Os testes de integração de paridade continuam valendo com o template padrão. O JSON **nunca** é personalizável (`formula`/`valor` e a estrutura `{Aba:{A1:...}}` são fixos).
4. **O cache guarda o `ExtractionResult`**, não a saída: templates e nomes de arquivo mudam sem invalidar o cache. Chave = caminho + (tamanho, mtime) + opções (abas, escopo, VBA). `export_outputs` passa a reutilizar o cache válido.
5. **Amostra reduz saída, IPC e renderização, não o parsing**: o calamine ainda lê o arquivo todo. É um teto conhecido.
6. **O histórico de recentes migra do `localStorage` para o estado do Rust**, para a lista da tela inicial e os "5 recentes com cache" não divergirem.
7. Toda mudança de comportamento atualiza `PRD.md`, `Architecture.md` e `Design-System.md`. Se algo conflitar com eles, pare e pergunte.

---

## Fase 1 — Configurações persistentes (RF08)

1. `src-tauri/src/store/settings.rs`: `Settings { version, default_out_dir, file_names, vba_options, use_cache, template, ... }` com defaults; ler/gravar JSON. Arquivo ausente ou corrompido → defaults (nunca erro fatal).
   Testes: round-trip, campo desconhecido, JSON inválido.
2. `commands.rs`: `get_settings`, `save_settings`.
3. Frontend: `types/models.ts` e `api/commands.ts`; `store/settingsStore.ts`; mover `vbaOptions` e a pasta de saída padrão para as configurações.
4. UI: engrenagem na barra superior e `Ctrl+,` abrem o `SettingsDialog`. Novos componentes em `ui/`: `Dialog`, `Input`, `Textarea`, `Select`, `Switch`.

**Pronto quando:** fechar e reabrir o app mantém a pasta padrão e as opções de VBA.

---

## Fase 2 — Nome dos arquivos e modelos de saída (RF09, RF10)

1. `output/naming.rs` (puro): padrão com `{arquivo}`, `{data}`, `{aba}`; valida caracteres inválidos, nomes reservados do Windows e tamanho; extensão fixa.
   Testes: `CON`, `a/b`, vazio, `..`, nome muito longo.
2. Política de colisão: perguntar (sobrescrever / adicionar sufixo `_2`).
3. `output/template.rs`: `OutputTemplate { title, intro, sheet_heading, cell_header, formula_label, value_label, vba_heading, footer }` com placeholders `{arquivo} {aba} {data} {componente} {tabela}`. `markdown.rs` passa a receber o template.
4. Predefinições: "Padrão" (= PRD §8), "Contexto para IA" (com prompt inicial) e as do usuário (criar, duplicar, excluir, restaurar o padrão).
5. UI: aba "Saída" no `SettingsDialog`, com campos de nome e editor de template com **pré-visualização ao vivo** (o Rust renderiza com dados fictícios).
6. `ExportPanel`: campo de nome editável por arquivo.

**Pronto quando:** `cargo test` prova que o template padrão gera saída idêntica à da v1.0 e que um template customizado altera só o Markdown (JSON idêntico).

---

## Fase 3 — Amostra e head (RF11)

1. `model.rs`: `ExtractScope { Full, Head { rows }, Sample { rows } }`; `SheetCells` ganha `sampled: Option<SampleInfo { total_rows, kept_rows }>`; novo `WarningKind::Sampled`.
2. `excel/cells.rs` aplica o escopo por linha:
   - **Head:** primeiras N linhas de cada aba.
   - **Amostra:** N linhas espaçadas uniformemente (determinística), sempre incluindo a primeira.
   - **Tabelas (ListObjects):** cabeçalho mantido, amostra só do corpo.
   - O VBA não é amostrado.
3. `commands.rs`: `extract_cells` e `export_outputs` recebem `scope`. Extração e exportação são independentes; o padrão da exportação é o escopo da extração.
4. Saída: o Markdown ganha o aviso "Amostra: 100 de 52.000 linhas". O JSON não muda (formato fixo), então o aviso fica na UI e no nome sugerido (`_amostra`).
5. UI: `ScopeSelector` (Tudo / Primeiras N / Amostra de N) no `ExtractPanel` e no `ExportPanel`; selo "amostra" no `CellTable`.

**Pronto quando:** testes provam determinismo, inclusão da primeira linha, N maior que o total e aba vazia; a exportação em amostra gera o aviso no `.md`.

> Teto conhecido: o parsing continua lendo o arquivo inteiro.

---

## Fase 4 — Cache dos 5 recentes e verificação em segundo plano (RF12, RNF08)

1. `store/cache.rs`: um arquivo por planilha em `app_data_dir/cache/`.
   - Impressão digital = tamanho + mtime.
   - A chave inclui as opções e `cache_version`.
   - LRU de 5 itens.
   - Arquivos acima de um limite (ex.: 50 MB) não entram no cache, com aviso.
   - `use_cache` desligável e `clear_cache`.
2. Migrar os recentes para `store/recent.rs` (estado do Rust); o `localStorage` deixa de ser a fonte.
3. `commands.rs`: `check_recent(paths) -> [{ path, status: Unchanged | Modified | Missing | Locked }]` em `spawn_blocking`, **só com `metadata`** (não abre a planilha). Disparado ao montar a tela inicial, sem bloquear a UI.
4. `open_workbook`: se `Unchanged` e o cache for válido, devolve o resultado do cache; senão re-extrai e atualiza o cache. `export_outputs` reutiliza o cache válido.
5. UI: `RecentFiles` com selo "em cache" / "modificado" / "não encontrado"; botão "Recarregar" (ignora o cache); "Limpar cache" nas configurações.
6. Documentar em `Architecture.md` e `PRD.md`: dados em disco, só local, entrada nunca modificada (RNF07), nenhuma rede (RNF01).

**Pronto quando:** abrir um recente sem alteração não relê o arquivo; alterar o arquivo no Excel e reabrir força a releitura. Testes cobrem hit, miss por mtime, miss por opções, LRU e cache corrompido (ignorado).

> Limite: tamanho + mtime pode falhar em pastas sincronizadas (OneDrive). O botão "Recarregar" é a válvula.

---

## Fase 5 — Diagnóstico da planilha (RF13)

1. `excel/diagnostics.rs`: erros de fórmula por aba (`#REF!`, `#DIV/0!`...), nomes definidos, abas ocultas, tabelas, tabelas dinâmicas e referências a arquivos externos (fórmulas com `[n]`).
   **Antes de prometer cada item, conferir na API do calamine 0.38 o que é exposto** (`defined_names`, `sheets_metadata`, tabelas dinâmicas).
2. `model.rs` e `models.ts`: `Diagnostics`; comando `diagnose_workbook`.
3. UI: aba "Diagnóstico" (cartões de contagem + lista; clicar num erro leva à célula).

**Pronto quando:** uma fixture sintética com `#DIV/0!`, nome definido e aba oculta aparece corretamente no diagnóstico.

---

## Fase 6 — Saída otimizada para IA (RF14)

1. `output/bundle.rs`: `.md` único combinado (resumo + células + VBA), usando o template "Contexto para IA".
2. `output/tokens.rs`: estimativa **aproximada** (≈ caracteres / 4, rotulada como estimativa). Divisão em partes de N tokens cortando só em limites de seção (aba, tabela, componente VBA), nunca no meio de bloco de código ou de linha de tabela.
3. UI: estimativa de tokens no `ExportPanel`; opções "Arquivo único" e "Dividir em partes de N tokens".

**Pronto quando:** testes provam que nenhuma parte quebra bloco de código ou tabela e que a junção das partes reproduz o arquivo único.

---

## Fase 7 — VBA avançado (RF15)

1. `excel/vba_index.rs`: índice de procedimentos (`Sub`, `Function`, `Property Get/Let/Set`; `Private/Public/Static/Friend`) com linha inicial e final.
   Testes de borda: `End Sub` em comentário, `Declare`, procedimento de uma linha.
2. Busca global (células + VBA) na UI, sobre o resultado já em memória (filtro de exibição, como a busca atual).
3. Exportar componentes como `.bas`/`.cls`/`.frm`.
   - **Decisão a confirmar na hora:** o VBE importa em ANSI (Windows-1252); UTF-8 com acentos vira texto quebrado ao reimportar.
   - O `.frm` sai sem o `.frx` (layout de UserForm está fora de escopo, PRD §5).
4. UI: índice de procedimentos na `ComponentList`; botão "Exportar como arquivos".

**Pronto quando:** o índice está correto na fixture `com_vba.xlsm` e os arquivos exportados reimportam no VBE sem perder acentos.

---

## Fase 8 — Comparar versões (RF16)

1. O cache guarda **uma geração anterior** (`previous`) ao detectar `Modified`.
2. `output/diff.rs` (puro): abas adicionadas/removidas, células alteradas (coordenada, antes, depois) e componentes VBA alterados (diff por linha simples; sem dependência nova).
3. Comando `diff_with_previous`; UI: aba "Mudanças"; exportação opcional em `.md`.

**Pronto quando:** testes com dois resultados sintéticos cobrem célula alterada/adicionada/removida, aba nova e VBA alterado.

---

## Fase 9 — Qualidade e release (v1.1)

1. `npm run lint`, `npm run typecheck`, `npm run test`.
2. `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.
3. Desempenho: RNF02 mantido com cache e com amostra; UI responsiva (RNF03).
4. Testes manuais: cache depois de editar no Excel, arquivo no OneDrive, caracteres inválidos em nomes, reimportar `.bas` no VBE.
5. Alinhar a versão em `package.json`, `tauri.conf.json` e `Cargo.toml`.
6. `npm run tauri build`; testar a instalação por cima da v1.0 (mesmo `identifier`).
7. Atualizar `PRD.md`, `Architecture.md` e `Design-System.md`.

---

## Fase 10 — Depois

- Processamento em lote (pasta inteira).
- Mapa de dependências entre fórmulas.
- Atualização automática (opt-in). Contraria o RNF01 e exige decisão explícita e atualização do PRD.
- Assinatura do instalador.

---

## Checklist de requisitos

| Req   | Descrição                          | Fase | OK  |
| ----- | ---------------------------------- | ---- | --- |
| RF08  | Configurações persistentes         | 1    | [ ] |
| RF09  | Nome dos arquivos de saída         | 2    | [ ] |
| RF10  | Modelos de saída (Markdown)        | 2    | [ ] |
| RF11  | Amostra / head (leitura e exportação) | 3 | [ ] |
| RF12  | Cache e verificação dos recentes   | 4    | [ ] |
| RF13  | Diagnóstico da planilha            | 5    | [ ] |
| RF14  | Saída otimizada para IA            | 6    | [ ] |
| RF15  | VBA avançado                       | 7    | [ ] |
| RF16  | Comparar versões                   | 8    | [ ] |
| RNF08 | Cache local e desligável           | 4    | [ ] |

---

## Armadilhas conhecidas

- **`calamine`**: `range` e `formula` podem começar em células diferentes; use posição absoluta. Confira a API da versão fixada antes de usar `defined_names`, tabelas dinâmicas e `sheets_metadata`.
- **Tipos**: mudou `model.rs`? Mude `models.ts` no mesmo commit.
- **mtime em pastas sincronizadas**: pode não refletir a edição; ofereça "Recarregar".
- **Versão do cache**: cache de uma versão antiga do app deve ser descartado (`cache_version`), nunca lido às cegas.
- **Nomes de arquivo no Windows**: nomes reservados (`CON`, `NUL`...), `/ \ : * ? " < > |` e ponto final.
- **UTF-8 x ANSI no VBE**: ao exportar `.bas`/`.cls`/`.frm`, acentos exigem Windows-1252.
- **Amostra não acelera o parsing**: só reduz saída, IPC e renderização.
- **Template padrão**: qualquer mudança no formato padrão quebra a paridade com a v1.0; teste byte a byte.
- **OneDrive**: pode bloquear build e arquivos de entrada.

---

## Quando chamar um agente (intervenção mínima)

Use agente apenas para:

- investigar a API do `calamine` quando um item do diagnóstico não for exposto;
- depurar diferença de paridade do template padrão;
- revisar o escopo de `capabilities` e a CSP antes do release.

Ao pedir, cite a fase e a etapa deste guia (ex.: "Fase 4, passo 3"). O agente seguirá `CLAUDE.md`/`AGENTS.md` e não rodará comandos sem você pedir.

---

## Commits sugeridos (Conventional Commits)

| Marco  | Mensagem                                      |
| ------ | --------------------------------------------- |
| Fase 1 | `feat: configurações persistentes`            |
| Fase 2 | `feat: nomes de arquivo e modelos de saída`   |
| Fase 3 | `feat: amostra e head na extração e exportação` |
| Fase 4 | `feat: cache dos recentes`                    |
| Fase 5 | `feat: diagnóstico da planilha`               |
| Fase 6 | `feat: saída otimizada para IA`               |
| Fase 7 | `feat(vba): índice e exportação de componentes` |
| Fase 8 | `feat: comparar versões`                      |
| Fase 9 | `chore: release v1.1.0`                       |
