import { cn } from './cn'

interface ProgressBarProps {
  current?: number
  total?: number
  label?: string
  className?: string
}

export function ProgressBar({
  current = 0,
  total,
  label,
  className,
}: ProgressBarProps) {
  const determinate = !!total && total > 0
  const pct = determinate ? Math.min(100, (current / total) * 100) : 0
  return (
    <div className={cn('flex flex-col gap-1', className)}>
      {label && <p className="text-sm text-muted">{label}</p>}
      <div
        role="progressbar"
        aria-label={label ?? 'Progresso'}
        aria-valuemin={0}
        aria-valuemax={determinate ? total : undefined}
        aria-valuenow={determinate ? current : undefined}
        className="h-1 overflow-hidden rounded-sm bg-surface-muted"
      >
        {determinate ? (
          <div
            className="h-full bg-primary transition-[width] duration-150 ease-out"
            style={{ width: `${pct}%` }}
          />
        ) : (
          <div className="h-full w-2/5 animate-[indeterminate_1.2s_ease-out_infinite] bg-primary" />
        )}
      </div>
    </div>
  )
}
