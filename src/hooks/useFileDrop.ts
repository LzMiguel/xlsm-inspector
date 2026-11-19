import { useEffect, useRef, useState } from 'react'

import { onFileDrag } from '../api/events'

export function useFileDrop(onDrop: (path: string) => void): boolean {
  const [dragging, setDragging] = useState(false)
  const handler = useRef(onDrop)
  handler.current = onDrop

  useEffect(() => {
    let cancelled = false
    let unlisten: (() => void) | undefined

    onFileDrag((e) => {
      if (e.type === 'enter' || e.type === 'over') {
        setDragging(true)
        return
      }
      setDragging(false)
      if (e.type === 'drop' && e.paths[0]) handler.current(e.paths[0])
    })
      .then((un) => (cancelled ? un() : (unlisten = un)))
      .catch(() => {})

    return () => {
      cancelled = true
      unlisten?.()
    }
  }, [])

  return dragging
}
