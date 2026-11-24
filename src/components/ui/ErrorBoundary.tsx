import { Component, type ReactNode } from 'react'
import { AlertTriangle, RotateCcw } from 'lucide-react'

import { Button } from './Button'

interface Props {
  children: ReactNode
}

interface State {
  error: Error | null
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null }

  static getDerivedStateFromError(error: Error): State {
    return { error }
  }

  render() {
    const { error } = this.state
    if (!error) return this.props.children

    return (
      <div
        role="alert"
        className="flex h-full items-center justify-center bg-bg p-8 text-text"
      >
        <div className="flex max-w-md flex-col items-start gap-4 rounded-lg border border-border bg-surface p-6">
          <div className="flex items-center gap-2 text-danger">
            <AlertTriangle
              size={20}
              strokeWidth={1.75}
              aria-hidden
            />
            <h1 className="text-lg font-semibold">Erro inesperado</h1>
          </div>
          <p className="text-sm text-muted">
            Ocorreu uma falha ao exibir a interface. Você pode tentar recarregar
            a tela sem alterar nenhum arquivo.
          </p>
          <p className="w-full wrap-break-word rounded-sm bg-surface-muted p-2 font-mono text-xs text-muted">
            {error.message}
          </p>
          <Button
            variant="primary"
            icon={RotateCcw}
            onClick={() => this.setState({ error: null })}
          >
            Tentar novamente
          </Button>
        </div>
      </div>
    )
  }
}
