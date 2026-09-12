<script setup lang="ts">
import { computed } from 'vue'
import { useEntryView, type EntryViewProps, type EntryViewEmits } from '@/composables/useEntryView'
import { state } from '@/store'

const props = defineProps<EntryViewProps>()
const emit = defineEmits<EntryViewEmits>()
const { getIconPath, getIconEmoji, getCoverPath, onHoverCheck, onContextMenu, pathExists, onDoubleClick } = useEntryView(props, emit)

/** 瀑布流封面最大高度（px），从配置读取，默认 480 */
const maxHeight = computed(() => {
  const h = state.config?.masonry_max_height
  return typeof h === 'number' && h > 0 ? h : 480
})
</script>

<template>
  <div class="waterfall-list">
    <div
      v-for="entry in entries"
      :key="entry.id"
      class="w-card"
      :class="{
        'path-invalid': !pathExists(entry),
        'selected': batchMode && selectedIds.has(entry.id),
      }"
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

      <!-- 封面图：按原图比例自适应宽度，最多限高 -->
      <div class="w-card-cover">
        <img v-if="getCoverPath(entry)" :src="getCoverPath(entry)" alt="cover" :style="{ maxHeight: maxHeight + 'px' }" @error="($event.target as HTMLImageElement).style.display='none'" @load="($event.target as HTMLImageElement).style.display=''" />
        <div v-else class="w-card-cover-fallback">
          <img v-if="getIconPath(entry)" :src="getIconPath(entry)" class="cover-icon" alt="icon" @error="($event.target as HTMLImageElement).style.display='none'" @load="($event.target as HTMLImageElement).style.display=''" />
          <span v-else-if="getIconEmoji(entry)" class="cover-emoji">{{ getIconEmoji(entry) }}</span>
          <span v-else class="cover-initial">{{ entry.name.charAt(0) }}</span>
        </div>
      </div>

      <!-- 底部信息 -->
      <div class="w-card-footer">
        <img v-if="getIconPath(entry)" :src="getIconPath(entry)" class="w-card-icon" alt="icon" @error="($event.target as HTMLImageElement).style.display='none'" @load="($event.target as HTMLImageElement).style.display=''" />
        <span v-else-if="getIconEmoji(entry)" class="w-card-icon-emoji">{{ getIconEmoji(entry) }}</span>
        <div v-else class="w-card-icon-placeholder"></div>
        <span class="w-card-name">{{ entry.name }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.waterfall-list {
  /* 分档固定列数，与竖向卡片一致（容器查询，按内容区实际宽度）：最小 2 列，最大 6 列 */
  column-count: 2;
  column-gap: 16px;
  padding-bottom: 20px;
}

@container (min-width: 860px) {
  .waterfall-list { column-count: 2; }
}
@container (min-width: 960px) {
  .waterfall-list { column-count: 3; }
}
@container (min-width: 1260px) {
  .waterfall-list { column-count: 4; }
}
@container (min-width: 1560px) {
  .waterfall-list { column-count: 5; }
}
@container (min-width: 1760px) {
  .waterfall-list { column-count: 6; }
}

.w-card {
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  overflow: hidden;
  cursor: pointer;
  transition: all var(--transition);
  position: relative;
  margin-bottom: 16px;
  break-inside: avoid;
}

.w-card:hover {
  box-shadow: 0 4px 12px var(--color-shadow-strong);
  transform: translateY(-2px);
}

.w-card.path-invalid {
  border-color: var(--color-danger);
}

.w-card.selected {
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

.batch-checkbox-hover {
  opacity: 0;
  transition: opacity var(--transition);
}

.w-card:hover .batch-checkbox-hover {
  opacity: 1;
}

.w-card.selected .batch-checkbox-hover {
  opacity: 1;
}

.admin-badge {
  position: absolute;
  top: 8px;
  right: 32px;
  font-size: 16px;
  z-index: 2;
}

.w-card-cover {
  width: 100%;
  background: var(--color-bg-sidebar);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.w-card-cover img {
  width: 100%;
  height: auto;
  object-fit: cover;
  display: block;
}

.w-card-cover-fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  min-height: 120px;
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

.w-card-footer {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
}

.w-card-icon {
  width: 20px;
  height: 20px;
  object-fit: contain;
  flex-shrink: 0;
}

.w-card-icon-emoji {
  font-size: 18px;
  line-height: 1;
  flex-shrink: 0;
}

.w-card-icon-placeholder {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

.w-card-name {
  font-size: 13px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>