# GUIDE — Roadmap do XLSM Inspector

Guia dos próximos passos, em fases. Estratégia: **backend Rust testável primeiro (via `cargo test`), UI depois**. Cada fase termina com um critério de pronto verificável. Só avance quando ele passar.

Referências: `PRD.md` (o quê), `Architecture.md` (como), `Design-System.md` (visual), `AGENTS.md` (regras).

## Decisões de arquitetura (valem para todas as fases)

1. **Configurações vivem no Rust** (`settings.json` em `app_config_dir`, via `get_settings`/`save_settings`), porque `export_outputs` roda no Rust e precisa delas. Esquema com `#[serde(default)]` e campo `version`. Sem plugin `fs` novo: só `std::fs`.
2. **Módulo `store/` sem Tauri** (recebe o diretório por parâmetro). Só `commands.rs` resolve caminhos com `app.path()`. Mantém a regra "`excel/` e `output/` não importam Tauri".
3. **Template padrão = formato do PRD §8, byte a byte.** Os testes de paridade continuam valendo. O JSON **nunca** é personalizável (`formula`/`valor` e a estrutura `{Aba:{A1:...}}` são fixos).
4. **O cache guarda o `ExtractionResult`**, não a saída. Chave = caminho + (tamanho, mtime) + opções (abas, escopo, VBA). Mudou o formato do resultado? Suba `CACHE_VERSION`.
5. **Amostra reduz saída, IPC e renderização, não o parsing**: o calamine ainda lê o arquivo todo. É um teto conhecido.
6. Toda mudança de comportamento atualiza `PRD.md`, `Architecture.md` e `Design-System.md`. Se algo conflitar com eles, pare e pergunte.

---

## Fase 1 — Lançamento da v1.0

Escopo da v1.0: leitura de `.xlsx`/`.xlsm` (células, fórmulas, VBA, Power Query, diagnóstico) e exportação. Os complementos de cobertura ficam na Fase 2 **por decisão**; por isso as limitações abaixo precisam estar documentadas.

