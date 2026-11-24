import { useEffect, useId, useRef, type ReactNode } from 'react'
import { X } from 'lucide-react'

import { Button } from './Button'
import { cn } from './cn'

interface DialogProps {
  open: boolean
  onClose: () => void
  title: string
  children: ReactNode
  wide?: boolean
  preventClose?: boolean
}

export function Dialog({
  open,
  onClose,
  title,
  children,
  wide = false,
  preventClose = false,
}: DialogProps) {
  const ref = useRef<HTMLDialogElement>(null)
  const openRef = useRef(open)
  openRef.current = open
  const backdropDown = useRef(false)
  const titleId = useId()

  useEffect(() => {
    const d = ref.current
    if (!d) return
    if (open && !d.open) d.showModal()
    if (!open && d.open) d.close()
  }, [open])

  return (
    <dialog
      ref={ref}
      aria-labelledby={titleId}
      onCancel={(e) => {
        if (preventClose) e.preventDefault()
      }}
      onClose={() => {
        if (openRef.current && !preventClose) onClose()
      }}
      onMouseDown={(e) => {
        backdropDown.current = e.target === e.currentTarget
      }}
      onClick={(e) => {
        if (
          !preventClose &&
          backdropDown.current &&
          e.target === e.currentTarget
        ) {
          onClose()
        }
        backdropDown.current = false
      }}
      className={cn(
        'm-auto max-h-[90vh] w-full overflow-y-auto rounded-lg border border-border bg-surface p-0 text-text shadow-md backdrop:bg-overlay',
        wide ? 'max-w-3xl' : 'max-w-lg',
      )}
    >
      <div className="flex items-center gap-2 border-b border-border px-4 py-3">
        <h2
          id={titleId}
          className="text-lg font-semibold"
        >
          {title}
        </h2>
        <Button
          className="ml-auto"
          size="sm"
          variant="ghost"
          icon={X}
          aria-label="Fechar"
          disabled={preventClose}
          onClick={onClose}
        />
      </div>
      <div className="flex flex-col gap-6 p-4">{children}</div>
    </dialog>
  )
}
