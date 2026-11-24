import { useEffect, useState } from 'react'

import type { ExtractScope } from '../types/models'
import { Input, Select } from './ui/Fields'

const DEFAULT_ROWS = 100

interface Props {
  value: ExtractScope
  onChange: (scope: ExtractScope) => void
  disabled?: boolean
}

export function ScopeSelector({ value, onChange, disabled }: Props) {
  const propRows = value.kind === 'full' ? DEFAULT_ROWS : value.rows
  const [rowsText, setRowsText] = useState(String(propRows))

  useEffect(() => {
    setRowsText(String(propRows))
  }, [propRows])

  return (
    <div className="flex items-center gap-2">
      <Select
        aria-label="Linhas a ler"
        className="w-auto"
        value={value.kind}
        disabled={disabled}
        onChange={(e) => {
          const kind = e.currentTarget.value as ExtractScope['kind']
          onChange(
            kind === 'full'
              ? { kind }
              : {
                  kind,
                  rows: value.kind === 'full' ? DEFAULT_ROWS : value.rows,
                },
          )
        }}
      >
        <option value="full">Todas as linhas</option>
        <option value="head">Só as primeiras linhas</option>
        <option value="sample">Amostra espaçada</option>
      </Select>
      {value.kind !== 'full' && (
        <Input
          type="number"
          min={1}
          aria-label="Quantas linhas"
          title="Quantas linhas"
          className="w-24"
          disabled={disabled}
          value={rowsText}
          onChange={(e) => {
            const text = e.currentTarget.value
            setRowsText(text)
            const n = Math.floor(Number(text))
            if (Number.isFinite(n) && n >= 1) {
              onChange({ kind: value.kind, rows: n })
            }
          }}
          onBlur={() => {
            const n = Math.floor(Number(rowsText))
            const valid = Number.isFinite(n) && n >= 1 ? n : 1
            setRowsText(String(valid))
            if (valid !== value.rows) {
              onChange({ kind: value.kind, rows: valid })
            }
          }}
        />
      )}
    </div>
  )
}
