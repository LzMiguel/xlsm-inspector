import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWebview } from '@tauri-apps/api/webview'

import type { Progress } from '../types/models'

export const onProgress = (cb: (p: Progress) => void): Promise<UnlistenFn> =>
  listen<Progress>('extraction://progress', (e) => cb(e.payload))

export interface FileDragEvent {
  type: 'enter' | 'over' | 'drop' | 'leave'
  paths: string[]
}

export const onFileDrag = (
  cb: (e: FileDragEvent) => void,
): Promise<UnlistenFn> =>
  getCurrentWebview().onDragDropEvent((e) => {
    const p = e.payload
    cb({
      type: p.type,
      paths: p.type === 'enter' || p.type === 'drop' ? p.paths : [],
    })
  })
