# To-DOs

Melhorias mapeadas de forma manual, a fazer **antes** de seguir com o `GUIDE.md`.

Mapeamento fechado. Item marcado com **(proposta)** ainda pode ser ajustado.

## 0. Carregamento e resumo

Implementado e validado.

- [x] Ao abrir o arquivo, ler só o necessário para a visão geral, sem percorrer célula por célula:
      abas, dimensões (linhas × colunas), tabelas, módulos VBA e consultas.
- [x] Mostrar essas informações em um resumo na visualização central (não na sidebar).
- [x] A sidebar fica só com os nomes e as caixas de marcação.
- [x] A contagem de células e de fórmulas por aba sai da visão geral.
- [x] "Ler arquivo" lê apenas o que está marcado (planilhas; VBA e consultas ganham marcação no item 2).
- [x] Marcar depois algo não lido pede nova leitura. (gera alerta de item adicionado)
- [x] Indicador de tamanho na sidebar para planilhas: células aproximadas pela dimensão
      (linhas × colunas), abreviadas (ex.: "1,2 mi"), com alerta acima de 500 mil.
- [ ] Linhas de código do VBA na sidebar (junto com a seção VBA do item 2; já aparecem no Resumo).

### Dimensões: verificado no `calamine` 0.38.0

- Usar o próprio `calamine`: `Xlsx::worksheet_cells_reader(aba)?.dimensions()` lê o XML da aba
  só até o início de `sheetData`. Não é preciso ler o XML por conta própria.
- É a dimensão **declarada** no arquivo (`<dimension>`):
  - pode ser maior que o conteúdo (células só formatadas, dados esparsos);
  - sem a tag, volta o valor padrão, igual ao de uma aba só com `A1`.
    Tratar uma célula como "tamanho não confiável" ou conferir na fixture.
- Abrir o arquivo ainda carrega todos os textos compartilhados (`sharedStrings`) e estilos.
- Hoje o `inspect` (`excel/workbook.rs`) lê valores e fórmulas de todas as abas: é o que precisa sair.

## 1. VBA sem código

Implementado, **falta validar** (testes e uso).

- [x] Ocultar componentes sem código, na visualização **e** nas exportações.
      Sem código = só linhas em branco, `Option ...` e `Attribute ...`.
- [x] Manter componentes que têm apenas comentários (podem ser úteis).

## 2. Sidebar: visão geral do arquivo

- [ ] A sidebar mostra tudo o que o arquivo tem, em seções, com caixas de marcação:
  - Planilhas (abas do Excel; as células de cada uma seguem o escopo de linhas: todas, primeiras ou amostra);
  - VBA (componentes);
  - Power Query (consultas e conexões).
- [ ] As guias da visualização central continuam (Células, VBA, Diagnóstico, Power Query, Busca).
- [ ] Desmarcar um item (ou a seção inteira) o esconde da visualização central e se um item for desmarcado a aba some tambem.
- [ ] Tudo começa marcado.
- [ ] Substitui o menu atual que troca entre lista de planilhas e lista de componentes VBA.
- [ ] A Busca respeita as marcações.
- [ ] No VBA, a marcação é por módulo inteiro (não por `Sub`/`Function`).

## 3. Diálogo de exportação

- [ ] Seleção do que exportar em um menu próprio, mais claro (seções iguais às da sidebar).
- [ ] Ao abrir, o diálogo copia as marcações da sidebar.
      Mudanças feitas no diálogo **não** voltam para a sidebar.
- [ ] Botão principal "Exportar" (Markdown) e botão menor "Exportar JSON", independente do primeiro.
- [ ] O JSON contém só as células das planilhas marcadas (estrutura atual inalterada).
- [ ] Toggle **Compilado** (padrão) ou **Separado**:
  - Compilado: um Markdown único para IA com tudo o que está marcado;
  - Separado: um `.zip` com um arquivo por tipo.
- [ ] Compilado dividido em partes também sai em `.zip`.
- [ ] Diagnóstico é opcional, **desmarcado** por padrão (hoje não existe em nenhuma exportação: precisa ser criado no Rust).
- [ ] Opção de exportar só o que está na tela (a planilha ou o módulo VBA exibido).
      Vale também para a consulta do Power Query aberta.
- [ ] Nome sugerido = nome do arquivo + o que foi exportado; renomear apenas na janela "Salvar como" do Windows.
      Remover os campos de nome do diálogo.
- [ ] Manter os nomes padrão das Configurações e os modelos de exportação.

## 4. Visualização

- [ ] Compactar os cards do topo (arquivo e "Ler arquivo") ao rolar o conteúdo.
- [ ] Opção de quebra de linha nas previews de código (VBA e Power Query).
- [ ] Colapsar as seções do Diagnóstico.

## 5. Memória da interface

- [ ] Lembrar: opções de exportação, guia ativa, quebra de linha, itens colapsados e escopo de linhas.
- [ ] Lembrar a janela: tamanho, posição, monitor e maximizada. Não reabrir minimizada.
      Plugin oficial `tauri-plugin-window-state` (dependência nova, sem rede); conferir antes de adotar.
- [ ] As marcações da sidebar **não** são lembradas: cada arquivo abre com tudo marcado.

## 6. Depois: leitura parcial de verdade

- [ ] "Primeiras linhas" e "Amostra" já existem, mas são aplicadas **depois** de ler a aba inteira:
      reduzem a saída e a tela, não o tempo de leitura.
- [ ] Viável com a leitura em fluxo (`next_cell_with_formula`):
  - primeiras linhas: parar na linha N, com ganho real de tempo e memória;
  - amostra: ainda percorre a aba toda; ganha memória, quase não ganha tempo.
- Limite: tabelas do Excel usam `table_by_name_ref`, que lê a aba inteira. Seria preciso montar
  as tabelas a partir das células já lidas.
- Substitui a Decisão 5 do `GUIDE.md` quando for feito.

## Impacto técnico

- Rust: filtro de componentes VBA e consultas na exportação (hoje só filtra planilhas), diagnóstico em Markdown, geração do `.zip` (a dependência `zip` já existe) e filtro de VBA sem código, com testes.
- Documentos: atualizar `PRD.md`, `Architecture.md` e `Design-System.md`; revisar a Fase 3 do `GUIDE.md`, que se sobrepõe a estes itens.

## Ordem sugerida

1. Backend com testes (itens 0, 1 e a parte Rust do item 3).
2. Sidebar (item 2).
3. Diálogo de exportação (item 3).
4. Visualização e memória (itens 4 e 5).
