import { FileSpreadsheet } from 'lucide-react'

import { chooseAndOpenWorkbook } from '../store/workbookActions'
import { Button } from './ui/Button'
import { cn } from './ui/cn'

interface DropZoneProps {
  dragging: boolean
  loading: boolean
  invalid: boolean
}

export function DropZone({ dragging, loading, invalid }: DropZoneProps) {
  return (
    <div
      className={cn(
        'flex flex-col items-center gap-4 rounded-lg border-2 border-dashed p-12 text-center',
        'transition-colors duration-150 ease-out',
        dragging
          ? 'border-primary bg-primary-soft'
          : invalid
            ? 'border-danger bg-surface'
            : 'border-border bg-surface',
      )}
    >
      <FileSpreadsheet
        size={48}
        strokeWidth={1.5}
        className="text-subtle"
        aria-hidden
      />
      <div>
        <h2 className="text-xl font-semibold">Arraste um .xlsx ou .xlsm</h2>
        <p className="text-base text-muted">
          ou selecione o arquivo no seu computador.
        </p>
      </div>
      <Button
        variant="primary"
        size="lg"
        loading={loading}
        onClick={chooseAndOpenWorkbook}
      >
        Selecionar arquivo
      </Button>
      <p className="max-w-md text-xs text-muted">
        Gera Markdown e JSON com células, fórmulas, VBA, Power Query e
        diagnóstico. Tudo é feito no seu computador: nada é enviado para fora.
      </p>
    </div>
  )
}
