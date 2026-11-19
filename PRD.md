# PRD — XLSM Inspector

## 1. Visão geral

O **XLSM Inspector** é um aplicativo desktop (Tauri + React) que lê diversos arquivos e os transforma em texto estruturado:

- **Excel** (`.xlsx` / `.xlsm`): células (valores e fórmulas), código VBA, consultas Power Query e diagnóstico.
- **Word** (`.docx` / `.docm`): títulos, parágrafos, listas, tabelas, comentários e campos.
- **PowerPoint** (`.pptx` / `.pptm`): texto, tabelas e notas de cada slide.
- **PDF** com texto selecionável: texto por página, metadados e campos de formulário.
- **VBA** também em `.docm` e `.pptm`.

A saída é **Markdown**, para leitura humana e para servir de contexto a agentes de IA e **JSON** para as células das planilhas, para automações e scripts.

O projeto nasceu como a substituição de dois scripts Python (`openpyxl` + `oletools`) por uma ferramenta única, com interface, sem depender de Python instalado na máquina.

## 2. Problema

- Entender uma planilha complexa (muitas abas, fórmulas encadeadas, macros) exige abrir célula por célula e o editor VBA.
- Agentes de IA não leem `.xlsm`, `.docx`, `.pptx` nem PDF diretamente; precisam de uma representação em texto, de tamanho controlado.
- Documentos do Office e PDFs que acompanham a planilha (especificações, apresentações, manuais) ficam em outras ferramentas, sem um formato de saída comum.

## 3. Público-alvo

- **Principal:** analistas e desenvolvedores que mantêm ou migram planilhas com macros.
- **Secundário:** pessoas sem perfil técnico que precisam documentar uma planilha ou entregá-la como contexto para uma IA.

## 4. Objetivos

1. Gerar o mapeamento de células (valor + fórmula) em JSON e Markdown com um clique.
2. Extrair todo o código VBA (Excel, Word e PowerPoint) em um Markdown organizado por componente.
3. Extrair o conteúdo de documentos Word, apresentações PowerPoint e PDFs em Markdown.
4. Diagnosticar planilhas (erros, referências externas, nomes, abas ocultas) e extrair Power Query.
5. Gerar uma saída otimizada para IA: arquivo único com estimativa de tokens e divisão em partes.
6. Funcionar 100% offline, sem enviar dados para fora da máquina.
7. Ser distribuído como um executável único para Windows.

## 5. Fora de escopo (não-objetivos)

- Editar os arquivos de entrada ou o código VBA.
- Recalcular fórmulas (o app usa os valores em cache salvos pelo Excel).
- Executar macros.
- Extrair o layout visual de UserForms (controles, posições, propriedades).
- Análise de segurança de macros (detecção de padrões suspeitos).
- Formatos binários antigos: `.xls`, `.doc`, `.ppt`.
- OCR: PDF escaneado (sem texto) só gera um aviso.
- Reproduzir a aparência dos documentos (formatação, imagens, gráficos, layout de slides e de páginas).
- Contagem exata de tokens de um modelo de IA (o app só estima).

## 6. Requisitos funcionais

### RF01 — Seleção de arquivo

- Abrir arquivo via botão (diálogo nativo) ou arrastando para a janela.
- Aceitar `.xlsx`, `.xlsm`, `.docx`, `.docm`, `.pptx`, `.pptm` e `.pdf`. O tipo é detectado pela extensão e a interface mostra só o que se aplica a ele.
- Exibir nome, tamanho, tipo e um resumo do conteúdo (abas, slides, páginas...) e se contém VBA.
- Tela inicial com histórico dos 10 últimos arquivos abertos (guardado localmente; clicar reabre, `X` remove, "Limpar histórico" apaga). Botão **Início** volta à tela inicial a partir de um arquivo aberto.

### RF02 — Seleção de abas

- Listar todas as abas com checkbox (todas marcadas por padrão).
- Permitir processar uma aba específica ou um subconjunto.

### RF03 — Extração de células

- Para cada célula não vazia, registrar a coordenada (`A1`, `B2`...).
- Se a célula tiver fórmula: registrar `formula` e `valor` (cache).
- Se não tiver: registrar apenas o valor.
- Ignorar células totalmente vazias.

### RF04 — Extração de VBA

- Detectar se o arquivo contém projeto VBA.
- Listar todos os componentes e extrair o código-fonte de cada um.
- Remover linhas em branco (comportamento do script atual; tornar configurável).
- Opção para ocultar linhas `Attribute VB_...` e `Option Explicit`.
- Classificar o componente (Módulo, Classe, Planilha, Pasta de trabalho, UserForm) quando possível.

### RF05 — Pré-visualização

