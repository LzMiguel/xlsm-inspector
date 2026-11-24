import { useEffect, useRef, type InputHTMLAttributes, type ReactNode } from 'react'

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
  disabled,
  ...rest
}: CheckboxProps) {
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (inputRef.current) inputRef.current.indeterminate = indeterminate
  }, [indeterminate])

  return (
    <label
      className={cn(
        'inline-flex items-center gap-2 text-sm',
        disabled ? 'cursor-not-allowed opacity-50' : 'cursor-pointer',
        className,
      )}
    >
      <input
        type="checkbox"
        ref={inputRef}
        disabled={disabled}
        onChange={(e) => onChange(e.currentTarget.checked)}
        className="size-4 cursor-pointer accent-primary focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:cursor-not-allowed"
        {...rest}
      />
      {label}
    </label>
  )
}
