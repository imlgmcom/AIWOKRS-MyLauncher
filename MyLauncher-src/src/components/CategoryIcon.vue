<script setup lang="ts">
// ════════════════════════════════════════════
// 分类图标渲染组件（emoji / 图片）
// ════════════════════════════════════════════

import { computed } from 'vue'
import type { Category } from '@/types'
import { resolveCategoryIcon } from '@/store'

const props = defineProps<{
  category?: Category | null
}>()

const icon = computed(() => resolveCategoryIcon(props.category ?? undefined))
</script>

<template>
  <span v-if="icon?.emoji" class="cat-icon-emoji">{{ icon.emoji }}</span>
  <img
    v-else-if="icon?.img"
    :src="icon.img"
    class="cat-icon-img"
    alt=""
    draggable="false"
    @error="($event.target as HTMLImageElement).style.display = 'none'"
  />
</template>

<style scoped>
.cat-icon-emoji {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: inherit;
  line-height: 1;
}

.cat-icon-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}
</style>
