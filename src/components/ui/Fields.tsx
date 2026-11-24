import type {
  ComponentPropsWithRef,
  ReactNode,
  SelectHTMLAttributes,
  TextareaHTMLAttributes,
} from 'react'

import { cn } from './cn'

const BASE =
  'w-full rounded-sm border border-border bg-surface px-2 py-1.5 text-sm text-text placeholder:text-muted disabled:opacity-50 focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-accent'

export function Input({ className, ...rest }: ComponentPropsWithRef<'input'>) {
  return (
    <input
      className={cn(BASE, className)}
      {...rest}
    />
  )
}

export function Textarea({
  className,
  ...rest
}: TextareaHTMLAttributes<HTMLTextAreaElement>) {
  return (
    <textarea
      rows={3}
      className={cn(BASE, 'resize-y font-mono', className)}
      {...rest}
    />
  )
}

export function Select({
  className,
  ...rest
}: SelectHTMLAttributes<HTMLSelectElement>) {
  return (
    <select
      className={cn(BASE, className)}
      {...rest}
    />
  )
}

export function Field({
  label,
  hint,
  children,
}: {
  label: string
  hint?: string
  children: ReactNode
}) {
  return (
    <label className="flex flex-col gap-1">
      <span className="text-sm font-medium">{label}</span>
      {children}
      {hint && <span className="text-xs text-muted">{hint}</span>}
    </label>
  )
}
