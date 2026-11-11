# Fixtures de teste

Planilhas sintéticas para `src-tauri/tests/fixtures/`. Nenhum dado real.

Sugestão: salve este arquivo como `src-tauri/tests/fixtures/README.md`, para documentar o que cada fixture cobre.

## Antes de começar: como o Excel grava fórmulas

O Excel em português **mostra** `=SOMA(A1;A2)`, mas **grava no arquivo** `SUM(A1,A2)`: nomes de função em inglês e separador `,`. O `calamine` lê o que está gravado e devolve a fórmula **sem o `=`**.

Por isso, nas tabelas abaixo:
- **Digitar no Excel** = o que você escreve na célula (Excel pt-BR).
- **Fórmula esperada** = o que o app deve exportar (já com o `=` que o app adiciona).

Datas também são gravadas como número de série (ex.: `45306` = 15/01/2024). Como o `calamine` as entrega como `DateTime`, decida no app como serializá-las (sugestão: ISO `2024-01-15`) e ajuste os valores esperados.

---

## 1. `simples.xlsx`

**Objetivo:** tipos básicos, duas abas, ordem das abas e células.

### Aba `Clientes`

| | A | B | C | D | E |
|---|---|---|---|---|---|
| **1** | ID | Nome | Cidade | Data Cadastro | Ativo |
| **2** | 1 | Ana Souza | Campinas | 15/01/2024 | VERDADEIRO |
| **3** | 2 | Bruno Lima | Santos | 03/02/2024 | FALSO |
| **4** | 3 | Carla Dias | Sorocaba | 28/02/2024 | VERDADEIRO |

- Coluna A: número inteiro.
- Coluna D: formatar como **Data** (digitar a data, não texto).
- Coluna E: digitar `VERDADEIRO`/`FALSO` (vira booleano).

### Aba `Produtos`

| | A | B | C | D |
|---|---|---|---|---|
| **1** | Codigo | Produto | Preco | Estoque |
| **2** | P001 | Caneta | 2,5 | 100 |
| **3** | P002 | Caderno | 15,9 | 0 |
| **4** | P003 | Mochila | 129,99 | -3 |

- Coluna C: decimais (testa `Float`).
- Coluna D: inclui `0` (não pode ser tratado como vazio) e negativo.

### Esperado
- Abas na ordem `Clientes`, `Produtos`.
- 15 células em `Clientes`, 16 em `Produtos`.
- Nenhuma fórmula, nenhum aviso.
- Trechos do JSON:
```json
{
  "Clientes": {
    "A2": 1,
    "D2": "2024-01-15",
    "E3": false
  },
  "Produtos": {
    "C4": 129.99,
    "D3": 0
  }
}
```

---

## 2. `formulas.xlsx`

**Objetivo:** fórmulas com valor em cache, acentuação em nome de aba e em texto, referência entre abas, erro.

Monte no Excel, **salve e feche** (para gravar o cache).

### Aba `Parâmetros`

| | A | B |
|---|---|---|
| **1** | Desconto | 0,1 |
| **2** | Região | São Paulo |

### Aba `Orçamento`

| | A | B | C | D |
|---|---|---|---|---|
| **1** | Item | Quantidade | Preço Unitário | Total |
| **2** | Lápis | 10 | 1,5 | *fórmula* |
| **3** | Borracha | 5 | 2 | *fórmula* |
| **4** | Régua | 2 | 4,75 | *fórmula* |

Fórmulas:

| Célula | Digitar no Excel | Fórmula esperada | Valor esperado |
|---|---|---|---|
| D2 | `=B2*C2` | `=B2*C2` | 15 |
| D3 | `=B3*C3` | `=B3*C3` | 10 |
| D4 | `=B4*C4` | `=B4*C4` | 9,5 |
| A6 | `Subtotal` (texto) | — | `Subtotal` |
| D6 | `=SOMA(D2:D4)` | `=SUM(D2:D4)` | 34,5 |
| A7 | `Com desconto` (texto) | — | `Com desconto` |
| D7 | `=D6*(1-Parâmetros!B1)` | `=D6*(1-Parâmetros!B1)` | 31,05 |
| A8 | `Situação` (texto) | — | `Situação` |
| D8 | `=SE(D7>30;"Atenção: acima do limite";"OK")` | `=IF(D7>30,"Atenção: acima do limite","OK")` | `Atenção: acima do limite` |
| D9 | `="Entrega em "&Parâmetros!B2` | `="Entrega em "&Parâmetros!B2` | `Entrega em São Paulo` |
| D10 | `=D6>20` | `=D6>20` | `true` |
| D11 | `=1/0` | `=1/0` | erro `#DIV/0!` |
| D12 | `=MÉDIA(B2:B4)` | `=AVERAGE(B2:B4)` | 5,666… |

