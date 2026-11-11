# PRD — XLSM Inspector

## 1. Visão geral

O **XLSM Inspector** é um aplicativo desktop (Tauri + React) que lê planilhas do Excel (`.xlsx` / `.xlsm`) e gera um mapeamento completo do conteúdo em dois formatos:

- **JSON**, para automações e scripts.
- **Markdown**, para leitura humana e para servir de contexto a agentes de IA.

Além das células (valores e fórmulas), o app extrai o **código VBA** de módulos, módulos de classe, planilhas, `ThisWorkbook` e UserForms.

Ele substitui dois scripts Python (`openpyxl` + `oletools`) por uma ferramenta única, com interface, sem depender de Python instalado na máquina.

## 2. Problema

- Entender uma planilha complexa (muitas abas, fórmulas encadeadas, macros) exige abrir célula por célula e o editor VBA.
- Agentes de IA não leem `.xlsm` diretamente; precisam de uma representação em texto.
- Os scripts atuais dependem de Python e bibliotecas instaladas, o que dificulta o uso por outras pessoas da equipe.

## 3. Público-alvo

- **Principal:** analistas e desenvolvedores que mantêm ou migram planilhas com macros.
- **Secundário:** pessoas sem perfil técnico que precisam documentar uma planilha ou entregá-la como contexto para uma IA.

## 4. Objetivos

1. Gerar o mapeamento de células (valor + fórmula) em JSON e Markdown com um clique.
2. Extrair todo o código VBA em um Markdown organizado por componente.
3. Funcionar 100% offline, sem enviar dados para fora da máquina.
4. Ser distribuído como um executável único para Windows.

## 5. Fora de escopo (não-objetivos)

- Editar a planilha ou o código VBA.
- Recalcular fórmulas (o app usa os valores em cache salvos pelo Excel).
- Executar macros.
- Extrair o layout visual de UserForms (controles, posições, propriedades).
- Análise de segurança de macros (detecção de padrões suspeitos).
- Suporte a `.xls` (formato binário antigo) no MVP.

## 6. Requisitos funcionais

### RF01 — Seleção de arquivo

- Abrir arquivo via botão (diálogo nativo) ou arrastando para a janela.
- Aceitar `.xlsx` e `.xlsm`.
- Exibir nome, tamanho, número de abas e se contém VBA.

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
- Opção para ocultar linhas `Attribute VB_...`.
- Classificar o componente (Módulo, Classe, Planilha, Pasta de trabalho, UserForm) quando possível.

### RF05 — Pré-visualização

- Aba **Células:** tabela com coordenada, fórmula e valor, filtrável por aba e por texto.
- Aba **VBA:** lista de componentes à esquerda e código com destaque de sintaxe à direita.

### RF06 — Exportação

- Escolher a pasta de saída.
- Gerar os arquivos:
  - `mapeamento_planilha.json`
  - `mapeamento_planilha.md`
  - `vba_code.md`
- Permitir escolher quais arquivos gerar.
- Botão para abrir a pasta de saída após exportar.

### RF07 — Avisos

- Alertar quando houver fórmulas sem valor em cache (arquivo provavelmente gerado por script e nunca salvo no Excel), sugerindo abrir, salvar e fechar no Excel.
- Alertar quando o arquivo estiver bloqueado (aberto no Excel) ou corrompido.

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

A ordem das abas e das células segue a ordem da planilha.

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

- Saída equivalente à dos scripts Python atuais para os mesmos arquivos de teste.
- Zero dependência de Python na máquina do usuário.
- Tempo total do fluxo (abrir → exportar) abaixo de 30 s para arquivos típicos.

## 11. Roadmap

| Fase       | Entregas                                                                                                                    |
| ---------- | --------------------------------------------------------------------------------------------------------------------------- |
| **MVP**    | RF01, RF03, RF04, RF06 (sem pré-visualização, todas as abas).                                                               |
| **v1.0**   | RF02, RF05, RF07, tema escuro.                                                                                              |
| **v1.1**   | Configurações persistentes (pasta padrão, opções de limpeza), histórico de arquivos recentes.                               |
| **Futuro** | Processamento em lote (pasta inteira), mapa de dependências entre fórmulas, exportação de um único `.md` combinado para IA. |

## 12. Riscos e questões em aberto

- **Valores em cache ausentes:** depende de o arquivo ter sido salvo pelo Excel. Mitigado pelo aviso RF07.
- **Codepage do VBA:** comentários com acentos podem vir com encoding errado em alguns projetos. Validar com arquivos reais.
- **Classificação de componentes:** a lib não informa o tipo diretamente; inferência por nome/atributos pode falhar em casos raros.
- **Arquivos em pastas sincronizadas (OneDrive/SharePoint):** podem estar bloqueados ou como placeholder não baixado.
