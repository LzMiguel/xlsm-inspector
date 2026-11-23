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
3. `commands.rs`: `check_recent() -> [{ path, status: Cached | NotCached | Modified | Missing | Locked }]` em `spawn_blocking`, **só com `metadata`** (não abre a planilha). Disparado ao montar a tela inicial, sem bloquear a UI.
4. `open_workbook`: se `Unchanged` e o cache for válido, devolve o resultado do cache; senão re-extrai e atualiza o cache. `export_outputs` reutiliza o cache válido.
5. UI: `RecentFiles` com selo "em cache" / "modificado" / "não encontrado"; botão "Recarregar" (ignora o cache); "Limpar cache" nas configurações.
6. Documentar em `Architecture.md` e `PRD.md`: dados em disco, só local, entrada nunca modificada (RNF07), nenhuma rede (RNF01).

**Pronto quando:** abrir um recente sem alteração não relê o arquivo; alterar o arquivo no Excel e reabrir força a releitura. Testes cobrem hit, miss por mtime, miss por opções, LRU e cache corrompido (ignorado).

> Limite: tamanho + mtime pode falhar em pastas sincronizadas (OneDrive). O botão "Recarregar" é a válvula.

---

## Fase 5 — Diagnóstico da planilha (RF14)

1. `excel/diagnostics.rs`: erros de fórmula por aba (`#REF!`, `#DIV/0!`...), nomes definidos, abas ocultas, tabelas, tabelas dinâmicas e referências a arquivos externos (fórmulas com `[n]`).
   **Antes de prometer cada item, conferir na API do calamine 0.38 o que é exposto** (`defined_names`, `sheets_metadata`, tabelas dinâmicas).
2. `model.rs` e `models.ts`: `Diagnostics`; comando `diagnose_workbook`.
3. UI: aba "Diagnóstico" (cartões de contagem + lista; clicar num erro leva à célula).

**Pronto quando:** uma fixture sintética com `#DIV/0!`, nome definido e aba oculta aparece corretamente no diagnóstico.

---

## Fase 6 — Saída otimizada para IA (RF19)

1. `output/bundle.rs`: `.md` único combinado (resumo + células + VBA), usando o modelo escolhido na exportação (o "Contexto para IA" é uma predefinição).
2. `output/tokens.rs`: estimativa **aproximada** (≈ caracteres / 4, rotulada como estimativa). Divisão em partes de N tokens cortando só em limites de seção (aba, tabela, componente VBA), nunca no meio de bloco de código ou de linha de tabela.
3. UI: o botão "Exportar…" abre um dialog com as opções (escopo, modelo do Markdown, arquivos, nomes); estimativa de tokens; opções "Arquivo único" e "Dividir em partes de N tokens". Configurações ganhou a aba "Modelos" (separada de "Saída").

**Pronto quando:** testes provam que nenhuma parte quebra bloco de código ou tabela e que a junção das partes reproduz o arquivo único.

---

## Fase 7 — VBA avançado (RF20)

1. `excel/vba_index.rs`: índice de procedimentos (`Sub`, `Function`, `Property Get/Let/Set`; `Private/Public/Static/Friend`) com linha inicial e final.
   Testes de borda: `End Sub` em comentário, `Declare`, procedimento de uma linha.
2. Busca global (células + VBA) na UI, sobre o resultado já em memória (filtro de exibição, como a busca atual).
3. Exportar componentes como `.bas`/`.cls`/`.frm`.
   - **Decisão a confirmar na hora:** o VBE importa em ANSI (Windows-1252); UTF-8 com acentos vira texto quebrado ao reimportar.
   - O `.frm` sai sem o `.frx` (layout de UserForm está fora de escopo, PRD §5).
4. UI: índice de procedimentos na `ComponentList`; botão "Exportar como arquivos".

**Pronto quando:** o índice está correto na fixture `com_vba.xlsm` e os arquivos exportados reimportam no VBE sem perder acentos.

---

## Fase 8 — Comparar versões (RF21)