- Aba **Células:** tabela com coordenada, fórmula e valor, filtrável por aba e por texto. Cada tabela do Excel (ListObject) vira uma linha com o intervalo na coluna Célula e o nome clicável em Valor (`A1:Z300 | tblProdutos`); ao clicar, abre a tabela com colunas nomeadas e botão Voltar. Os arquivos exportados não mudam.
- Aba **VBA:** lista de componentes à esquerda e código com destaque de sintaxe à direita.
- Abas por tipo de arquivo: **Diagnóstico** (RF14) e **Power Query** (RF13) nas planilhas; **Documento** (Word, RF16), **Slides** (PowerPoint, RF17) e **PDF** (RF18). Listas grandes são virtualizadas.

### RF06 — Exportação

- O botão **Exportar…** (ou `Ctrl+E`) abre um diálogo com as opções da exportação; o botão **Exportar** do diálogo pede a pasta de saída e grava os arquivos.
- Opções no diálogo: escopo de linhas (RF11), modelo do Markdown (RF10, só para aquela exportação), quais arquivos gerar e o nome de cada um (RF09), arquivo único para IA (RF19).
- Arquivos que podem ser gerados:
  - Planilhas: `mapeamento_planilha.json`, `mapeamento_planilha.md`, `vba_code.md`, `power_query.md`.
  - Word: `documento.md`. PowerPoint: `apresentacao.md`. PDF: `pdf.md`. Documentos com macro também geram `vba_code.md`.
  - Qualquer tipo: arquivo único para IA (`contexto_ia.md`, ou `contexto_ia_parte_N.md`).
- Botão para abrir a pasta de saída após exportar.

### RF07 — Avisos

- Alertar quando houver fórmulas sem valor em cache (arquivo provavelmente gerado por script e nunca salvo no Excel), sugerindo abrir, salvar e fechar no Excel.
- Alertar quando o arquivo estiver bloqueado (aberto no Excel) ou corrompido.

### RF08 — Configurações

- Janela de configurações (engrenagem na barra superior ou `Ctrl+,`).
- Persistem entre sessões, em `settings.json` na pasta de configuração do app: pasta de exportação padrão (o diálogo de exportação abre nela) e opções de limpeza do VBA.
- Arquivo ausente ou corrompido volta aos padrões, sem erro.

### RF09 — Nome dos arquivos

- Padrão de nome por arquivo gerado (`json`, `md` das células, `md` do VBA, arquivo único para IA e, conforme as fases, Power Query, documento, apresentação e PDF), com `{arquivo}` (arquivo de entrada, sem extensão) e `{data}` (AAAA-MM-DD). Extensão fixa. Configurável em Configurações → Saída e ajustável a cada exportação no diálogo Exportar.
- Validação (Windows): vazio, `< > : " / \ | ? *`, controle, terminar em ponto, nomes reservados (`CON`, `NUL`, `COM1`…), mais de 100 caracteres, dois arquivos com o mesmo nome. Falha antes de extrair.
- Se o arquivo já existir na pasta, o app pergunta: sobrescrever, adicionar sufixo (`_2`, `_3`…) ou cancelar.

### RF10 — Modelos de saída

- Os textos do Markdown de células e VBA são personalizáveis: título, texto de abertura, título de aba, cabeçalhos da tabela, rótulos de fórmula/valor, título de componente VBA e rodapé. Placeholders: `{arquivo}` (com extensão), `{data}`, `{aba}`, `{componente}`.
- O modelo **Padrão** reproduz byte a byte o formato da seção 8. O JSON nunca é personalizável.
- Predefinições: "Padrão", "Contexto para IA" (com texto de abertura) e as do usuário (salvar, substituir, excluir; duplicar = salvar com outro nome).
- Configurações → **Modelos** (aba própria): predefinição ativa, editor e pré-visualização ao vivo, renderizada pelo Rust com dados fictícios. No diálogo Exportar, o modelo pode ser trocado só para aquela exportação.
- Os modelos de Word, PowerPoint e PDF são próprios de cada formato (a predefinição de planilha não se aplica).

### RF11 — Escopo: primeiras linhas e amostra

- Seletor "Todas as linhas / Primeiras N linhas / Amostra de N linhas" na extração e, de forma independente, na exportação (que começa igual à extração).
- Conta linhas com conteúdo, por aba. **Primeiras N**: as N primeiras. **Amostra**: N linhas espaçadas por igual, sempre incluindo a primeira (mesma entrada, mesma amostra).
- Tabelas do Excel: o cabeçalho é mantido e o corpo segue a mesma seleção. O VBA não é cortado.
- Avisos: selo "amostra: X de Y linhas" na aba, alerta informativo e, no `mapeamento_planilha.md`, `> Amostra: X de Y linhas.` sob o título da aba. O JSON mantém o formato (só tem menos células). Os nomes sugeridos ganham `_amostra`.
- Limite conhecido: reduz saída e renderização, não o tempo de leitura (o arquivo é lido por inteiro).

