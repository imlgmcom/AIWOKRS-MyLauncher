<script setup lang="ts">
import { useEntryView, type EntryViewProps, type EntryViewEmits } from '@/composables/useEntryView'

const props = defineProps<EntryViewProps>()
const emit = defineEmits<EntryViewEmits>()
const { getIconPath, getIconEmoji, getCoverPath, onHoverCheck, onContextMenu, pathExists, onDoubleClick, hoverTip } = useEntryView(props, emit)
</script>

<template>
  <div class="vertical-card-grid">
    <div
      v-for="entry in entries"
      :key="entry.id"
      class="v-card"
      :class="{
        'path-invalid': !pathExists(entry),
        'selected': batchMode && selectedIds.has(entry.id),
      }"
      :title="hoverTip(entry)"
      @dblclick="onDoubleClick(entry)"
      @contextmenu="onContextMenu(entry, $event)"
    >
      <!-- hover 勾选框 -->
      <input
        type="checkbox"
        class="batch-checkbox"
        :class="{ 'batch-checkbox-hover': !batchMode }"
        :checked="batchMode && selectedIds.has(entry.id)"
        @click.stop="onHoverCheck($event, entry)"
      />

      <!-- 管理员标识 -->
      <div v-if="entry.run_as_admin" class="admin-badge" title="以管理员身份运行">🛡</div>

      <!-- 封面图 -->
      <div class="v-card-cover">
        <img v-if="getCoverPath(entry)" :src="getCoverPath(entry)" alt="cover" @error="($event.target as HTMLImageElement).style.display='none'" @load="($event.target as HTMLImageElement).style.display=''" />
        <div v-else class="v-card-cover-fallback">
          <img v-if="getIconPath(entry)" :src="getIconPath(entry)" class="cover-icon" alt="icon" @error="($event.target as HTMLImageElement).style.display='none'" @load="($event.target as HTMLImageElement).style.display=''" />
          <span v-else-if="getIconEmoji(entry)" class="cover-emoji">{{ getIconEmoji(entry) }}</span>
          <span v-else class="cover-initial">{{ entry.name.charAt(0) }}</span>
        </div>
      </div>

      <!-- 底部信息 -->
      <div class="v-card-footer">
        <img v-if="getIconPath(entry)" :src="getIconPath(entry)" class="v-card-icon" alt="icon" @error="($event.target as HTMLImageElement).style.display='none'" @load="($event.target as HTMLImageElement).style.display=''" />
        <span v-else-if="getIconEmoji(entry)" class="v-card-icon-emoji">{{ getIconEmoji(entry) }}</span>
        <div v-else class="v-card-icon-placeholder"></div>
        <span class="v-card-name">{{ entry.name }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.vertical-card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 16px;
  padding-bottom: 20px;
}

.v-card {
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  overflow: hidden;
  cursor: pointer;
  transition: all var(--transition);
  position: relative;
}

.v-card:hover {
  box-shadow: 0 4px 12px var(--color-shadow-strong);
  transform: translateY(-2px);
}

.v-card.path-invalid {
  border-color: var(--color-danger);
}

.v-card.selected {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px var(--color-primary);
}

.batch-checkbox {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 18px;
  height: 18px;
  cursor: pointer;
  z-index: 3;
}

/* 非多选模式下 hover 显示勾选框 */
.batch-checkbox-hover {
  opacity: 0;
  transition: opacity var(--transition);
}

.v-card:hover .batch-checkbox-hover {
  opacity: 1;
}

.v-card.selected .batch-checkbox-hover {
  opacity: 1;
}

.admin-badge {
  position: absolute;
  top: 8px;
  right: 32px;
  font-size: 16px;
  z-index: 2;
}

.v-card-cover {
  width: 100%;
  height: 140px;
  background: var(--color-bg-sidebar);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.v-card-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.v-card-cover-fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
}

.cover-icon {
  width: 48px;
  height: 48px;
  object-fit: contain;
}

.cover-emoji {
  font-size: 46px;
  line-height: 1;
}

.cover-initial {
  font-size: 36px;
  font-weight: 300;
  color: var(--color-text-tertiary);
}

.v-card-footer {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
}

.v-card-icon {
  width: 20px;
  height: 20px;
  object-fit: contain;
  flex-shrink: 0;
}

.v-card-icon-emoji {
  font-size: 18px;
  line-height: 1;
  flex-shrink: 0;
}

.v-card-icon-placeholder {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

.v-card-name {
  font-size: 13px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
