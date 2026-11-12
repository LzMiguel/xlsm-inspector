import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";

import type { Progress } from "../types/models";

/** Escuta `extraction://progress`. Chame o retorno para cancelar a escuta. */
export const onProgress = (cb: (p: Progress) => void): Promise<UnlistenFn> =>
  listen<Progress>("extraction://progress", (e) => cb(e.payload));

export interface FileDragEvent {
  type: "enter" | "over" | "drop" | "leave";
  /** Preenchido em `enter` e `drop`. */
  paths: string[];
}

/** Arrastar arquivos do Explorer para a janela (a WebView não expõe o caminho via HTML5). */
export const onFileDrag = (cb: (e: FileDragEvent) => void): Promise<UnlistenFn> =>
  getCurrentWebview().onDragDropEvent((e) => {
    const p = e.payload;
    cb({ type: p.type, paths: p.type === "enter" || p.type === "drop" ? p.paths : [] });
  });