### RF12 — Cache e recentes

- O histórico (até 10 arquivos) fica no app, em `recent.json`. Cada item mostra um selo: "em cache", "modificado", "não encontrado" ou "bloqueado", verificado em segundo plano (só metadados; não abre as planilhas) quando a tela inicial aparece.
- O app guarda em disco a última extração das 5 planilhas mais recentes. Reabrir uma planilha **que não mudou** (mesmo tamanho e data de modificação) restaura abas, escopo e resultado sem reler o arquivo; modificada, relê. "Recarregar" ignora o cache.
- A exportação reaproveita o resultado guardado quando abas, escopo e opções de VBA são iguais.
- Configurações → Geral: ligar/desligar o cache e "Limpar cache". Planilhas acima de 50 MB não entram no cache.

### RF13 — Extração de Power Query

- Detectar se o arquivo contém consultas Power Query (elemento `DataMashup` em `customXml/`).
- Extrair o código M de todas as consultas a partir de `Formulas/Section1.m`.
- Separar uma seção por consulta, preservando nomes com espaço ou acento (`#"Relatório Mensal"`).
- Extrair também as conexões de `xl/connections.xml` (nome, descrição e string de conexão).
- Arquivo sem consultas: exibir estado vazio, sem erro.
- A string de conexão só é exibida; o app nunca testa a conexão (RNF01).
- O resultado entra no cache (RF12), na exportação `power_query.md` (omitido se vazio) e no arquivo único para IA (RF19).

### RF14 — Diagnóstico

- Aba **Diagnóstico** (ao lado de Células e VBA): cartões de contagem e listas de erros de fórmula (`#REF!`, `#DIV/0!`…), fórmulas com referência a outro arquivo, abas ocultas (e "muito ocultas"), nomes definidos, tabelas e tabelas dinâmicas.
- Clicar numa célula da lista abre a aba Células nela (ou na tabela que a contém).
- Só leitura e local; não altera os arquivos exportados. Listas de células mostram até 500 itens; o total aparece nos cartões.

### RF15 — VBA em Word e PowerPoint

- Aceitar `.docm` e `.pptm`.
- Extrair o código VBA com a mesma lógica e saída do RF04 (`vba_code.md`).
- Projeto VBA lido de `word/vbaProject.bin` ou `ppt/vbaProject.bin`.

### RF16 — Extração de Word

- Aceitar `.docx` e `.docm`.
- Extrair, na ordem do documento:
  - títulos (pelo estilo `Título 1`, `Título 2`... / `Heading 1`...), convertidos em `#`, `##`...;
  - parágrafos e listas (numeradas e com marcadores);
  - tabelas, convertidas em tabelas Markdown;
  - cabeçalhos e rodapés;
  - comentários, com autor e trecho comentado;
  - campos de formulário (controles de conteúdo).
- Alterações controladas: exportar o texto final e, opcionalmente, listar inserções e exclusões.

### RF17 — Extração de PowerPoint

- Aceitar `.pptx` e `.pptm`.
- Uma seção por slide, na ordem da apresentação (não pela numeração dos arquivos internos).
- Por slide: título, texto de cada forma, tabelas e notas do apresentador.
- Opcional: tipo, posição e tamanho de cada forma (convertidos de EMU para centímetros).
- Indicar slides ocultos.

### RF18 — Extração de PDF

- Aceitar `.pdf` com texto selecionável.
- Extrair o texto por página.
- Extrair metadados (título, autor, datas) e campos de formulário (nome e valor).
- PDF sem texto (escaneado): exibir aviso, sem erro.

### RF19 — Saída otimizada para IA

- Opção "Arquivo único para IA": um só `.md` com resumo, conteúdo (células, documento, slides ou páginas), VBA e Power Query, usando o modelo escolhido.
- Estimativa de tokens (≈ caracteres ÷ 4), sempre rotulada como estimativa, mostrada no diálogo Exportar.
- Opção "Dividir em partes de N tokens": corta só entre seções (aba, tabela, componente VBA, slide, página), nunca no meio de um bloco de código ou de uma linha de tabela. A junção das partes reproduz o arquivo único. Uma seção maior que N fica sozinha na parte e passa do limite.

### RF20 — VBA avançado

- Índice de procedimentos (`Sub`, `Function`, `Property Get/Let/Set`, com `Private/Public/Static/Friend`) com linha inicial e final, na lista de componentes.
- Busca global (células e VBA) sobre o resultado já em memória.
- Exportar componentes como `.bas`, `.cls` e `.frm` (sem o `.frx`), em Windows-1252 para reimportar no editor VBA sem perder acentos.

### RF21 — Comparar versões

