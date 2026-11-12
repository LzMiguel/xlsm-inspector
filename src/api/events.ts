import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type { Progress } from "../types/models";

/** Escuta `extraction://progress`. Chame o retorno para cancelar a escuta. */
export const onProgress = (cb: (p: Progress) => void): Promise<UnlistenFn> =>
  listen<Progress>("extraction://progress", (e) => cb(e.payload));
