export function fmtDate(d: Date): string {
  const m = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  return `${d.getFullYear()}-${m}-${day}`
}

export function todayStr(): string {
  return fmtDate(new Date())
}

function daysFromToday(due: string): number {
  const [y, m, d] = due.split('-').map(Number)
  const dueMs = Date.UTC(y, m - 1, d)
  const now = new Date()
  const todayMs = Date.UTC(now.getFullYear(), now.getMonth(), now.getDate())
  return Math.round((dueMs - todayMs) / 86400000)
}

export interface DueInfo {
  label: string
  cls: 'overdue' | 'today' | 'soon' | 'later'
}

export function dueInfo(due: string): DueInfo {
  const diff = daysFromToday(due)
  if (diff < 0) {
    return { label: diff === -1 ? '逾期 1 天' : `逾期 ${-diff} 天`, cls: 'overdue' }
  }
  if (diff === 0) return { label: '今天', cls: 'today' }
  if (diff === 1) return { label: '明天', cls: 'soon' }
  if (diff <= 7) return { label: `${diff} 天后`, cls: 'soon' }
  return { label: due.slice(5).replace('-', '月') + '日', cls: 'later' }
}
