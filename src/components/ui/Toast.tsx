import { useEffect, useRef, useState } from 'react'
import { CheckCircle2, X } from 'lucide-react'

import { Button } from './Button'

interface ToastProps {
  message: string
  actionLabel?: string
  onAction?: () => void
  onClose: () => void
  duration?: number
}

export function Toast({
  message,
  actionLabel,
  onAction,
  onClose,
  duration = 8000,
}: ToastProps) {
  const closeRef = useRef(onClose)
  closeRef.current = onClose

  // parado enquanto o mouse ou o foco estão sobre ele: dá tempo de usar a ação
  const [paused, setPaused] = useState(false)

  useEffect(() => {
    if (paused) return
    const t = setTimeout(() => closeRef.current(), duration)
    return () => clearTimeout(t)
  }, [message, duration, paused])

  return (
    <div
      role="status"
      onMouseEnter={() => setPaused(true)}
      onMouseLeave={() => setPaused(false)}
      onFocus={() => setPaused(true)}
      onBlur={(e) => {
        if (!e.currentTarget.contains(e.relatedTarget as Node | null)) {
          setPaused(false)
        }
      }}
      className="fixed right-4 bottom-4 z-50 flex items-center gap-3 rounded-lg border border-border bg-surface p-3 shadow-md"
    >
      <CheckCircle2
        size={16}
        strokeWidth={1.75}
        className="text-success"
        aria-hidden
      />
      <span className="text-sm">{message}</span>
      {actionLabel && onAction && (
        <Button
          size="sm"
          variant="ghost"
          onClick={onAction}
        >
          {actionLabel}
        </Button>
      )}
      <Button
        size="sm"
        variant="ghost"
        aria-label="Fechar"
        onClick={onClose}
        icon={X}
      />
    </div>
  )
}
