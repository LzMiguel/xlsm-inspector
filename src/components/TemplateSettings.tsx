import { useEffect, useState } from 'react'
import { Save, Trash2 } from 'lucide-react'

import {
  getBuiltinTemplates,
  previewTemplate,
  toAppError,
} from '../api/commands'
import {
  updateSettings,
  updateSettingsDebounced,
} from '../store/settingsActions'
import { useSettingsStore } from '../store/settingsStore'
import type { AppError, OutputTemplate, TemplatePreset } from '../types/models'
import { ErrorAlert } from './ErrorAlert'
import { Button } from './ui/Button'
import { Field, Input, Select, Textarea } from './ui/Fields'

const TEMPLATE_FIELDS: {
  key: keyof OutputTemplate
  label: string
  hint?: string
  multiline?: boolean
}[] = [
  { key: 'title', label: 'Título', hint: 'Use {arquivo} e {data}.' },
  {
    key: 'intro',
    label: 'Texto de abertura (opcional)',
    hint: 'Logo após o título. Ex.: instruções para uma IA.',
    multiline: true,
  },
  { key: 'sheet_heading', label: 'Título de cada planilha', hint: 'Use {aba}.' },
  { key: 'cell_header', label: 'Cabeçalho da coluna de células' },
  { key: 'content_header', label: 'Cabeçalho da coluna de conteúdo' },
  { key: 'formula_label', label: 'Rótulo da fórmula' },
  { key: 'value_label', label: 'Rótulo do valor' },
  {
    key: 'vba_heading',
    label: 'Título de cada componente VBA',
    hint: 'Use {componente}.',
  },
  { key: 'footer', label: 'Rodapé (opcional)', multiline: true },
]

const same = (a: OutputTemplate, b: OutputTemplate) =>
  (Object.keys(a) as (keyof OutputTemplate)[]).every((k) => a[k] === b[k])

export function TemplateSettings() {
  const { template, presets } = useSettingsStore((s) => s.settings)
  const [builtin, setBuiltin] = useState<TemplatePreset[]>([])
  const [preview, setPreview] = useState('')
  const [newName, setNewName] = useState('')
  const [error, setError] = useState<AppError | null>(null)

  useEffect(() => {
    let cancelled = false
    getBuiltinTemplates()
      .then((list) => {
        if (!cancelled) setBuiltin(list)
      })
      .catch((e) => {
        if (!cancelled) setError(toAppError(e))
      })
    return () => {
      cancelled = true
    }
  }, [])

  useEffect(() => {
    let stale = false
    const id = setTimeout(() => {
      previewTemplate(template)
        .then((p) => !stale && setPreview(p))
        .catch((e) => !stale && setError(toAppError(e)))
    }, 250)
    return () => {
      stale = true
      clearTimeout(id)
    }
  }, [template])

  const all = [...builtin, ...presets]
  const current = all.find((p) => same(p.template, template))
  const canDelete = !!current && !builtin.some((b) => b.name === current.name)
  const name = newName.trim()
  const canSave = name !== '' && !builtin.some((b) => b.name === name)

  const setField = (k: keyof OutputTemplate) => (v: string) =>
    updateSettingsDebounced({ template: { ...template, [k]: v } })

  return (
    <div className="flex flex-col gap-3">
      {error && <ErrorAlert error={error} />}

      <Field label="Modelo">
        <Select
          value={current?.name ?? ''}
          onChange={(e) => {
            const p = all.find((x) => x.name === e.currentTarget.value)
            if (p) updateSettings({ template: p.template })
          }}
        >
          {!current && <option value="">Personalizado</option>}
          {builtin.map((p) => (
            <option key={p.name}>{p.name}</option>
          ))}
          {presets.length > 0 && (
            <optgroup label="Meus modelos">
              {presets.map((p) => (
                <option key={p.name}>{p.name}</option>
              ))}
            </optgroup>
          )}
        </Select>
      </Field>

      <div className="flex items-end gap-2">
        <div className="flex-1">
          <Field
            label="Salvar o modelo atual como"
            hint="Mesmo nome de um modelo seu = substitui."
          >
            <Input
              value={newName}
              placeholder="Nome do modelo"
              onChange={(e) => setNewName(e.currentTarget.value)}
            />
          </Field>
        </div>
        <Button
          size="sm"
          icon={Save}
          disabled={!canSave}
          onClick={() => {
            updateSettings({
              presets: [
                ...presets.filter((p) => p.name !== name),
                { name, template },
              ],
            })
            setNewName('')
          }}
        >
          Salvar
        </Button>
        <Button
          size="sm"
          variant="ghost"
          icon={Trash2}
          disabled={!canDelete}
          onClick={() =>
            updateSettings({
              presets: presets.filter((p) => p.name !== current?.name),
            })
          }
        >
          Excluir
        </Button>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <div className="flex flex-col gap-3">
          {TEMPLATE_FIELDS.map(({ key, label, hint, multiline }) => (
            <Field
              key={key}
              label={label}
              hint={hint}
            >
              {multiline ? (
                <Textarea
                  value={template[key]}
                  onChange={(e) => setField(key)(e.currentTarget.value)}
                />
              ) : (
                <Input
                  value={template[key]}
                  onChange={(e) => setField(key)(e.currentTarget.value)}
                />
              )}
            </Field>
          ))}
        </div>
        <div className="flex min-w-0 flex-col gap-1">
          <h4 className="text-sm font-medium">Pré-visualização</h4>
          <pre
            aria-label="Pré-visualização do Markdown"
            className="max-h-112 flex-1 overflow-auto whitespace-pre-wrap rounded-sm border border-border bg-surface-muted p-2 font-mono text-xs"
          >
            {preview}
          </pre>
        </div>
      </div>
    </div>
  )
}
