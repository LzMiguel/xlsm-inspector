# Design System — XLSM Inspector

Aplicativo de utilidade técnica: a interface deve ser **limpa, densa em informação e calma**. O conteúdo (células e código) é o protagonista; a interface fica em segundo plano.

**Princípios**

1. **Clareza antes de decoração:** nada de gradientes ou sombras.
2. **Dados legíveis:** fonte monoespaçada para coordenadas, fórmulas e código.
3. **Estado sempre visível:** o usuário sabe o que foi carregado, o que está processando e o que foi exportado.
4. **Teclado e mouse:** tudo acessível pelos dois.

Implementação sugerida: **Tailwind CSS** com os tokens abaixo como variáveis CSS, e componentes base do **shadcn/ui** (ou Radix) adaptados.

---

## 1. Cores

Todas as cores são tokens. Componentes nunca usam valores hexadecimais diretamente.

### Tema claro

| Token             | Valor     | Uso                                                         |
| ----------------- | --------- | ----------------------------------------------------------- |
| `--bg`            | `#F7F8FA` | Fundo da janela                                             |
| `--surface`       | `#FFFFFF` | Cards, painéis, tabelas                                     |
| `--surface-muted` | `#F0F2F5` | Cabeçalho de tabela, áreas secundárias                      |
| `--border`        | `#E1E4EA` | Bordas e divisores                                          |
| `--text`          | `#1A1D23` | Texto principal                                             |
| `--text-muted`    | `#5F6672` | Texto secundário, legendas                                  |
| `--text-subtle`   | `#666D79` | Ícones, desabilitado (contraste AA)                         |
| `--primary`       | `#1F7A55` | Ações principais (verde planilha)                           |
| `--primary-hover` | `#186446` | Hover da ação principal                                     |
| `--primary-soft`  | `#E3F2EB` | Fundo de seleção, badges                                    |
| `--accent`        | `#4B5BD6` | Links, foco, destaque de VBA                                |
| `--success`       | `#3F8212` | Exportação concluída (verde-limão, distinto do `--primary`) |
| `--warning`       | `#8F5D0F` | Valores em cache ausentes (texto AA)                        |
| `--warning-soft`  | `#FDF4E3` | Fundo de alerta                                             |
| `--danger`        | `#C53030` | Erros                                                       |
| `--danger-soft`   | `#FDECEC` | Fundo de erro                                               |

### Tema escuro

| Token             | Valor     |
| ----------------- | --------- |
| `--bg`            | `#111318` |
| `--surface`       | `#181B21` |
| `--surface-muted` | `#20242B` |
| `--border`        | `#2C313A` |
| `--text`          | `#E7E9ED` |
| `--text-muted`    | `#A0A7B2` |
| `--text-subtle`   | `#8A919C` |
| `--primary`       | `#3DB884` |
| `--primary-hover` | `#52C894` |
| `--primary-soft`  | `#173326` |
| `--accent`        | `#8592F0` |
| `--success`       | `#93D155` |
| `--warning`       | `#E0A84A` |
| `--warning-soft`  | `#33280F` |
| `--danger`        | `#F06A6A` |
| `--danger-soft`   | `#3A1717` |

### Destaque de sintaxe VBA

| Token            | Claro     | Escuro    | Elemento                  |
| ---------------- | --------- | --------- | ------------------------- |
| `--code-keyword` | `#4B5BD6` | `#8592F0` | `Sub`, `Dim`, `If`, `End` |
| `--code-string`  | `#1F7A55` | `#7FD1A8` | `"texto"`                 |
| `--code-comment` | `#5F6672` | `#8A919C` | `' comentário`            |
| `--code-number`  | `#B7591F` | `#E8A06A` | `42`, `3.14`              |
| `--code-bg`      | `#F7F8FA` | `#13161B` | Fundo do bloco            |

