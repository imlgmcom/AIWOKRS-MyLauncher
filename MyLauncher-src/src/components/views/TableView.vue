<script setup lang="ts">
import type { Entry } from '@/types'
import { getCategoryById } from '@/store'
import { useEntryView, type EntryViewProps, type EntryViewEmits } from '@/composables/useEntryView'

const props = defineProps<EntryViewProps>()
const emit = defineEmits<EntryViewEmits>()
const { getIconPath, getIconEmoji, onHoverCheck, onContextMenu, pathExists, onDoubleClick, hoverTip } = useEntryView(props, emit)

function getCategoryName(catId: string): string {
  const cat = getCategoryById(catId)
  return cat?.name || '未分类'
}

function formatTime(time: string): string {
  if (!time) return '—'
  try {
    const d = new Date(time)
    const now = new Date()
    const diff = now.getTime() - d.getTime()
    if (diff < 60000) return '刚刚'
    if (diff < 3600000) return Math.floor(diff / 60000) + '分钟前'
    if (diff < 86400000) return Math.floor(diff / 3600000) + '小时前'
    if (diff < 2592000000) return Math.floor(diff / 86400000) + '天前'
    return d.toLocaleDateString('zh-CN')
  } catch {
    return time
  }
}

function onRowClick(entry: Entry) {
  if (props.batchMode) emit('toggle-select', entry.id)
}
</script>

<template>
  <div class="table-view">
    <table class="entry-table">
      <thead>
        <tr>
          <th class="col-check"></th>
          <th class="col-icon"></th>
          <th class="col-name">名称</th>
          <th class="col-category">分类</th>
          <th class="col-last-used">最后使用</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="entry in entries"
          :key="entry.id"
          class="table-row"
          :class="{
            'path-invalid': !pathExists(entry),
            'selected': batchMode && selectedIds.has(entry.id),
          }"
          :title="hoverTip(entry)"
          @dblclick="onDoubleClick(entry)"
          @click="onRowClick(entry)"
          @contextmenu="onContextMenu(entry, $event)"
        >
          <td class="col-check">
            <input
              type="checkbox"
              class="batch-checkbox"
              :class="{ 'batch-checkbox-hover': !batchMode }"
              :checked="batchMode && selectedIds.has(entry.id)"
              @click.stop="onHoverCheck($event, entry)"
            />
          </td>
          <td class="col-icon">
            <img v-if="getIconPath(entry)" :src="getIconPath(entry)" class="row-icon" alt="icon" @error="($event.target as HTMLImageElement).style.display='none'" @load="($event.target as HTMLImageElement).style.display=''" />
            <span v-else-if="getIconEmoji(entry)" class="row-icon-emoji">{{ getIconEmoji(entry) }}</span>
            <div v-else class="row-icon-placeholder">{{ entry.name.charAt(0) }}</div>
          </td>
          <td class="col-name">
            <span class="row-name" :title="entry.name">{{ entry.name }}</span>
            <span v-if="entry.run_as_admin" class="admin-mini-badge" title="管理员">🛡</span>
          </td>
          <td class="col-category">{{ getCategoryName(entry.category_id) }}</td>
          <td class="col-last-used">{{ formatTime(entry.last_used) }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.table-view {
  /* 名称列自适应剩余宽度并截断，表格总宽不超容器 → 不出现横向滚动条 */
  overflow-x: hidden;
  padding-bottom: 20px;
}

.entry-table {
  width: 100%;
  table-layout: fixed;
  border-collapse: collapse;
  font-size: 13px;
}

.entry-table th {
  text-align: left;
  padding: 8px 12px;
  background: var(--color-bg-sidebar);
  border-bottom: 1px solid var(--color-border);
  font-weight: 600;
  color: var(--color-text-secondary);
  white-space: nowrap;
}

.entry-table th:first-child {
  border-radius: var(--radius-sm) 0 0 0;
}

.entry-table th:last-child {
  border-radius: 0 var(--radius-sm) 0 0;
}

.entry-table td {
  padding: 8px 12px;
  border-bottom: 1px solid var(--color-border-light);
  white-space: nowrap;
}

.table-row {
  cursor: pointer;
  transition: background var(--transition);
}

.table-row:hover {
  background: var(--color-bg-hover);
}

.table-row.path-invalid {
  background: var(--color-danger-bg);
}

.table-row.path-invalid:hover {
  background: var(--color-path-invalid);
}

.table-row.selected {
  background: var(--color-primary-bg);
}

.col-check {
  width: 36px;
  text-align: center;
}

.batch-checkbox {
  width: 16px;
  height: 16px;
  cursor: pointer;
  vertical-align: middle;
}

/* 非多选模式下 hover 显示勾选框 */
.batch-checkbox-hover {
  opacity: 0;
  transition: opacity var(--transition);
}

.table-row:hover .batch-checkbox-hover {
  opacity: 1;
}

.table-row.selected .batch-checkbox-hover {
  opacity: 1;
}

.col-icon {
  width: 40px;
}

.row-icon {
  width: 20px;
  height: 20px;
  object-fit: contain;
}

.row-icon-emoji {
  font-size: 18px;
  line-height: 1;
}

.row-icon-placeholder {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-primary-bg);
  border-radius: 4px;
  font-size: 11px;
  color: var(--color-primary-dark);
}

.col-name {
  /* 占据剩余宽度：其他列内容固定窄，此列吃掉全部剩余空间 */
  width: auto;
}

.row-name {
  font-weight: 500;
  /* 名称过长截断省略，悬停 title 看全名 */
  display: inline-block;
  max-width: calc(100% - 20px);
  vertical-align: bottom;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.admin-mini-badge {
  margin-left: 4px;
  font-size: 11px;
}

.col-category {
  color: var(--color-text-secondary);
  /* 固定窄列：让名称列独占剩余宽度（table-layout:fixed 下未设宽的列会平分剩余空间） */
  width: 110px;
  /* 分类名过长同样截断保底（正常不会太长） */
  overflow: hidden;
  text-overflow: ellipsis;
}

.status-bad {
  color: var(--color-danger);
}

.col-last-used {
  color: var(--color-text-tertiary);
  font-size: 12px;
  /* 固定窄列：仅显示“3分钟前/2天前/日期”短文本 */
  width: 90px;
}
</style>
