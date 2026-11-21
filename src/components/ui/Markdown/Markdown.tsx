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
  const re = /(`[^`]+`|\*\*[^*]+\*\*|https?:\/\/\S+)/g
  let last = 0
  let m: RegExpExecArray | null
  let key = 0
  while ((m = re.exec(text))) {
    if (m.index > last) parts.push(text.slice(last, m.index))
    const token = m[0]
    if (token.startsWith('`')) {
      parts.push(
        <code key={key++} className={styles.codeInline}>
          {token.slice(1, -1)}
        </code>,
      )
    } else if (token.startsWith('**')) {
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

function headingLevel(line: string): number | null {
  const m = /^(#{1,6})\s+(.+)$/.exec(line)
  if (!m) return null
  return m[1].length
}

function headingText(line: string): string {
  return line.replace(/^#{1,6}\s+/, '')
}

function listItem(line: string): { ordered: boolean; text: string } | null {
  const ul = /^[-*]\s+(.+)$/.exec(line)
  if (ul) return { ordered: false, text: ul[1] }
  const ol = /^\d+[.)]\s+(.+)$/.exec(line)
  if (ol) return { ordered: true, text: ol[1] }
  return null
}

/** GFM table separator: `| --- | :---: | ---: |` */
function isTableSeparator(line: string): boolean {
  const t = line.trim()
  if (!t.includes('-') || !t.includes('|')) return false
  return /^\|?(\s*:?-+:?\s*\|)+\s*:?-+:?\s*\|?\s*$/.test(t)
}

function isTableRow(line: string): boolean {
  const t = line.trim()
  if (!t.includes('|')) return false
  if (isTableSeparator(t)) return false
  // At least one pipe that separates cells (not a lone leading/trailing edge).
  const cells = splitTableRow(t)
  return cells.length >= 2
}

function splitTableRow(line: string): string[] {
  let t = line.trim()
  if (t.startsWith('|')) t = t.slice(1)
  if (t.endsWith('|')) t = t.slice(0, -1)
  return t.split('|').map((c) => c.trim())
}

export function Markdown({ source, compact }: MarkdownProps) {
  const raw = source.replace(/\r\n/g, '\n')
  const lines = raw.split('\n')
  const nodes: ReactNode[] = []
  let key = 0

  let i = 0
  while (i < lines.length) {
    const line = lines[i]
    const trimmed = line.trim()

    if (!trimmed) {
      i += 1
      continue
    }

    // Fenced code block
    if (trimmed.startsWith('```')) {
      const buf: string[] = []
      i += 1
      while (i < lines.length && !lines[i].trim().startsWith('```')) {
        buf.push(lines[i])
        i += 1
      }
      i += 1 // closing fence
      nodes.push(
        <pre key={key++} className={styles.codeBlock}>
          <code>{buf.join('\n')}</code>
        </pre>,
      )
      continue
    }

    const level = headingLevel(trimmed)
    if (level) {
      const text = headingText(trimmed)
      const Tag = (`h${Math.min(level, 4)}` as 'h1' | 'h2' | 'h3' | 'h4')
      nodes.push(
        <Tag key={key++} className={styles[`h${Math.min(level, 4)}` as 'h1' | 'h2' | 'h3' | 'h4']}>
          {inline(text)}
        </Tag>,
      )
      i += 1
      continue
    }

    // Blockquote / GitHub callout
    if (trimmed.startsWith('>')) {
      const buf: string[] = []
      while (i < lines.length && lines[i].trim().startsWith('>')) {
        buf.push(lines[i].trim().replace(/^>\s?/, ''))
        i += 1
      }
      const first = buf[0] ?? ''
      const callout = /^\[!(TIP|NOTE|WARNING|IMPORTANT|CAUTION)\]\s*(.*)$/i.exec(first)
      if (callout) {
        const rest = [callout[2], ...buf.slice(1)].filter(Boolean).join(' ')
        nodes.push(
          <aside key={key++} className={styles.callout} data-kind={callout[1].toLowerCase()}>
            <div className={styles.calloutLabel}>{callout[1].toUpperCase()}</div>
            <div>{inline(rest)}</div>
          </aside>,
        )
      } else {
        nodes.push(
          <blockquote key={key++} className={styles.quote}>
            {buf.map((b, idx) => (
              <p key={idx} className={styles.p}>
                {inline(b)}
              </p>
            ))}
          </blockquote>,
        )
      }
      continue
    }

    // Lists (ul / ol) — consecutive items
    const firstItem = listItem(trimmed)
    if (firstItem) {
      const ordered = firstItem.ordered
      const items: string[] = []
      while (i < lines.length) {
        const t = lines[i].trim()
        if (!t) {
          // blank inside list ends it
          break
        }
        const item = listItem(t)
        if (!item || item.ordered !== ordered) break
        items.push(item.text)
        i += 1
      }
      const ListTag = ordered ? 'ol' : 'ul'
      nodes.push(
        <ListTag key={key++} className={styles.list}>
          {items.map((item, idx) => (
            <li key={idx}>{inline(item)}</li>
          ))}
        </ListTag>,
      )
      continue
    }

    // Horizontal rule
    if (/^(-{3,}|\*{3,}|_{3,})$/.test(trimmed)) {
      nodes.push(<hr key={key++} className={styles.hr} />)
      i += 1
      continue
    }

    // GFM tables: header + separator + body rows
    if (
      isTableRow(trimmed) &&
      i + 1 < lines.length &&
      isTableSeparator(lines[i + 1].trim())
    ) {
      const header = splitTableRow(trimmed)
      i += 2 // skip header + separator
      const body: string[][] = []
      while (i < lines.length) {
        const t = lines[i].trim()
        if (!t || !isTableRow(t)) break
        body.push(splitTableRow(t))
        i += 1
      }
      const cols = header.length
      nodes.push(
        <div key={key++} className={styles.tableWrap}>
          <table className={styles.table}>
            <thead>
              <tr>
                {header.map((cell, idx) => (
                  <th key={idx}>{inline(cell)}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {body.map((row, rIdx) => (
                <tr key={rIdx}>
                  {Array.from({ length: cols }, (_, cIdx) => (
                    <td key={cIdx}>{inline(row[cIdx] ?? '')}</td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>,
      )
      continue
    }

    // Paragraph (merge soft-wrapped consecutive non-special lines)
    const para: string[] = [trimmed]
    i += 1
    while (i < lines.length) {
      const t = lines[i].trim()
      if (!t) break
      if (
        headingLevel(t) ||
        listItem(t) ||
        t.startsWith('>') ||
        t.startsWith('```') ||
        /^(-{3,}|\*{3,}|_{3,})$/.test(t) ||
        (isTableRow(t) &&
          i + 1 < lines.length &&
          isTableSeparator(lines[i + 1].trim()))
      ) {
        break
      }
      para.push(t)
      i += 1
    }
    const joined = para.join(' ')
    if (joined.toLowerCase().startsWith('full changelog') || joined.startsWith('**Full Changelog**')) {
      nodes.push(
        <p key={key++} className={styles.meta}>
          {inline(joined.replace(/\*\*/g, ''))}
        </p>,
      )
    } else {
      nodes.push(
        <p key={key++} className={styles.p}>
          {inline(joined)}
        </p>,
      )
    }
  }

  return (
    <div className={[styles.root, compact ? styles.compact : ''].join(' ')}>{nodes}</div>
  )
}

export function markdownPreview(source: string, maxLen = 140): string {
  const plain = source
    .replace(/```[\s\S]*?```/g, ' ')
    .replace(/^#{1,6}\s+/gm, '')
    .replace(/^\s*[-*]\s+/gm, '')
    .replace(/^\s*\d+[.)]\s+/gm, '')
    .replace(/^>\s?/gm, '')
    .replace(/\[!(TIP|NOTE|WARNING|IMPORTANT|CAUTION)\]/gi, '')
    .replace(/`([^`]+)`/g, '$1')
    .replace(/\*\*/g, '')
    .replace(/https?:\/\/\S+/g, '')
    .replace(/\s+/g, ' ')
    .trim()
  if (plain.length <= maxLen) return plain
  return `${plain.slice(0, maxLen).trim()}…`
}