1. O cache guarda **uma geração anterior** (`previous`) ao detectar `Modified`.
2. `output/diff.rs` (puro): abas adicionadas/removidas, células alteradas (coordenada, antes, depois) e componentes VBA alterados (diff por linha simples; sem dependência nova).
3. Comando `diff_with_previous`; UI: aba "Mudanças"; exportação opcional em `.md`.

**Pronto quando:** testes com dois resultados sintéticos cobrem célula alterada/adicionada/removida, aba nova e VBA alterado.

---

## Fase 9 — Power Query (RF13)

1. `excel/power_query.rs`: abre o `.xlsx`/`.xlsm` como ZIP e procura em `customXml/item*.xml` o elemento `DataMashup` (conteúdo em base64).
   - Formato do `DataMashup`: 4 bytes de versão, 4 bytes (little-endian) com o tamanho do pacote, e então um ZIP (OPC) com `Formulas/Section1.m`. **Confirmar o layout com uma fixture antes de implementar.**
   - O código M traz uma consulta por `shared <nome> = ...;`. Separar por consulta e preservar nomes como `#"Relatório Mensal"`.
2. Conexões: ler `xl/connections.xml` (nome, descrição e string de conexão). **A string de conexão pode ter caminho de servidor ou usuário: só exibir, nunca testar a conexão (RNF01).**
3. Dependências novas (`zip`, `quick-xml`): o `calamine` as usa por dentro, mas não as reexporta. Justificar no PR; sem plugin de rede.
4. `model.rs` e `models.ts`: `PowerQuery { queries: [{ name, code }], connections: [...] }`; comando `extract_power_query`; sem consultas → lista vazia (não é erro).
5. UI: aba "Power Query" (lista de consultas + código `m` com destaque simples). Saída: `{arquivo}_power_query.md` como novo alvo da exportação, com nome configurável (RF09).
6. O Power Query entra no `ExtractionResult` (portanto no cache, `cache_version` 3), no resumo e nos blocos do arquivo único (RF19). Sem consultas nem conexões, o `{arquivo}_power_query.md` é omitido.

**Pronto quando:** uma fixture sintética com duas consultas (uma com nome com espaço e acento) e uma conexão sai separada e correta; arquivo sem consultas mostra o estado vazio.

**Status:** feito (backend, aba, exportação, cache e arquivo único). **Pendente:** a fixture real `src-tauri/tests/fixtures/power_query.xlsx`, criada no Excel (duas consultas, uma com espaço e acento no nome, e uma conexão). O teste `power_query_fixture_real` já está pronto e roda sozinho quando o arquivo existir; confirmar nele o layout do `DataMashup` (passo 1).

---

## Fase 10 — Base multiformato e VBA em Word e PowerPoint (RF15)

Esta fase prepara o app para receber `.docx`, `.pptx` e `.pdf` (Fases 11-13). Hoje tudo parte de "planilha".

1. Detecção por extensão: `SourceKind { Excel, Word, PowerPoint, Pdf }` (em `excel/` ou módulo novo `source.rs`, sem Tauri). Extensão não suportada → `UnsupportedFormat`. Diálogo, arrastar e soltar e `README` passam a listar os formatos.
2. O modelo atual (`WorkbookSummary`, `ExtractionResult`) continua para Excel. Cada formato novo ganha o **seu** resultado (`WordDocument`, `Presentation`, `PdfDocument`); não forçar tudo em `sheets`. Decidir na hora se `open_workbook` vira `open_document` com um enum ou se cada formato tem comando próprio (recomendado: enum, para a tela inicial e os recentes continuarem únicos).
3. Subir `cache_version`: o cache atual é de planilha.
4. RF15: `.docm` e `.pptm`. Projeto VBA em `word/vbaProject.bin` ou `ppt/vbaProject.bin`. **Conferir se o `calamine` 0.38 lê um `vbaProject.bin` solto (`VbaProject` a partir do CFB); se não, ler o ZIP e passar os bytes.** Saída e opções iguais às do RF04 (`{arquivo}_codigo_vba.md`).
5. UI: o seletor de arquivo e a tela inicial aceitam os formatos; as abas mudam conforme o tipo (Células só para Excel; VBA para `.xlsm`, `.docm`, `.pptm`).

