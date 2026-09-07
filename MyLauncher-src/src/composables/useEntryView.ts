// ════════════════════════════════════════════
// 四视图组件共享逻辑
// ════════════════════════════════════════════

import type { Entry } from '@/types'
import { resolveAssetUrl, state } from '@/store'

export interface EntryViewProps {
  entries: Entry[]
  pathStatus: Map<string, boolean>
  batchMode: boolean
  selectedIds: Set<string>
}

export interface EntryViewEmits {
  (e: 'launch', entry: Entry): void
  (e: 'edit', entry: Entry): void
  (e: 'toggle-select', id: string): void
  (e: 'contextmenu', entry: Entry, ev: MouseEvent): void
}

export function useEntryView(props: EntryViewProps, emit: EntryViewEmits) {
  function getIconPath(entry: Entry): string {
    if (entry.icon?.source && entry.icon.type !== 'emoji') return resolveAssetUrl(entry.icon.source)
    return ''
  }

  /** emoji 图标字符（type 为 emoji 且有 source 时返回，否则空串） */
  function getIconEmoji(entry: Entry): string {
    if (entry.icon?.type === 'emoji' && entry.icon.source) return entry.icon.source
    return ''
  }

  function getCoverPath(entry: Entry): string {
    if (entry.cover?.enabled && entry.cover.source) return resolveAssetUrl(entry.cover.source)
    return ''
  }

  function onHoverCheck(e: Event, entry: Entry) {
    e.stopPropagation()
    if (!props.batchMode) state.batchMode = true
    emit('toggle-select', entry.id)
  }

  function onContextMenu(entry: Entry, e: MouseEvent) {
    emit('contextmenu', entry, e)
  }

  function pathExists(entry: Entry): boolean {
    if (entry.type === 'url' || entry.type === 'system' || entry.type === 'appx') return true
    // 特殊启动目标（系统功能/APPX 的 URI 或裸命令名）由后端恒判有效，前端兜底不标失效
    const t = entry.relative_path?.trim() || ''
    if (t.startsWith('shell:') || t.startsWith('ms-settings:')) return true
    if (t && !t.includes('\\') && !t.includes('/') && !t.includes(':')) return true
    return props.pathStatus.get(entry.id) ?? true
  }

  function onDoubleClick(entry: Entry) {
    if (props.batchMode) emit('toggle-select', entry.id)
    else emit('launch', entry)
  }

  function hoverTip(entry: Entry): string {
    if (entry.type === 'url') return entry.url || entry.name
    if (entry.type === 'system' || entry.type === 'appx') return entry.relative_path || entry.notes || entry.name
    return entry.notes || entry.name
  }

  return { getIconPath, getIconEmoji, getCoverPath, onHoverCheck, onContextMenu, pathExists, onDoubleClick, hoverTip }
}