> O Excel pode gravar a referência a outra aba com aspas (`'Parâmetros'!B1`). Confira o que o calamine devolve e ajuste o esperado.

### Esperado
- Abas na ordem `Parâmetros`, `Orçamento` (com acento intacto no JSON e no Markdown).
- 10 células com fórmula em `Orçamento`, todas com `valor` preenchido.
- D11 exportada como erro (ex.: `"#DIV/0!"`), sem derrubar a extração.
- Nenhum aviso de cache ausente.

---

## 3. `sem_cache.xlsx`

**Objetivo:** fórmulas sem valor em cache → aviso RF07.

**Não abra no Excel depois de gerar** (abrir e salvar grava o cache e estraga o teste). Gere com Python:

```python
import openpyxl

wb = openpyxl.Workbook()
ws = wb.active
ws.title = "Calculos"

ws["A1"] = "Valor A"
ws["B1"] = 10
ws["A2"] = "Valor B"
ws["B2"] = 20
ws["A3"] = "Soma"
ws["B3"] = "=SUM(B1:B2)"
ws["A4"] = "Dobro"
ws["B4"] = "=B3*2"
ws["A5"] = "Texto"
ws["B5"] = '=CONCATENATE("Total: ",B3)'

wb.save("sem_cache.xlsx")
```

### Esperado
- B3, B4 e B5 com `formula` preenchida e `valor` vazio/nulo.
- Um aviso indicando **3** fórmulas sem valor em cache.
- Células de valor fixo (A1:A5, B1, B2) normais.

---

## 4. `especiais.xlsx`

**Objetivo:** escape no Markdown, quebras de linha, aba vazia, dados que não começam em A1.

### Aba `Especiais`

| Célula | Conteúdo | Como digitar |
|---|---|---|
| A1 | `valor\|com\|pipe` | Normal |
| A2 | `Linha 1` ↵ `Linha 2` ↵ `Linha 3` | `Alt+Enter` entre as linhas |
| A3 | `Coluna\|A` ↵ `Coluna\|B` | Pipe + `Alt+Enter` |
| A4 | `   espaços nas pontas   ` | Com espaços antes e depois |
| A5 | `00123` | Digitar `'00123` (apóstrofo força texto) |
| A6 | `Ação, coração & "aspas"` | Acentos, vírgula, `&`, aspas |
| A7 | `` `crase` e **asterisco** `` | Caracteres de Markdown |
| A8 | Texto com ~300 caracteres | Qualquer parágrafo longo |
| C10 | `célula isolada` | Única célula fora do bloco |

### Aba `Deslocada`

Dados começando em **D5** (não em A1), para testar o `range.start()`:

| | D | E |
|---|---|---|
| **5** | Chave | Valor |
| **6** | x | 1 |
| **7** | y | 2 |

### Aba `Vazia`

Nenhum valor. Opcional: pinte o fundo de algumas células sem digitar nada (formatação sem conteúdo não deve virar célula).

### Esperado
- **JSON:** textos exatamente como digitados (`\n` preservado em A2/A3, espaços em A4, `"00123"` como string).
- **Markdown:**
  - `|` escapado (`\|`) em A1 e A3, para a tabela não quebrar.
  - Quebras de linha substituídas por ` | ` (comportamento do script original; com o escape, o ideal é usar `<br>`; decida e fixe no teste).
  - Tabela renderiza com 2 colunas em todas as linhas.
- `Deslocada`: coordenadas `D5`, `E5`, `D6`… e **não** `A1`, `B1`.
- `C10` aparece com a coordenada correta.
- `Vazia`: aparece com zero células (ou seção "sem dados"), sem erro.

---

## 5. `com_vba.xlsm`

**Objetivo:** todos os tipos de componente, acentos, linhas em branco e `Attribute`.

Crie um arquivo novo, salve como **Pasta de Trabalho Habilitada para Macro (`.xlsm`)**, abra o editor (`Alt+F11`) e adicione os componentes abaixo.

> No Excel em português, os componentes de documento se chamam **`EstaPasta_de_trabalho`** e **`Planilha1`** (não `ThisWorkbook`/`Sheet1`). A inferência de tipo do app precisa reconhecer os dois idiomas, ou ler os atributos.

### Módulo padrão: `modUtilitarios`
*(Inserir → Módulo; renomear na janela Propriedades)*

```vba
Option Explicit

' Funções utilitárias de formatação
' Atenção: não alterar sem revisar os relatórios


Public Function FormatarMoeda(ByVal valor As Double) As String
    ' Retorna o valor no padrão R$
    FormatarMoeda = "R$ " & Format(valor, "#,##0.00")
End Function

Public Sub ExibirMensagem()
    MsgBox "Operação concluída com êxito!", vbInformation, "Informação"
End Sub
```

