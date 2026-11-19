import type { ReactNode } from 'react'
import { AlertTriangle, Info, XCircle, type LucideIcon } from 'lucide-react'

import { cn } from './cn'

export type AlertVariant = 'warning' | 'danger' | 'info'

const STYLES: Record<
  AlertVariant,
  { box: string; icon: string; Icon: LucideIcon }
> = {
  warning: {
    box: 'bg-warning-soft border-warning',
    icon: 'text-warning',
    Icon: AlertTriangle,
  },
  danger: {
    box: 'bg-danger-soft border-danger',
    icon: 'text-danger',
    Icon: XCircle,
  },
  info: {
    box: 'bg-surface-muted border-border',
    icon: 'text-accent',
    Icon: Info,
  },
}

interface AlertProps {
  variant?: AlertVariant
  title: string
  children?: ReactNode
  action?: ReactNode
  className?: string
}

export function Alert({
  variant = 'info',
  title,
  children,
  action,
  className,
}: AlertProps) {
  const { box, icon, Icon } = STYLES[variant]
  return (
    <div
      role={variant === 'info' ? 'status' : 'alert'}
      className={cn(
        'flex items-start gap-3 rounded-md border p-3',
        box,
        className,
      )}
    >
      <Icon
        size={16}
        strokeWidth={1.75}
        className={cn('mt-0.5 shrink-0', icon)}
        aria-hidden
      />
      <div className="min-w-0 flex-1">
        <p className="text-sm font-medium text-text">{title}</p>
        {children && <div className="text-sm text-muted">{children}</div>}
      </div>
      {action && <div className="shrink-0">{action}</div>}
    </div>
  )
}