O tema segue o sistema operacional por padrão. O botão na barra superior alterna automático → claro → escuro (a escolha fica em `localStorage`; `data-theme` no `<html>` força o tema).

---

## 2. Tipografia

| Família | Fonte              | Fallback                             | Uso                           |
| ------- | ------------------ | ------------------------------------ | ----------------------------- |
| Sans    | **Inter**          | `system-ui, Segoe UI, sans-serif`    | Interface                     |
| Mono    | **JetBrains Mono** | `Cascadia Code, Consolas, monospace` | Coordenadas, fórmulas, código |

Incluir as fontes no bundle (sem carregar da internet, por causa do RNF01).

### Escala

| Token       | Tamanho / Altura | Peso | Uso                           |
| ----------- | ---------------- | ---- | ----------------------------- |
| `text-xs`   | 11px / 16px      | 500  | Badges, metadados             |
| `text-sm`   | 13px / 20px      | 400  | Tabelas, código, texto padrão |
| `text-base` | 14px / 22px      | 400  | Texto de corpo                |
| `text-lg`   | 16px / 24px      | 600  | Títulos de seção              |
| `text-xl`   | 20px / 28px      | 600  | Título de página              |
| `text-2xl`  | 24px / 32px      | 700  | Estado vazio / boas-vindas    |

Tabelas e código usam `text-sm` para caber mais conteúdo. Números em tabelas usam `font-variant-numeric: tabular-nums`.

---

## 3. Espaçamento

Escala base de **4px**.

| Token      | Valor | Uso típico                               |
| ---------- | ----- | ---------------------------------------- |
| `space-1`  | 4px   | Ícone ↔ texto                            |
| `space-2`  | 8px   | Padding interno de badges, gaps pequenos |
| `space-3`  | 12px  | Padding de células de tabela, inputs     |
| `space-4`  | 16px  | Padding de cards, gap entre controles    |
| `space-6`  | 24px  | Separação entre seções                   |
| `space-8`  | 32px  | Margem de página                         |
| `space-12` | 48px  | Estados vazios                           |

### Bordas e elevação

| Token       | Valor                                               |
| ----------- | --------------------------------------------------- |
| `radius-sm` | 4px (badges, inputs)                                |
| `radius-md` | 6px (botões, cards)                                 |
| `radius-lg` | 10px (modais, drop zone)                            |
| `shadow-sm` | `0 1px 2px rgb(0 0 0 / 0.06)`                       |
| `shadow-md` | `0 4px 12px rgb(0 0 0 / 0.08)` (só modais e toasts) |

### Layout

- Janela mínima: **960 × 640**.
- Barra superior fixa: 48px (nome do arquivo, ações principais).
- Barra lateral (seções Planilhas, VBA e Power Query, com a exportação no rodapé): 240px, fixa.
- Área de conteúdo: preenche o restante, com scroll próprio. A pré-visualização ocupa a altura que sobra (mínimo 384px); em janelas baixas a página rola.

---

## 4. Ícones

**Lucide** (`lucide-react`), tamanho 16px na interface e 20px em botões grandes, `stroke-width` 1.75.

| Ícone             | Uso                  |
| ----------------- | -------------------- |
| `FileSpreadsheet` | Arquivo carregado    |
| `Table2`          | Aba / células        |
| `Code2`           | VBA                  |
| `FunctionSquare`  | Célula com fórmula   |
| `Download`        | Exportar             |
| `FolderOpen`      | Abrir pasta de saída |
| `AlertTriangle`   | Aviso                |

---

## 5. Componentes

### Button

- **Variantes:** `primary`, `secondary` (borda), `ghost` (sem borda), `danger`.
- **Tamanhos:** `sm` (28px), `md` (32px), `lg` (40px).
- **Estados:** hover, foco (anel `--accent` de 2px), desabilitado (opacidade 50%), carregando (spinner substitui o ícone).
- Só um botão `primary` visível por tela.

### DropZone

