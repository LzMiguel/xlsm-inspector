import type { ExtractScope } from '../types/models'
import { Input, Select } from './ui/Fields'

const DEFAULT_ROWS = 100

interface Props {
  value: ExtractScope
  onChange: (scope: ExtractScope) => void
  disabled?: boolean
}

export function ScopeSelector({ value, onChange, disabled }: Props) {
  return (
    <div className="flex items-center gap-2">
      <Select
        aria-label="Linhas a extrair"
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
        <option value="head">Primeiras N linhas</option>
        <option value="sample">Amostra de N linhas</option>
      </Select>
      {value.kind !== 'full' && (
        <Input
          type="number"
          min={1}
          aria-label="Número de linhas"
          className="w-24"
          disabled={disabled}
          value={value.rows}
          onChange={(e) =>
            onChange({
              kind: value.kind,
              rows: Math.max(1, Math.floor(Number(e.currentTarget.value)) || 1),
            })
          }
        />
      )}
    </div>
  )
}
