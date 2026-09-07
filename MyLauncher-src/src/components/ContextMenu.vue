<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'

export interface MenuItem {
  label: string
  icon?: string
  danger?: boolean
  action: () => void
}

const props = defineProps<{
  x: number
  y: number
  items: MenuItem[]
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const menuRef = ref<HTMLElement | null>(null)

// 点击外部关闭
function onGlobalClick(e: MouseEvent) {
  const el = menuRef.value
  if (el && !el.contains(e.target as Node)) emit('close')
}

// 右键在别处触发也关闭
function onGlobalContext(e: MouseEvent) {
  const el = menuRef.value
  if (el && !el.contains(e.target as Node)) emit('close')
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close')
}

onMounted(() => {
  window.addEventListener('click', onGlobalClick)
  window.addEventListener('contextmenu', onGlobalContext)
  window.addEventListener('keydown', onKeydown)
})

onBeforeUnmount(() => {
  window.removeEventListener('click', onGlobalClick)
  window.removeEventListener('contextmenu', onGlobalContext)
  window.removeEventListener('keydown', onKeydown)
})

// 超出视口时调整菜单位置
function adjustedStyle() {
  const menuWidth = 160
  const menuHeight = props.items.length * 36 + 8
  const winW = window.innerWidth
  const winH = window.innerHeight
  let left = props.x
  let top = props.y
  if (left + menuWidth > winW - 4) left = Math.max(4, winW - menuWidth - 4)
  if (top + menuHeight > winH - 4) top = Math.max(4, winH - menuHeight - 4)
  return { left: left + 'px', top: top + 'px' }
}

function run(item: MenuItem) {
  emit('close')
  item.action()
}
</script>

<template>
  <div ref="menuRef" class="context-menu" :style="adjustedStyle()">
    <button
      v-for="(item, i) in items"
      :key="i"
      class="context-menu-item"
      :class="{ danger: item.danger }"
      @click="run(item)"
    >
      <span v-if="item.icon" class="menu-icon">{{ item.icon }}</span>
      <span>{{ item.label }}</span>
    </button>
  </div>
</template>

<style scoped>
.context-menu {
  position: fixed;
  z-index: 10000;
  min-width: 140px;
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  box-shadow: var(--shadow-md);
  padding: 4px;
  user-select: none;
}

.context-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 7px 12px;
  border: none;
  background: transparent;
  font-size: 13px;
  color: var(--color-text);
  cursor: pointer;
  border-radius: 4px;
  text-align: left;
  transition: background var(--transition);
}

.context-menu-item:hover {
  background: var(--color-bg-hover);
}

.context-menu-item.danger {
  color: var(--color-danger);
}

.context-menu-item.danger:hover {
  background: var(--color-danger-bg);
}

.menu-icon {
  width: 16px;
  text-align: center;
  font-size: 12px;
}
</style>