- Área tracejada (`--border`, 2px, `radius-lg`) com ícone, texto "Arraste um arquivo (.xlsx, .xlsm, .docx, .docm, .pptx, .pptm ou .pdf)" e botão "Selecionar arquivo".
- Ao arrastar por cima: borda `--primary`, fundo `--primary-soft`.
- Arquivo inválido: borda `--danger` e mensagem curta.
- Abaixo do botão, uma linha `text-xs` `--text-muted` explica o que o app gera e que tudo é local.

### FileCard

- Mostra ícone, nome, tamanho, nº de abas e badge "VBA" (se houver).
- "Início" (`ghost`) volta à tela inicial; "Trocar arquivo" (`secondary`) abre o seletor.

### SheetList

- Lista com checkbox por aba e o tamanho aproximado em células (`1,2 mi`; "—" se não informado), com a dimensão no `title`. Acima de 500 mil: texto `--warning` com ícone `TriangleAlert`.
- Guia **Resumo** (`WorkbookOverview`, ícone `LayoutList`): guia inicial; seções em lista (Planilhas, VBA, Consultas, Conexões). Depois do "Ler arquivo", passa para Células.
- `ExtractPanel`: `Alert` `warning` "Marcado, mas ainda não lido" lista as abas marcadas que não estão no resultado.
- Ações "Marcar todas" / "Desmarcar todas". Título "Planilhas". Texto de ajuda: "Marque o que será lido, exibido e exportado." Com mais de 8 planilhas, aparece um campo "Filtrar planilhas…" (só filtra a lista; "Marcar todas" vale para todas).

### Dialog / SettingsDialog

- `<dialog>` nativo modal (foco preso, `Esc` fecha), `radius-lg`, `shadow-md`, fundo (`::backdrop`) escurecido com o token `--color-overlay` (preto a 60%, igual nos dois temas), largura máx. 512px.
- Cabeçalho com título `text-lg` e botão fechar. Corpo em seções com título `text-sm` semibold.
- Abre pela engrenagem da barra superior ou `Ctrl+,`.
- `wide`: largura máx. 768px (usada nas configurações). Abas "Geral", "Saída" e "Modelos" (`Tabs`).
- Aba "Saída" (`OutputSettings`): campos de nome de arquivo.
- Aba "Modelos" (`TemplateSettings`): seletor de predefinição, formulário do modelo e `<pre>` de pré-visualização em duas colunas (uma só em telas estreitas).

### ExportPanel (diálogo Exportar)

- Na barra lateral há só o botão "Exportar…" (`Ctrl+E`), que abre um `Dialog`. As opções ficam no diálogo: `ScopeSelector`, `Select` "Modelo do texto (Markdown)" (só para aquela exportação), um `Checkbox` por arquivo (com o campo de nome e a extensão) e, no arquivo único para IA, "Dividir em partes de N tokens" (`Input` numérico) e a estimativa de tokens em `text-xs` `--text-muted`, sempre rotulada "estimativa".
- Rodapé do diálogo: "Cancelar" (`ghost`) e "Exportar" (`primary`, com `loading`). Conflito de nome e erros aparecem como `Alert` dentro do diálogo; o resultado vira `Toast` "Abrir pasta" depois que o diálogo fecha.

### Recentes e cache

- Cada recente mostra um `Badge`: "salvo" (`primary`), "modificado" e "bloqueado" (`warning`), "não encontrado" (`danger`). Com cache ou modificado, um botão ícone (recarregar) ignora o cache.
- Resultado vindo do cache: `Alert` `info` "Resultado salvo" com botão "Recarregar" no `ExtractPanel` (o botão principal do painel é "Ler arquivo" / "Ler de novo"). O texto explica que o arquivo não foi relido e não mudou desde a última leitura, e orienta a recarregar se o resultado parecer antigo.
- Erros (`ErrorAlert`): `CorruptedFile` usa o título "Arquivo corrompido, inválido ou protegido por senha"; `UnsupportedFormat` mostra a extensão recebida ou "arquivo sem extensão".