- Ao detectar que um arquivo mudou, o app guarda a geração anterior no cache e mostra as diferenças: abas adicionadas/removidas, células alteradas (antes e depois) e componentes VBA alterados (diff por linha).
- Aba **Mudanças** e exportação opcional em `.md`.

## 7. Requisitos não-funcionais

| ID    | Requisito                                                                                 |
| ----- | ----------------------------------------------------------------------------------------- |
| RNF01 | Processamento 100% local, sem chamadas de rede.                                           |
| RNF02 | Planilha com ~100 mil células processada em menos de 5 s em máquina comum.                |
| RNF03 | Interface não trava durante o processamento (trabalho em thread separada, com progresso). |
| RNF04 | Suporte completo a UTF-8 e acentuação em português, inclusive no VBA.                     |
| RNF05 | Plataforma principal: Windows 10/11.                                                      |
| RNF06 | Executável instalável sem privilégios de administrador (quando possível).                 |
| RNF07 | O arquivo original nunca é modificado (abertura somente leitura).                         |
| RNF08 | Cache só local, em disco, desligável e apagável; nenhum dado sai do computador.           |

## 8. Formato de saída

### JSON (células)

```json
{
  "NomeDaAba": {
    "A1": "Texto simples",
    "B2": 42,
    "C3": { "formula": "=SOMA(B1:B2)", "valor": 84 }
  }
}
```

A ordem das abas e das células segue a ordem da planilha. Datas à meia-noite saem como `"2024-01-15"`; erros de fórmula como texto (`"#DIV/0!"`); fórmula sem valor em cache tem `valor: null`.

### Markdown (células)

```markdown
# Mapeamento de Estrutura: arquivo.xlsm

## Aba: NomeDaAba

| Célula | Conteúdo Extraído                          |
| ------ | ------------------------------------------ |
| **A1** | Texto simples                              |
| **C3** | `Fórmula: =SOMA(B1:B2)` <br> **Valor:** 84 |
```

### Markdown (VBA)

````markdown
## COMPONENTE: Module1

```vba
Sub Exemplo()
    MsgBox "Olá"
End Sub
```
````

## 9. Fluxo principal

1. Usuário abre o app.
2. Arrasta o `.xlsm` para a janela.
3. App mostra resumo (abas, VBA detectado).
4. Usuário confere/ajusta as abas selecionadas.
5. Clica em **Extrair**; acompanha o progresso.
6. Confere a pré-visualização.
7. Clica em **Exportar**, escolhe a pasta e abre o resultado.

## 10. Métricas de sucesso

- Saída do modelo Padrão estável: idêntica, byte a byte, à da v1.0 (testes de integração).
- Zero dependência de Python na máquina do usuário.
- Tempo total do fluxo (abrir → exportar) abaixo de 30 s para arquivos típicos.

## 11. Roadmap

| Fase       | Entregas                                                                                                                                                                                                                      |
| ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **MVP**    | RF01, RF03, RF04, RF06 (sem pré-visualização, todas as abas).                                                                                                                                                                 |
| **v1.0**   | RF02, RF05, RF07, tema escuro.                                                                                                                                                                                                |
| **v1.1**   | Configurações persistentes (pasta padrão, opções de limpeza).                                                                                                                                                                 |
| **v1.2**   | Saída para IA (RF19), Power Query (RF13), base multiformato e VBA em Word/PowerPoint (RF15), Word (RF16), PowerPoint (RF17), PDF (RF18), VBA avançado (RF20), comparar versões (RF21). A ordem de entrega está no `GUIDE.md`. |
| **Futuro** | Processamento em lote (pasta inteira), mapa de dependências entre fórmulas.                                                                                                                                                   |

## 12. Riscos e questões em aberto

- **Valores em cache ausentes:** depende de o arquivo ter sido salvo pelo Excel. Mitigado pelo aviso RF07.
- **Codepage do VBA:** comentários com acentos podem vir com encoding errado em alguns projetos. Validar com arquivos reais.
- **Classificação de componentes:** a lib não informa o tipo diretamente; inferência por nome/atributos pode falhar em casos raros.
- **PDF:** a extração de texto depende de como o PDF foi gerado (colunas, tabelas e ordem de leitura podem sair misturadas). Validar com PDFs variados; a biblioteca ainda será escolhida (`GUIDE.md`, Fase 13).
- **Word:** os nomes dos estilos de título mudam com o idioma do Office; resolver pelo `styles.xml`.
- **Arquivos Office grandes ou maliciosos:** são ZIPs de XML; ler em streaming e tratar entradas gigantes como arquivo corrompido.
- **Tokens:** a estimativa (≈ caracteres ÷ 4) pode diferir bastante da contagem real de cada modelo.
- **Arquivos em pastas sincronizadas (OneDrive/SharePoint):** podem estar bloqueados ou como placeholder não baixado.
