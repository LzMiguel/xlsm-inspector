import type { ReactNode } from 'react'

import { cn } from './cn'

type BadgeVariant = 'neutral' | 'primary' | 'warning' | 'danger'

const VARIANTS: Record<BadgeVariant, string> = {
  neutral: 'bg-surface-muted text-muted',
  primary: 'bg-primary-soft text-primary',
  warning: 'bg-warning-soft text-warning',
  danger: 'bg-danger-soft text-danger',
}

export function Badge({
  variant = 'neutral',
  children,
  className,
}: {
  variant?: BadgeVariant
  children: ReactNode
  className?: string
}) {
  return (
    <span
      className={cn(
        'inline-flex items-center rounded-sm px-2 text-xs font-medium',
        VARIANTS[variant],
        className,
      )}
    >
      {children}
    </span>
  )
}
