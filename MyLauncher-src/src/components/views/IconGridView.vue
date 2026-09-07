<script setup lang="ts">
import { useEntryView, type EntryViewProps, type EntryViewEmits } from '@/composables/useEntryView'

const props = defineProps<EntryViewProps>()
const emit = defineEmits<EntryViewEmits>()
const { getIconPath, getIconEmoji, onHoverCheck, onContextMenu, pathExists, onDoubleClick, hoverTip } = useEntryView(props, emit)
</script>

<template>
  <div class="icon-grid">
    <div
      v-for="entry in entries"
      :key="entry.id"
      class="icon-cell"
      :class="{
        'path-invalid': !pathExists(entry),
        'selected': batchMode && selectedIds.has(entry.id),
      }"
      :title="hoverTip(entry)"
      @dblclick="onDoubleClick(entry)"
      @contextmenu="onContextMenu(entry, $event)"
    >
      <!-- hover 勾选框（始终显示在右上角，非多选时 hover 可见） -->
      <input
        type="checkbox"
        class="batch-checkbox"
        :class="{ 'batch-checkbox-hover': !batchMode }"
        :checked="batchMode && selectedIds.has(entry.id)"
        @click.stop="onHoverCheck($event, entry)"
      />

      <!-- 管理员标识 -->
      <div v-if="entry.run_as_admin" class="admin-badge" title="以管理员身份运行">🛡</div>

      <!-- 图标 -->
      <div class="icon-wrapper">
        <img
          v-if="getIconPath(entry)"
          :src="getIconPath(entry)"
          class="icon-img"
          alt="icon"
          @error="($event.target as HTMLImageElement).style.display='none'"
          @load="($event.target as HTMLImageElement).style.display=''"
        />
        <span v-else-if="getIconEmoji(entry)" class="icon-emoji">{{ getIconEmoji(entry) }}</span>
        <div v-else class="icon-fallback">
          <span>{{ entry.name.charAt(0) }}</span>
        </div>
      </div>

      <!-- 名称 -->
      <span class="icon-name">{{ entry.name }}</span>
      <span v-if="!pathExists(entry)" class="icon-status">⚠</span>
    </div>
  </div>
</template>

<style scoped>
.icon-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(96px, 1fr));
  gap: 12px;
  padding: 4px 0 20px;
}

.icon-cell {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 12px 8px;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition);
  position: relative;
  text-align: center;
}

.icon-cell:hover {
  background: var(--color-bg-hover);
}

.icon-cell.path-invalid .icon-img,
.icon-cell.path-invalid .icon-fallback {
  filter: grayscale(1) opacity(0.5);
}

.icon-cell.selected {
  background: var(--color-primary-bg);
  box-shadow: 0 0 0 2px var(--color-primary);
}

.batch-checkbox {
  position: absolute;
  top: 4px;
  right: 4px;
  width: 16px;
  height: 16px;
  cursor: pointer;
  z-index: 3;
}

/* 非多选模式下 hover 显示勾选框 */
.batch-checkbox-hover {
  opacity: 0;
  transition: opacity var(--transition);
}

.icon-cell:hover .batch-checkbox-hover {
  opacity: 1;
}

.icon-cell.selected .batch-checkbox-hover {
  opacity: 1;
}

.admin-badge {
  position: absolute;
  top: 4px;
  right: 24px;
  font-size: 12px;
  z-index: 2;
}

.icon-wrapper {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.icon-emoji {
  font-size: 38px;
  line-height: 1;
}

.icon-fallback {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-primary-bg);
  border-radius: var(--radius-sm);
  font-size: 20px;
  font-weight: 300;
  color: var(--color-primary-dark);
}

.icon-name {
  font-size: 12px;
  color: var(--color-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 100%;
}

.icon-status {
  font-size: 10px;
  color: var(--color-danger);
}
</style>