(As duas linhas em branco após o comentário são propositais.)

### Módulo de classe: `clsFuncionario`
*(Inserir → Módulo de Classe)*

```vba
Option Explicit

Private pNome As String
Private pSalario As Double

' Propriedade Nome
Public Property Get Nome() As String
    Nome = pNome
End Property

Public Property Let Nome(ByVal valor As String)
    pNome = valor
End Property

' Calcula o salário com reajuste
Public Function SalarioReajustado(ByVal percentual As Double) As Double
    SalarioReajustado = pSalario * (1 + percentual)
End Function
```

### Pasta de trabalho: `EstaPasta_de_trabalho`

```vba
Option Explicit

' Executado ao abrir o arquivo
Private Sub Workbook_Open()
    Debug.Print "Arquivo aberto em " & Now
End Sub
```

### Planilha: `Planilha1`

```vba
Option Explicit

' Dispara quando uma célula da coluna A é alterada
Private Sub Worksheet_Change(ByVal Target As Range)
    If Target.Column = 1 Then
        Target.Offset(0, 1).Value = "Alterado às " & Format(Now, "hh:mm")
    End If
End Sub
```

### UserForm: `frmCadastro`
*(Inserir → UserForm. Adicione uma caixa de texto `txtNome` e um botão `btnSalvar`, depois clique duas vezes no botão.)*

```vba
Option Explicit

' Validação do formulário de cadastro
Private Sub btnSalvar_Click()
    If Trim(txtNome.Text) = "" Then
        MsgBox "Informe o nome do funcionário.", vbExclamation
        Exit Sub
    End If
    MsgBox "Funcionário " & txtNome.Text & " cadastrado.", vbInformation
    Unload Me
End Sub
```

Coloque também algum dado em `Planilha1!A1` (ex.: `Teste`), para o arquivo ter células e VBA ao mesmo tempo.

### Esperado
- `has_vba = true`.
- 5 componentes, com os tipos:

| Componente | Tipo esperado |
|---|---|
| `modUtilitarios` | Module |
| `clsFuncionario` | Class |
| `EstaPasta_de_trabalho` | Workbook |
| `Planilha1` | Document |
| `frmCadastro` | UserForm |

- Acentos corretos (`Operação`, `êxito`, `salário`, `às`, `funcionário`). Se vierem como `Opera��o`, o problema é de codepage.
- Com "remover linhas em branco" ativo: as duas linhas vazias de `modUtilitarios` somem.
- Com "ocultar Attribute" ativo: nenhuma linha começando com `Attribute VB_` no Markdown.
- O código do UserForm vem; o layout (`txtNome`, `btnSalvar` como controles) não.

### Variante opcional: `sem_vba.xlsm`
Um `.xlsm` salvo sem nenhum código (só `Planilha1` e `EstaPasta_de_trabalho` vazios), para checar se `has_vba` e a lista de componentes se comportam bem quando o projeto existe mas está vazio.

---

## 6. `corrompido.xlsx`

**Objetivo:** erro tratado, sem panic.

PowerShell:
```powershell
Set-Content -Path corrompido.xlsx -Value "isto nao e uma planilha"
```

Variante opcional `truncado.xlsx` (ZIP válido no começo, cortado no meio):
```powershell
$bytes = [System.IO.File]::ReadAllBytes("simples.xlsx")
[System.IO.File]::WriteAllBytes("truncado.xlsx", $bytes[0..($bytes.Length / 2)])
```

### Esperado
- `inspect_workbook` retorna `AppError` com código `CorruptedFile` (ou `UnsupportedFormat`), nunca panic.
- A interface mostra um `Alert` de erro e continua utilizável.

---

## Resumo de cobertura

| Caso | Fixture |
|---|---|
| Inteiro, decimal, zero, negativo | `simples` |
| Data, booleano | `simples` |
| Várias abas e ordem | `simples`, `formulas` |
| Fórmula com cache | `formulas` |
| Fórmula sem cache (aviso) | `sem_cache` |
| Nome de aba com acento | `formulas` |
| Referência a outra aba | `formulas` |
| Resultado de fórmula texto/bool/erro | `formulas` |
| Pipe e quebra de linha no Markdown | `especiais` |
| Texto com zero à esquerda | `especiais` |
| Dados fora de A1 / célula isolada | `especiais` |
| Aba vazia | `especiais` |
| Todos os tipos de componente VBA | `com_vba` |
| Acentos no VBA (codepage) | `com_vba` |
| Arquivo inválido | `corrompido` |
