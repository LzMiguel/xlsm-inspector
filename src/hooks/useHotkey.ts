import { useEffect, useRef } from 'react'

interface HotkeyOptions {
  enabled?: boolean
  allowInModal?: boolean
}

export function useHotkey(
  key: string,
  handler: () => void,
  options?: HotkeyOptions,
) {
  const ref = useRef(handler)
  ref.current = handler
  const enabled = options?.enabled ?? true
  const allowInModal = options?.allowInModal ?? false

  useEffect(() => {
    if (!enabled) return
    const onKey = (e: KeyboardEvent) => {
      if (!allowInModal && document.querySelector('dialog[open]')) return
      if (
        (e.ctrlKey || e.metaKey) &&
        !e.altKey &&
        !e.shiftKey &&
        e.key.toLowerCase() === key
      ) {
        e.preventDefault()
        ref.current()
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [key, enabled, allowInModal])
}
