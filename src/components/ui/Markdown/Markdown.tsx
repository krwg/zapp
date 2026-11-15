// ╔════════════════════════════════════════════════════╗
// ║         components/ui/Markdown/Markdown          ║
// ╚════════════════════════════════════════════════════╝

import type { ReactNode } from 'react'
import styles from './Markdown.module.css'

interface MarkdownProps {
  source: string
  compact?: boolean
}

function inline(text: string): ReactNode[] {
  const parts: ReactNode[] = []
  const re = /(\*\*[^*]+\*\*|https?:\/\/\S+)/g
  let last = 0
  let m: RegExpExecArray | null
  let key = 0
  while ((m = re.exec(text))) {
    if (m.index > last) parts.push(text.slice(last, m.index))
    const token = m[0]
    if (token.startsWith('**')) {
      parts.push(<strong key={key++}>{token.slice(2, -2)}</strong>)
    } else {
      parts.push(
        <a key={key++} href={token} target="_blank" rel="noreferrer">
          {token.replace(/^https?:\/\//, '').replace(/\/$/, '')}
        </a>,
      )
    }
    last = m.index + token.length
  }
  if (last < text.length) parts.push(text.slice(last))
  return parts
}

function normalize(source: string): string[] {
  return source
    .replace(/\r\n/g, '\n')
    .split(/\n+/)
    .map((l) => l.trim())
    .filter(Boolean)
}

export function Markdown({ source, compact }: MarkdownProps) {
  const lines = normalize(source)
  const nodes: ReactNode[] = []
  let list: string[] = []
  let key = 0

  const flushList = () => {
    if (!list.length) return
    nodes.push(
      <ul key={key++} className={styles.list}>
        {list.map((item, i) => (
          <li key={i}>{inline(item)}</li>
        ))}
      </ul>,
    )
    list = []
  }

  for (const line of lines) {
    if (line.startsWith('* ') || line.startsWith('- ')) {
      list.push(line.slice(2))
      continue
    }
    flushList()
    if (line.startsWith('**Full Changelog**') || line.toLowerCase().startsWith('full changelog')) {
      nodes.push(
        <p key={key++} className={styles.meta}>
          {inline(line.replace(/\*\*/g, ''))}
        </p>,
      )
      continue
    }
    nodes.push(
      <p key={key++} className={styles.p}>
        {inline(line)}
      </p>,
    )
  }
  flushList()

  return (
    <div className={[styles.root, compact ? styles.compact : ''].join(' ')}>{nodes}</div>
  )
}

export function markdownPreview(source: string, maxLen = 140): string {
  const plain = source
    .replace(/\*\*/g, '')
    .replace(/https?:\/\/\S+/g, '')
    .replace(/\s+/g, ' ')
    .trim()
  if (plain.length <= maxLen) return plain
  return `${plain.slice(0, maxLen).trim()}…`
}
