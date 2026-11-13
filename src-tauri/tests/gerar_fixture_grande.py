"""Gera tests/fixtures/grande.xlsx (100 mil células sintéticas) para o teste de desempenho (RNF02).

Uso: python src-tauri/tests/gerar_fixture_grande.py   (precisa de openpyxl)
"""
import pathlib

import openpyxl

out = pathlib.Path(__file__).parent / "fixtures" / "grande.xlsx"

wb = openpyxl.Workbook(write_only=True)
ws = wb.create_sheet("Dados")
for r in range(1, 5001):  # 5000 linhas x 20 colunas
    ws.append([f"Item {r}", r, r * 1.5, r % 7 == 0] + [r * c for c in range(1, 17)])
wb.save(out)
print(f"gerado: {out}")
