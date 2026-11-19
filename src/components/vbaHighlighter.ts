import {
  createCssVariablesTheme,
  createHighlighterCore,
  type HighlighterCore,
} from 'shiki/core'
import { createJavaScriptRegexEngine } from 'shiki/engine/javascript'
import vb from 'shiki/langs/vb.mjs'

let instance: Promise<HighlighterCore> | undefined

function getHighlighter() {
  instance ??= createHighlighterCore({
    themes: [createCssVariablesTheme({ fontStyle: false })],
    langs: [vb],
    engine: createJavaScriptRegexEngine(),
  })
  return instance
}

export interface CodeToken {
  content: string
  color?: string
}

export async function highlightVba(code: string): Promise<CodeToken[][]> {
  const h = await getHighlighter()
  return h.codeToTokensBase(code, { lang: 'vb', theme: 'css-variables' })
}
