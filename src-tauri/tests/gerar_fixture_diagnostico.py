import pathlib

import openpyxl
from openpyxl.workbook.defined_name import DefinedName
from openpyxl.worksheet.table import Table

out = pathlib.Path(__file__).parent / "fixtures" / "diagnostico.xlsx"

wb = openpyxl.Workbook()
ws = wb.active
ws.title = "Dados"
ws.append(["Item", "Valor"])
ws.append(["a", 1])
ws.append(["b", 2])
ws.add_table(Table(displayName="tblItens", ref="A1:B3"))

ws["D1"] = "#DIV/0!"  # célula de erro (tipo "e"), não texto
ws["D1"].data_type = "e"
ws["D2"] = "=[1]Externa!A1"  # referência a outro arquivo

wb.defined_names["Total"] = DefinedName("Total", attr_text="Dados!$B$2:$B$3")

wb.create_sheet("Oculta").sheet_state = "hidden"
wb.create_sheet("MuitoOculta").sheet_state = "veryHidden"

wb.save(out)
print(f"gerado: {out}")