### Diagnóstico (`DiagnosticsView`)

- Texto da UI: "planilha" é o arquivo e também cada aba, como no Excel em português; padronizar "planilha" x "aba" está na Fase 3 do `GUIDE.md`.

- Grade de cartões (`--surface-muted`, `radius-md`) com a contagem em `text-lg` e o rótulo em `text-xs`; contagem em `--danger` quando há erros ou referências externas.
- Abaixo, uma seção por categoria (`text-sm` semibold) com lista em linhas divididas por `--border`. Células são botões `--accent` sublinhados (`Aba!A1`) que abrem a aba Células. "muito oculta" usa `Badge` `warning`. Ícone da aba: `Stethoscope`.

### Power Query (`PowerQueryView`)

- Aba **Power Query** (ícone `Database`), entre Diagnóstico e Busca. Enquanto lê: "Lendo Power Query…" em `text-sm` `--text-muted`.
- Seção "Consultas (N)": um `<details>` por consulta (`--surface-muted`, borda `--border`, `radius-md`); o título é o nome e, aberto, mostra o código M em `font-mono text-xs` com rolagem.
- Seção "Conexões (N)": cartão por conexão com o nome em `text-sm` medium, a descrição em `text-xs text-muted` e a string de conexão em `font-mono text-xs` com quebra (`break-all`). Só exibe; nunca testa a conexão.
- Aberta, a consulta mostra o código M com destaque (Shiki, `powerquery`) e o botão "Copiar". Erro de leitura aparece dentro da aba, com "Tentar de novo". Sem consultas nem conexões: `EmptyState` "Sem Power Query". No diálogo Exportar, "Power Query (Markdown)" é uma linha como as outras (checkbox + nome editável) e é omitido na gravação se o arquivo não tiver nada.

### Pré-visualização antes da leitura

- A pré-visualização aparece assim que o arquivo abre. As guias Células, VBA e Busca mostram `EmptyState` "Nada lido ainda" até o "Ler arquivo"; Diagnóstico e Power Query funcionam sem ler.
- As opções do VBA ("Remover linhas em branco", "Ocultar linhas Attribute VB\_") aparecem na seção VBA da barra lateral enquanto a guia VBA está aberta; também estão em Configurações → Geral.
- O diálogo Exportar termina com uma linha `text-xs` `--text-muted` com os nomes finais dos arquivos marcados.

### ScopeSelector

- `Select` (Todas as linhas / Só as primeiras linhas / Amostra espaçada) + `Input` numérico "Quantas linhas" (mín. 1, padrão 100) quando não é "Todas". Aparece no `ExtractPanel` e no `ExportPanel`.
- Selo "amostra: X de Y linhas" no `CellTable`: `radius-sm`, fundo `--warning-soft`, `text-xs`. Aviso `Sampled` usa `Alert` variante `info`.

### Fields (`ui/Fields.tsx`)

- `Input`, `Textarea` (monoespaçada), `Select`: borda `--border`, fundo `--surface`, `radius-sm`, foco com contorno `--accent`.
- `Field`: rótulo `text-sm` medium + controle + dica `text-xs` `--muted`; envolve tudo em `<label>`.

### Tabs

- Alterna entre **Células** e **VBA**.
- Sublinhado de 2px em `--primary` na aba ativa.

### DataTable (células)

- Colunas: Célula (mono, 80px), Fórmula (mono), Valor.
- Cabeçalho fixo com `--surface-muted`.
- Linhas com 32px de altura; zebra opcional.
- **Virtualizada** (ex.: `@tanstack/react-virtual`) para aguentar centenas de milhares de linhas.
- Campo de busca acima, filtrando por coordenada, fórmula ou valor.

### CodeViewer (VBA)