1. **Fixture real do Power Query:** criar no Excel `src-tauri/tests/fixtures/power_query.xlsx` (duas consultas, uma com espaço e acento no nome, e uma conexão). O teste `power_query_fixture_real` já existe e roda sozinho quando o arquivo existir; confirmar nele o layout do `DataMashup`.
2. **Qualidade:** `npm run lint`, `npm run typecheck`, `npm run test`; `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (inclui o teste novo `abas_completas_so_com_todas_as_abas_e_escopo_full`).
3. **Testes manuais:**
   - cache depois de editar no Excel (e o botão "Recarregar");
   - diagnóstico com leitura completa em cache e com escopo parcial (esse caso não usa o cache);
   - arquivo no OneDrive, nomes com caracteres inválidos, reimportar `.bas` no VBE;
   - `.xls`, `.xlsb`, arquivo sem extensão e arquivo com senha: mensagens claras;
   - ícone na janela, na barra de tarefas e no instalador.
4. **Limitações conhecidas** no `README.md` e no `PRD.md` (seção de riscos): células mescladas não são mapeadas; comentários, notas e hiperlinks não são lidos; valores saem sem formatação (0,15 em vez de 15%); `.xls`/`.xlsb` não abrem; senha aparece como erro de arquivo inválido.
5. Revisar o escopo de `capabilities` e a CSP.
6. Conferir a versão `1.0.0` em `package.json`, `tauri.conf.json` e `Cargo.toml` (hoje alinhadas).
7. `npm run tauri build`; instalar e abrir um `.xlsm` real.
8. Atualizar `PRD.md`, `Architecture.md` (o `diagnose_workbook` usa o cache) e `Design-System.md`.
9. Commit `chore: release v1.0.0` e tag.

**Pronto quando:** os itens 2 e 3 passam e o instalador abre, lê e exporta um `.xlsm` de ponta a ponta.

---

## Fase 2 — Complementos de Excel (v1.1)

Objetivo: o mapeamento da planilha ser completo **antes** de abrir outros formatos. Ordem de prioridade; a cada item, `model.rs` e `models.ts` mudam juntos e `CACHE_VERSION` sobe.

1. **Células mescladas:** só o canto superior esquerdo guarda valor; as demais parecem vazias para quem lê o JSON. Gravar os intervalos mesclados por aba (`SheetCells.merged`); conferir `load_merged_regions` no `calamine` 0.38. Saída: Markdown indica a mesclagem; o JSON mantém a estrutura fixa (decidir onde entra o metadado sem quebrar a Decisão 3).
2. **Comentários, notas e hiperlinks:** o `calamine` não expõe. Ler `xl/comments*.xml`, `xl/threadedComments` e as relações de hiperlink com `zip` + `quick-xml` (já são dependências).
3. **Célula perdida longe do resto** (ex.: `XFD1048576`): o `Range` do `calamine` é denso e pode estourar a memória em `read_sheet` e `inspect`. Hoje só as tabelas têm limite. Criar a fixture e proteger (ler `dimension`, leitura em streaming ou tratar a falha de alocação).
4. **Fixtures e testes que faltam:** células mescladas, fórmula de matriz, data com hora e duração, aba oculta e muito oculta, aba de gráfico. Documentar a formatação de números (valor bruto x exibido) e avaliar uma opção para exportar o texto formatado.
5. **Erro de senha:** arquivo protegido é um contêiner OLE e hoje cai em `CorruptedFile`. Criar `AppError::PasswordProtected` (Rust, `ErrorAlert` e `models.ts`).
6. **`.xls` e `.xlsb`:** o `calamine` lê, mas `Book` está fixo em `Xlsx`. Avaliar um enum `Book` (`Xlsx`/`Xlsb`/`Xls`). Sem VBA em `.xls` pelo mesmo caminho; conferir.
7. **Tabelas sem linha de cabeçalho ou com linha de totais** (hoje assume 1 cabeçalho).
8. **Tabela grande por completo, sob demanda:** acima de `MAX_TABLE_CELLS` (500 mil células) o corpo é cortado com o aviso `TableTruncated`. Falta o comando IPC (ex.: `read_full_table`), o botão "Ler tabela inteira" no aviso e os tipos. Avaliar leitura em partes (streaming).
9. **Leitura em duas fases:** hoje o arquivo é lido na visão geral (`inspect`) e de novo na extração, ambas bloqueantes. Ler uma vez, ou fazer a visão geral só pelos metadados do ZIP, e emitir progresso por aba.
10. **Mapa de dependências entre fórmulas** (precedentes e dependentes de uma célula).

**Pronto quando:** fixtures sintéticas de cada item saem corretas em `cargo test`, e uma planilha com mesclagem, comentário e hiperlink aparece completa na UI e no Markdown.

---

## Fase 3 — Ajustes de UI e UX (v1.1)

1. **Exportação:** redesenhar o diálogo (`ExportPanel`): hoje concentra escopo, modelo, arquivos, nomes, arquivo único e partes. Agrupar por passos e deixar o essencial à vista.
2. **Aproveitar mais o espaço de tela:** o conteúdo está limitado a `max-w-5xl`; permitir largura total nas guias de Células, VBA e Power Query.
3. **Clicar para destacar o VBA** no editor de código (como o Excel faz): ao selecionar um procedimento no índice, rolar e destacar suas linhas.
4. **Página de Power Query:** melhorar a lista e a leitura do código M (destaque, copiar consulta, busca).
5. **Arquivos grandes:** paginação ou cache de renderização para evitar travamentos da UI (junto com a leitura em duas fases da Fase 2).
6. **Textos e avisos:** padronizar "planilha" (arquivo) x "aba" (dentro do arquivo) em toda a UI e nas mensagens do Rust.
7. Atualizar `Design-System.md` com os componentes novos.

**Pronto quando:** abrir e exportar um arquivo grande não trava a janela e o fluxo de exportação cabe em uma tela sem rolagem na configuração padrão.

---

## Fase 4 — Comparar versões (RF21, RNF11)

1. O cache guarda **uma geração anterior** (`previous`) ao detectar `Modified`.
2. `output/diff.rs` (puro): abas adicionadas/removidas, células alteradas (coordenada, antes, depois) e componentes VBA alterados (diff por linha simples; sem dependência nova).
3. Comando `diff_with_previous`; UI: aba "Mudanças"; exportação opcional em `.md`.

**Pronto quando:** testes com dois resultados sintéticos cobrem célula alterada/adicionada/removida, aba nova e VBA alterado.

---

## Fase 5 — Base multiformato e VBA em Word e PowerPoint (RF15)

Prepara o app para `.docx`, `.pptx` e `.pdf` (Fases 6-8). Hoje tudo parte de "planilha". **Só começa depois das Fases 1-3.**

1. Detecção por extensão: `SourceKind { Excel, Word, PowerPoint, Pdf }` (módulo novo `source.rs`, sem Tauri). Extensão não suportada → `UnsupportedFormat`. Diálogo, arrastar e soltar e `README` passam a listar os formatos.
2. O modelo atual (`WorkbookSummary`, `ExtractionResult`) continua para Excel. Cada formato novo ganha o **seu** resultado (`WordDocument`, `Presentation`, `PdfDocument`); não forçar tudo em `sheets`. Decidir na hora se `open_workbook` vira `open_document` com um enum (recomendado, para a tela inicial e os recentes continuarem únicos) ou se cada formato tem comando próprio.
3. Subir `CACHE_VERSION`: o cache atual é de planilha.
4. RF15: `.docm` e `.pptm`. Projeto VBA em `word/vbaProject.bin` ou `ppt/vbaProject.bin`. **Conferir se o `calamine` 0.38 lê um `vbaProject.bin` solto; se não, ler o ZIP e passar os bytes.** Saída e opções iguais às do RF04 (`{arquivo}_codigo_vba.md`).
5. UI: o seletor de arquivo e a tela inicial aceitam os formatos; as abas mudam conforme o tipo (Células só para Excel; VBA para `.xlsm`, `.docm`, `.pptm`).

**Pronto quando:** fixtures sintéticas `.docm` e `.pptm` com um módulo VBA saem em `{arquivo}_codigo_vba.md` igual ao de uma planilha; `.docx` sem VBA devolve `NoVbaProject`; extensão desconhecida devolve `UnsupportedFormat`.

---

## Fase 6 — Extração de Word (RF16)

1. Criar `word/` (sem Tauri). Ler `word/document.xml` com `quick-xml` em streaming.
2. Na ordem do corpo: títulos (`w:pStyle`; **o id do estilo muda com o idioma**, conferir em `word/styles.xml`), parágrafos, listas (`w:numPr`; numeradas x marcadores vêm de `word/numbering.xml`) e tabelas (`w:tbl` → tabela Markdown, com `|` e quebra de linha escapados como no RF03).
3. Cabeçalhos e rodapés (`word/header*.xml`, `footer*.xml`), comentários (`word/comments.xml`, com autor e o trecho entre `commentRangeStart`/`End`) e controles de conteúdo (`w:sdt`: título e valor).
4. Alterações controladas: `w:ins` conta como texto; `w:del` (`w:delText`) não. Opção "listar alterações" gera uma seção extra com autor e texto.
5. `model.rs` e `models.ts`: `WordDocument`; comando; saída `documento.md` (nome configurável) com modelo próprio.
6. UI: aba "Documento" (virtualizada se grande).

**Pronto quando:** fixture com título 1 e 2, lista numerada e com marcadores, tabela com `|`, cabeçalho, comentário, controle de conteúdo e uma inserção/exclusão sai na ordem certa e com o Markdown esperado.

---

## Fase 7 — Extração de PowerPoint (RF17)

1. `pptx/` (sem Tauri). A ordem dos slides vem de `ppt/presentation.xml` (`sldIdLst`) + `_rels`, **não** do número no nome do arquivo.
2. Por slide: título (placeholder `title`/`ctrTitle`), texto de cada forma (`a:t` por parágrafo), tabelas (`a:tbl`) e notas (`ppt/notesSlides/`). Slide oculto: `show="0"`.
3. Opcional "posição e tamanho": `a:off`/`a:ext` em EMU; 1 cm = 360000 EMU. Desligado por padrão.
4. Modelo, comando e UI (aba "Slides"), saída `apresentacao.md`.

**Pronto quando:** fixture com 3 slides fora de ordem nos arquivos internos, um oculto, uma tabela e uma nota sai na ordem da apresentação, com o oculto indicado.

---

## Fase 8 — Extração de PDF (RF18)

1. **Decidir a biblioteca antes de codar** (nova dependência, justificar): `lopdf` ou `pdf-extract`. Critérios: só Rust, sem rede, texto em UTF-8/acentos, tamanho do binário. Testar com a fixture antes de adotar.
2. Texto por página; metadados (título, autor, datas); campos de formulário AcroForm (nome e valor).
3. PDF sem texto (escaneado): aviso `NoText`, sem erro e sem OCR (OCR continua fora de escopo).
4. PDF protegido por senha: reaproveitar `AppError::PasswordProtected` da Fase 2.
5. Modelo, comando e UI (aba "PDF"); saída `pdf.md`. Listas grandes virtualizadas.

**Pronto quando:** fixture com 2 páginas de texto com acento, metadados e um campo de formulário sai correta; PDF só de imagem devolve o aviso e nenhum erro.

---

## Fase 9 — Qualidade e release (v1.2)

1. Lint, typecheck, `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.
2. Desempenho: RNF02 mantido com cache e com amostra; UI responsiva (RNF03), agora também para Word, PowerPoint e PDF.
3. Testes manuais por formato; reimportar `.bas` no VBE.
4. Alinhar a versão em `package.json`, `tauri.conf.json` e `Cargo.toml`.
5. `npm run tauri build`; testar a instalação por cima da versão anterior (mesmo `identifier`).
6. Atualizar `PRD.md`, `Architecture.md`, `Design-System.md` e `README.md`.