**Pronto quando:** fixtures sintéticas `.docm` e `.pptm` com um módulo VBA saem em `{arquivo}_codigo_vba.md` igual ao de uma planilha; `.docx` sem VBA devolve `NoVbaProject`; extensão desconhecida devolve `UnsupportedFormat`.

---

## Fase 11 — Extração de Word (RF16)

1. `excel/` não é lugar disso: criar `word/` (sem Tauri). Ler `word/document.xml` com `quick-xml` em streaming (arquivo grande não vai inteiro para a memória).
2. Na ordem do corpo: títulos (`w:pStyle` = `Heading1`/`Título1`...; **o id do estilo muda com o idioma**, conferir em `word/styles.xml` o nome de cada estilo), parágrafos, listas (`w:numPr`; numeradas x marcadores vêm de `word/numbering.xml`) e tabelas (`w:tbl` → tabela Markdown, com `|` e quebra de linha escapados como no RF03).
3. Cabeçalhos e rodapés (`word/header*.xml`, `footer*.xml`), comentários (`word/comments.xml`, com autor e o trecho entre `commentRangeStart`/`End`) e controles de conteúdo (`w:sdt`: título e valor).
4. Alterações controladas: `w:ins` conta como texto; `w:del` (`w:delText`) não. Opção "listar alterações" gera uma seção extra com autor e texto.
5. `model.rs` e `models.ts`: `WordDocument`; comando; saída `documento.md` (nome configurável) usando um modelo próprio (a predefinição de Excel não serve).
6. UI: aba "Documento" (virtualizada se grande).

**Pronto quando:** fixture sintética com título 1 e 2, lista numerada e com marcadores, tabela com `|`, cabeçalho, comentário, controle de conteúdo e uma inserção/exclusão sai na ordem certa e com o Markdown esperado.

---

## Fase 12 — Extração de PowerPoint (RF17)

1. `pptx/` (sem Tauri). A ordem dos slides vem de `ppt/presentation.xml` (`sldIdLst`) + `_rels`, **não** do número no nome do arquivo.
2. Por slide: título (placeholder `title`/`ctrTitle`), texto de cada forma (`a:t` por parágrafo), tabelas (`a:tbl`) e notas (`ppt/notesSlides/`). Slide oculto: atributo `show="0"`.
3. Opcional "posição e tamanho": `a:off`/`a:ext` em EMU; 1 cm = 360000 EMU. Desligado por padrão (polui a saída).
4. Modelo, comando e UI (aba "Slides"), saída `apresentacao.md`.

**Pronto quando:** fixture com 3 slides fora de ordem nos arquivos internos, um oculto, uma tabela e uma nota sai na ordem da apresentação, com o oculto indicado.

---

## Fase 13 — Extração de PDF (RF18)

1. **Decidir a biblioteca antes de codar** (nova dependência, justificar): `lopdf` (baixo nível, só Rust) ou `pdf-extract`. Critérios: só Rust, sem rede, texto em UTF-8/acentos, tamanho do binário. Testar com a fixture antes de adotar.
2. Texto por página; metadados (título, autor, datas); campos de formulário AcroForm (nome e valor).
3. PDF sem texto (escaneado): aviso `NoText`, sem erro e sem OCR (OCR continua fora de escopo).
4. PDF protegido por senha → erro claro (`CorruptedFile` ou um código novo; manter `models.ts` em sincronia).
5. Modelo, comando e UI (aba "PDF"); saída `pdf.md`. Listas grandes virtualizadas.

**Pronto quando:** fixture com 2 páginas de texto com acento, metadados e um campo de formulário sai correta; PDF só de imagem devolve o aviso e nenhum erro.

---

## Fase 14 — Qualidade e release (v1.1)