- Fonte mono `text-sm`, números de linha em `--text-subtle`.
- Destaque de sintaxe com os tokens da seção 1 (Shiki ou Prism com gramática VBA).
- Botão "Copiar" no canto superior direito.

### ComponentList (VBA)

- Seção da barra lateral, presente quando há módulos. Cabeçalho com `Checkbox` "VBA (marcados/total)" (indeterminado quando parcial).
- Itens agrupados por tipo: Módulos, Classes, Planilhas, UserForms.
- Cada item: `Checkbox` + botão com o nome (abre a guia VBA; desabilitado se desmarcado) e badge com o número de linhas.

### PowerQueryList

- Seção da barra lateral com o mesmo cabeçalho marcável; subgrupos "Consultas" e "Conexões", um `Checkbox` por item.
- Guias sem nada marcado (Células, VBA, Power Query) somem; se era a guia ativa, volta ao Resumo.
- No Diagnóstico, o link de célula de uma aba desmarcada fica desabilitado.

### ProgressBar

- Barra de 4px em `--primary`, com texto "Processando aba X de Y".
- Indeterminada quando não houver total conhecido.

### Alert

- Variantes `warning`, `danger`, `info`.
- Ícone + título + descrição + ação opcional.
- Exemplo: "12 fórmulas sem valor em cache. Abra o arquivo no Excel, salve e feche, e tente de novo."

### Toast

- Canto inferior direito, some em 8 s; fica parado enquanto o mouse ou o foco estão sobre ele.
- Usado para "Exportação concluída" com ação "Abrir pasta".

### EmptyState

- Ícone grande (48px, `--text-subtle`), título `text-xl` e descrição curta.
- Usado antes de carregar arquivo e quando o arquivo não tem VBA.

---

## Vocabulário da interface

| Use                                              | Evite                     | Observação                                                                                                                                                               |
| ------------------------------------------------ | ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **Planilha**                                     | aba                       | Cada folha do arquivo. "Guia" é a seção da pré-visualização (Células, VBA…). Nos modelos Markdown o título "Aba" e o placeholder `{aba}` continuam (formato Padrão fixo) |
| **Ler arquivo** / **Exportar**                   | extrair                   | Ler = trazer o conteúdo para a tela; Exportar = gravar os arquivos                                                                                                       |
| **Modelo**                                       | predefinição, template    | "Padrão", "Contexto para IA" e "Meus modelos"                                                                                                                            |
| **salvo** / **resultados salvos**                | cache                     | Só a documentação técnica usa "cache"                                                                                                                                    |
| **Vínculos externos**                            | refs. externas            |                                                                                                                                                                          |
| **Amostra espaçada**, **só as primeiras linhas** | amostra de N, primeiras N | Sem "N" nem outras variáveis                                                                                                                                             |

## 6. Movimento

- Transições de 120–180 ms, `ease-out`.
- Animar apenas cor, opacidade e transform.
- Respeitar `prefers-reduced-motion`.

## 7. Acessibilidade

- Contraste mínimo AA (4.5:1 para texto).
- Foco sempre visível.
- Navegação completa por teclado na tabela e na lista de componentes.
- Atalhos: `Ctrl+O` abrir, `Ctrl+E` exportar, `Ctrl+,` configurações, `Ctrl+F` buscar (na aba Células foca o filtro da tabela; nas demais abas abre a aba Busca).
- Ícones sem texto têm `aria-label`.

### Índice de procedimentos e Busca (RF20)

- `ComponentList`: o componente selecionado expande seus procedimentos (texto `text-xs text-muted`); clicar leva à linha no `VbaViewer`. Botão secundário "Exportar como arquivos" (só os marcados) no fim da lista, na guia VBA.
- Aba **Busca** (`SearchView`): campo de busca + lista de resultados (selo Célula/VBA, local em `font-mono text-accent`, trecho). Clicar abre a célula ou a linha de VBA.
