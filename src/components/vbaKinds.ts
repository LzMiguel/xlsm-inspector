import type { VbaComponentKind } from '../types/models'

export const KIND_ORDER: VbaComponentKind[] = [
  'Module',
  'Class',
  'Document',
  'Workbook',
  'UserForm',
  'Unknown',
]

export const KIND_GROUP_LABEL: Record<VbaComponentKind, string> = {
  Module: 'Módulos',
  Class: 'Classes',
  Document: 'Planilhas',
  Workbook: 'Pasta de trabalho',
  UserForm: 'UserForms',
  Unknown: 'Outros',
}

export const KIND_LABEL: Record<VbaComponentKind, string> = {
  Module: 'Módulo',
  Class: 'Classe',
  Document: 'Planilha',
  Workbook: 'Pasta de trabalho',
  UserForm: 'UserForm',
  Unknown: 'Desconhecido',
}