---

## Fase 10 — Depois

- Processamento em lote (pasta inteira).
- Exportar Word e PowerPoint em Markdown dividido por slide ou seção.
- Exportar PDF em Markdown dividido por página.
- Comparar versões de Word, PowerPoint e PDF.
- Assinatura do instalador.
- Atualização automática (opt-in). Contraria o RNF01 e exige decisão explícita e atualização do PRD.

---

## Checklist de requisitos pendentes

| Req   | Descrição                | Fase | OK  |
| ----- | ------------------------ | ---- | --- |
| —     | Lançamento da v1.0       | 1    | [ ] |
| —     | Complementos de Excel    | 2    | [ ] |
| —     | Ajustes de UI e UX       | 3    | [ ] |
| RF21  | Comparar versões         | 4    | [ ] |
| RNF11 | Comparar versões         | 4    | [ ] |
| RF15  | VBA em Word e PowerPoint | 5    | [ ] |
| RF16  | Extração de Word         | 6    | [ ] |
| RF17  | Extração de PowerPoint   | 7    | [ ] |
| RF18  | Extração de PDF          | 8    | [ ] |

---

## Armadilhas conhecidas

- **`calamine`**: `range` e `formula` podem começar em células diferentes; use posição absoluta. Confira a API da versão fixada antes de usar `defined_names`, tabelas dinâmicas, regiões mescladas e `sheets_metadata`. Fórmulas compartilhadas já vêm expandidas.
- **`Range` denso**: uma célula isolada longe do resto aloca a área inteira entre ela e o resto (Fase 2, item 3).
- **Tipos**: mudou `model.rs`? Mude `models.ts` no mesmo commit.
- **mtime em pastas sincronizadas**: pode não refletir a edição; ofereça "Recarregar". O diagnóstico via cache herda esse limite.
- **Versão do cache**: cache de uma versão antiga do app deve ser descartado (`cache_version`), nunca lido às cegas.
- **Nomes de arquivo no Windows**: nomes reservados (`CON`, `NUL`...), `/ \ : * ? " < > |` e ponto final.
- **UTF-8 x ANSI no VBE**: ao exportar `.bas`/`.cls`/`.frm`, acentos exigem Windows-1252.
- **Amostra não acelera o parsing**: só reduz saída, IPC e renderização.
- **Template padrão**: qualquer mudança no formato padrão quebra a paridade; teste byte a byte.
- **OneDrive**: pode bloquear build e arquivos de entrada.
- **Ícone em dev**: fica embutido no `.exe`; se a imagem muda e o Cargo não recompila, apague `src-tauri/target/debug/xlsm-inspector.exe`.
- **Office Open XML**: `.docx`, `.pptx` e `.xlsx` são ZIPs de XML; ler em streaming (`quick-xml`), nunca executar nada do arquivo e tratar ZIP corrompido ou com entradas gigantes (ZIP bomb) como `CorruptedFile`.
- **Estilos do Word dependem do idioma**: `Heading1` x `Título1`; resolver pelo nome do estilo em `styles.xml`.
- **Dependências novas** (biblioteca de PDF etc.): justificar cada uma (CLAUDE.md) e conferir que não puxam rede.
- **Fixtures sintéticas** para cada formato (CLAUDE.md): nada de documento real.
- **Power Query**: o `DataMashup` costuma vir em UTF-16 com BOM; a string de conexão só é exibida, nunca testada.

---

## Quando chamar um agente (intervenção mínima)

Use agente apenas para:

- investigar a API do `calamine` quando um item não for exposto (mesclagens, comentários);
- depurar diferença de paridade do template padrão;
- revisar o escopo de `capabilities` e a CSP antes do release.

Ao pedir, cite a fase e a etapa deste guia (ex.: "Fase 2, item 1"). O agente seguirá `CLAUDE.md`/`AGENTS.md` e não rodará comandos sem você pedir.

---

## Commits sugeridos (Conventional Commits)

| Marco  | Mensagem                                             |
| ------ | ---------------------------------------------------- |
| Fase 1 | `chore: release v1.0.0`                              |
| Fase 2 | `feat(excel): mesclagens, comentários e hiperlinks`  |
| Fase 3 | `feat(ui): exportação, espaço de tela e Power Query` |
| Fase 4 | `feat: comparar versões`                             |
| Fase 5 | `feat: base multiformato e VBA em Word e PowerPoint` |
| Fase 6 | `feat: extração de Word`                             |
| Fase 7 | `feat: extração de PowerPoint`                       |
| Fase 8 | `feat: extração de PDF`                              |
| Fase 9 | `chore: release v1.2.0`                              |