1. `npm run lint`, `npm run typecheck`, `npm run test`.
2. `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.
3. Desempenho: RNF02 mantido com cache e com amostra; UI responsiva (RNF03).
4. Testes manuais: cache depois de editar no Excel, arquivo no OneDrive, caracteres inválidos em nomes, reimportar `.bas` no VBE.
5. Alinhar a versão em `package.json`, `tauri.conf.json` e `Cargo.toml`.
6. `npm run tauri build`; testar a instalação por cima da v1.0 (mesmo `identifier`).
7. Atualizar `PRD.md`, `Architecture.md` e `Design-System.md`.

---

## Fase 15 — Depois

- Processamento em lote (pasta inteira).
- Mapa de dependências entre fórmulas.
- Atualização automática (opt-in). Contraria o RNF01 e exige decisão explícita e atualização do PRD.
- Assinatura do instalador.

---

## Checklist de requisitos

| Req   | Descrição                             | Fase | OK  |
| ----- | ------------------------------------- | ---- | --- |
| RF08  | Configurações persistentes            | 1    | [x] |
| RF09  | Nome dos arquivos de saída            | 2    | [x] |
| RF10  | Modelos de saída (Markdown)           | 2    | [x] |
| RF11  | Amostra / head (leitura e exportação) | 3    | [x] |
| RF12  | Cache e verificação dos recentes      | 4    | [x] |
| RF14  | Diagnóstico da planilha               | 5    | [x] |
| RF19  | Saída otimizada para IA               | 6    | [x] |
| RF20  | VBA avançado                          | 7    | [x] |
| RF21  | Comparar versões                      | 8    | [ ] |
| RF13  | Extração de Power Query               | 9    | [x] |
| RF15  | VBA em Word e PowerPoint              | 10   | [ ] |
| RF16  | Extração de Word                      | 11   | [ ] |
| RF17  | Extração de PowerPoint                | 12   | [ ] |
| RF18  | Extração de PDF                       | 13   | [ ] |
| RNF08 | Cache local e desligável              | 4    | [x] |
| RNF09 | Exportação para IA                    | 6    | [x] |
| RNF10 | VBA avançado                          | 7    | [x] |
| RNF11 | Comparar versões                      | 8    | [ ] |

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
- **Office Open XML**: `.docx`, `.pptx` e `.xlsx` são ZIPs de XML; ler em streaming (`quick-xml`), nunca executar nada do arquivo e tratar ZIP corrompido ou com entradas gigantes (ZIP bomb) como `CorruptedFile`.
- **Estilos do Word dependem do idioma**: `Heading1` x `Título1`; resolver pelo nome do estilo em `styles.xml`.
- **Dependências novas** (`zip`, `quick-xml`, biblioteca de PDF): justificar cada uma (CLAUDE.md) e conferir que não puxam rede.
- **Fixtures sintéticas** para cada formato (CLAUDE.md): nada de documento real.
- **Power Query**: o `DataMashup` costuma vir em UTF-16 com BOM; a string de conexão só é exibida, nunca testada.

---

## Quando chamar um agente (intervenção mínima)

Use agente apenas para:

- investigar a API do `calamine` quando um item do diagnóstico não for exposto;
- depurar diferença de paridade do template padrão;
- revisar o escopo de `capabilities` e a CSP antes do release.

Ao pedir, cite a fase e a etapa deste guia (ex.: "Fase 4, passo 3"). O agente seguirá `CLAUDE.md`/`AGENTS.md` e não rodará comandos sem você pedir.

---

## Commits sugeridos (Conventional Commits)

| Marco   | Mensagem                                             |
| ------- | ---------------------------------------------------- |
| Fase 1  | `feat: configurações persistentes`                   |
| Fase 2  | `feat: nomes de arquivo e modelos de saída`          |
| Fase 3  | `feat: amostra e head na extração e exportação`      |
| Fase 4  | `feat: cache dos recentes`                           |
| Fase 5  | `feat: diagnóstico da planilha`                      |
| Fase 6  | `feat: saída otimizada para IA`                      |
| Fase 7  | `feat(vba): índice e exportação de componentes`      |
| Fase 8  | `feat: comparar versões`                             |
| Fase 9  | `feat: extração de Power Query`                      |
| Fase 10 | `feat: base multiformato e VBA em Word e PowerPoint` |
| Fase 11 | `feat: extração de Word`                             |
| Fase 12 | `feat: extração de PowerPoint`                       |
| Fase 13 | `feat: extração de PDF`                              |
| Fase 14 | `chore: release v1.1.0`                              |
