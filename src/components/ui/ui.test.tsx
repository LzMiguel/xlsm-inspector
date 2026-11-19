import { useState } from 'react'
import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { Alert } from './Alert'
import { Button } from './Button'
import { Checkbox } from './Checkbox'
import { ProgressBar } from './ProgressBar'
import { Tabs } from './Tabs'

describe('Button', () => {
  it('dispara onClick e fica inerte enquanto carrega', () => {
    const onClick = vi.fn()
    const { rerender } = render(<Button onClick={onClick}>Extrair</Button>)
    fireEvent.click(screen.getByRole('button', { name: 'Extrair' }))
    expect(onClick).toHaveBeenCalledTimes(1)

    rerender(
      <Button
        loading
        onClick={onClick}
      >
        Extrair
      </Button>,
    )
    const btn = screen.getByRole('button', { name: 'Extrair' })
    expect(btn).toBeDisabled()
    expect(btn).toHaveAttribute('aria-busy', 'true')
  })
})

describe('Checkbox', () => {
  it('informa o novo estado', () => {
    const onChange = vi.fn()
    render(
      <Checkbox
        label="Aba 1"
        checked={false}
        onChange={onChange}
      />,
    )
    fireEvent.click(screen.getByRole('checkbox', { name: 'Aba 1' }))
    expect(onChange).toHaveBeenCalledWith(true)
  })
})

describe('Tabs', () => {
  function Demo() {
    const [v, setV] = useState<'a' | 'b'>('a')
    return (
      <Tabs
        value={v}
        onChange={setV}
        tabs={[
          { id: 'a', label: 'Células' },
          { id: 'b', label: 'VBA' },
        ]}
      />
    )
  }

  it('troca com clique e com setas do teclado', () => {
    render(<Demo />)
    const cells = screen.getByRole('tab', { name: 'Células' })
    const vba = screen.getByRole('tab', { name: 'VBA' })
    expect(cells).toHaveAttribute('aria-selected', 'true')

    fireEvent.click(vba)
    expect(vba).toHaveAttribute('aria-selected', 'true')

    fireEvent.keyDown(vba, { key: 'ArrowRight' })
    expect(cells).toHaveAttribute('aria-selected', 'true')
  })
})

describe('Alert e ProgressBar', () => {
  it('danger usa role=alert', () => {
    render(
      <Alert
        variant="danger"
        title="Falha"
      >
        detalhe
      </Alert>,
    )
    expect(screen.getByRole('alert')).toHaveTextContent('Falha')
  })

  it('barra determinada expõe valores; sem total fica indeterminada', () => {
    const { rerender } = render(
      <ProgressBar
        current={2}
        total={5}
        label="Aba 2 de 5"
      />,
    )
    const bar = screen.getByRole('progressbar')
    expect(bar).toHaveAttribute('aria-valuenow', '2')
    expect(bar).toHaveAttribute('aria-valuemax', '5')

    rerender(<ProgressBar label="Carregando" />)
    expect(screen.getByRole('progressbar')).not.toHaveAttribute('aria-valuenow')
  })
})
