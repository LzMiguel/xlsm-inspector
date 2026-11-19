import type { InputHTMLAttributes, ReactNode } from 'react'

import { cn } from './cn'

interface CheckboxProps extends Omit<
  InputHTMLAttributes<HTMLInputElement>,
  'onChange' | 'type'
> {
  label: ReactNode
  onChange: (checked: boolean) => void
  indeterminate?: boolean
}

export function Checkbox({
  label,
  onChange,
  indeterminate = false,
  className,
  ...rest
}: CheckboxProps) {
  return (
    <label
      className={cn(
        'inline-flex cursor-pointer items-center gap-2 text-sm',
        className,
      )}
    >
      <input
        type="checkbox"
        ref={(el) => {
          if (el) el.indeterminate = indeterminate
        }}
        onChange={(e) => onChange(e.currentTarget.checked)}
        className="size-4 cursor-pointer accent-primary focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
        {...rest}
      />
      {label}
    </label>
  )
}
