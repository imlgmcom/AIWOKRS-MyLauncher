<script setup lang="ts">
// ════════════════════════════════════════════
// 图标选择器（分类 emoji 面板 + 自定义 emoji + 上传图片）
// ════════════════════════════════════════════

import { ref, computed } from 'vue'
import { pickAndCopyImage } from '@/composables/useAssetImagePicker'
import { EMOJI_LIBRARY, CUSTOM_TAB_ID } from '@/data/emojiLibrary'

const props = defineProps<{
  /** 当前图标类型 */
  currentType: string
  /** 当前图标来源 */
  currentSource: string
  /** 上传图片的目标子目录（如 'icons/custom'） */
  uploadDir: string
  /** 用户自定义 emoji 列表（从 settings 传入） */
  customEmojis?: string[]
}>()

const emit = defineEmits<{
  (e: 'select-emoji', emoji: string): void
  (e: 'upload', relPath: string): void
}>()

// 构建标签页列表：内置分类 + 自定义
const tabs = computed(() => {
  const builtIn = EMOJI_LIBRARY.map(c => ({ id: c.id, label: c.label, emojis: c.emojis }))
  if (props.customEmojis && props.customEmojis.length > 0) {
    return [...builtIn, { id: CUSTOM_TAB_ID, label: '自定义', emojis: props.customEmojis }]
  }
  return builtIn
})

const activeTab = ref(tabs.value[0]?.id ?? '')

const activeEmojis = computed(() => {
  const tab = tabs.value.find(t => t.id === activeTab.value)
  return tab ? tab.emojis : []
})

async function handleUpload() {
  const rel = await pickAndCopyImage(props.uploadDir)
  if (rel) emit('upload', rel)
}
</script>

<template>
  <div class="icon-picker">
    <!-- 分类标签页 -->
    <div class="emoji-tabs">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="emoji-tab"
        :class="{ active: activeTab === tab.id }"
        @click="activeTab = tab.id"
      >{{ tab.label }}</button>
    </div>

    <!-- emoji 面板 -->
    <div class="emoji-grid">
      <button
        v-for="emoji in activeEmojis"
        :key="emoji"
        class="emoji-btn"
        :class="{ active: currentType === 'emoji' && currentSource === emoji }"
        @click="emit('select-emoji', emoji)"
      >{{ emoji }}</button>
    </div>

    <!-- 上传图片行 -->
    <div class="icon-upload-row">
      <button class="btn" @click="handleUpload">📁 上传图片图标</button>
      <span v-if="currentType === 'custom'" class="icon-current">已使用自定义图片</span>
      <span v-else-if="currentType === 'emoji'" class="icon-current">当前：{{ currentSource }}</span>
    </div>
  </div>
</template>

<style scoped>
.emoji-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 2px;
  margin-bottom: 6px;
}

.emoji-tab {
  padding: 3px 8px;
  font-size: 11.5px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-bg-card);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition);
  white-space: nowrap;
}

.emoji-tab:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.emoji-tab.active {
  border-color: var(--color-primary);
  background: var(--color-primary-bg);
  color: var(--color-primary);
  font-weight: 500;
}

.emoji-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 3px;
  margin-bottom: 8px;
  max-height: 144px;
  overflow-y: auto;
}

.emoji-btn {
  width: 30px;
  height: 30px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-bg-card);
  font-size: 17px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--transition);
  padding: 0;
}

.emoji-btn:hover {
  border-color: var(--color-primary);
  background: var(--color-primary-bg);
}

.emoji-btn.active {
  border-color: var(--color-primary);
  background: var(--color-primary-bg);
  box-shadow: 0 0 0 2px var(--color-primary-light);
}

.icon-upload-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.icon-current {
  font-size: 12px;
  color: var(--color-text-tertiary);
}
</style>
