<script setup lang="ts">
import { computed } from 'vue'
import { ref } from 'vue'
import { useDismissable } from '@/composables/useDismissable'
import { state, setTheme } from '@/store'
import type { AppConfig } from '@/types'

const props = defineProps<{
  viewMode: string
  hasCategory: boolean
  batchMode: boolean
}>()

const emit = defineEmits<{
  (e: 'switch-view', mode: string): void
  (e: 'add-entry'): void
  (e: 'manage-categories'): void
  (e: 'manage-environments'): void
  (e: 'scan-directory'): void
  (e: 'import-bookmarks'): void
  (e: 'open-settings'): void
}>()

// ─── 主题切换（明/暗/随系统三态循环） ───
// 图标用文本字符（非 emoji），与视图切换按钮字符行高一致，视觉对齐
const themeOrder: AppConfig['theme'][] = ['light', 'dark', 'system']
const themeMeta: Record<AppConfig['theme'], { icon: string; label: string; next: string }> = {
  light: { icon: '☀', label: '明亮模式', next: '切换为暗黑模式' },
  dark: { icon: '☾', label: '暗黑模式', next: '切换为随系统' },
  system: { icon: '◐', label: '随系统', next: '切换为明亮模式' },
}
const currentTheme = computed(() => {
  const t = state.config.theme
  return t === 'light' || t === 'dark' ? t : 'system'
})
const themeMetaInfo = computed(() => themeMeta[currentTheme.value])

function cycleTheme() {
  const idx = themeOrder.indexOf(currentTheme.value)
  const next = themeOrder[(idx + 1) % themeOrder.length]
  setTheme(next)
}

// 视图模式选项
const viewModes = [
  { id: 'vertical_card', icon: '▦', label: '竖向卡片' },
  { id: 'horizontal_card', icon: '▭', label: '横向卡片' },
  { id: 'icon_grid', icon: '⊞', label: '图标平铺' },
  { id: 'table', icon: '☰', label: '表格列表' },
  { id: 'waterfall', icon: '⬓', label: '瀑布流' },
]

// 菜单展开状态
const moreMenuRef = ref<HTMLElement | null>(null)
const { isOpen: moreMenuOpen, toggle: toggleMoreMenu, close: closeMoreMenu } = useDismissable(moreMenuRef)

function runMore(item: { label: string; icon: string; action: () => void }) {
  closeMoreMenu()
  item.action()
}

// 菜单项定义（设置固定在最后一项）
const moreItems = [
  { label: '添加项目', icon: '➕', action: () => emit('add-entry') },
  { label: '扫描文件夹', icon: '🗂', action: () => emit('scan-directory') },
  { label: '导入书签', icon: '🔖', action: () => emit('import-bookmarks') },
  { label: '环境管理', icon: '💻', action: () => emit('manage-environments') },
  { label: '分类管理', icon: '🗂', action: () => emit('manage-categories') },
  { label: '设置', icon: '⚙️', action: () => emit('open-settings') },
]
</script>

<template>
  <div class="footer-nav">
    <!-- 左侧菜单区（与侧导航同宽对齐） -->
    <div ref="moreMenuRef" class="footer-menu-area">
      <!-- 上拉菜单 -->
      <div v-if="moreMenuOpen" class="more-menu">
        <button
          v-for="(item, i) in moreItems"
          :key="i"
          class="more-menu-item"
          @click="runMore(item)"
        >
          <span class="menu-icon">{{ item.icon }}</span>
          <span>{{ item.label }}</span>
        </button>
      </div>
      <!-- 菜单按钮 -->
      <button class="more-btn" :class="{ open: moreMenuOpen }" @click.stop="toggleMoreMenu">
        <span class="more-btn-icon">☰</span>
        <span class="more-btn-label">菜单</span>
      </button>
    </div>

    <!-- 菜单右侧区域：视图模式切换 + 主题切换 -->
    <div class="footer-info-area">
      <div class="view-mode-switcher" v-if="props.hasCategory && !props.batchMode">
        <button
          v-for="mode in viewModes"
          :key="mode.id"
          class="view-mode-btn"
          :class="{ active: props.viewMode === mode.id }"
          :title="mode.label"
          @click="emit('switch-view', mode.id)"
        >{{ mode.icon }}</button>
      </div>

      <!-- 主题切换（明/暗/随系统循环） -->
      <button
        class="view-mode-btn theme-btn"
        :title="`${themeMetaInfo.label}（点击${themeMetaInfo.next}）`"
        @click="cycleTheme"
      >{{ themeMetaInfo.icon }}</button>
    </div>
  </div>
</template>

<style scoped>
.footer-nav {
  display: flex;
  align-items: center;
  height: 44px;
  background: var(--gradient-nav);
  border-top: 1px solid var(--color-border);
  flex-shrink: 0;
}

/* 菜单区：与侧导航同宽对齐 */
.footer-menu-area {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 180px;
  height: 100%;
  padding: 0 8px;
  box-sizing: border-box;
  border-right: 1px solid var(--color-border);
  flex-shrink: 0;
}

.more-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 12px;
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 13px;
  cursor: pointer;
  border-radius: var(--radius-md);
  transition: background var(--transition), color var(--transition);
  user-select: none;
}

.more-btn:hover,
.more-btn.open {
  background: var(--color-bg-hover);
  color: var(--color-text);
}

.more-btn-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  font-size: 14px;
  line-height: 1;
  flex-shrink: 0;
}

.more-menu {
  position: absolute;
  bottom: calc(100% + 6px);
  left: 50%;
  transform: translateX(-50%);
  z-index: 10000;
  min-width: 176px;
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: 0 -8px 24px var(--color-shadow-strong);
  padding: 4px;
  user-select: none;
  max-height: calc(100vh - 120px);
  overflow-y: auto;
  animation: moreMenuUp 0.15s cubic-bezier(0.2, 0, 0, 1);
}

@keyframes moreMenuUp {
  from {
    opacity: 0;
    transform: translateX(-50%) translateY(4px);
  }
  to {
    opacity: 1;
    transform: translateX(-50%) translateY(0);
  }
}

.more-menu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 8px 12px;
  border: none;
  background: transparent;
  font-size: 13px;
  color: var(--color-text);
  cursor: pointer;
  border-radius: 6px;
  text-align: left;
  transition: background var(--transition);
}

.more-menu-item:hover {
  background: var(--color-primary-bg);
  color: var(--color-primary-dark);
}

.menu-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  font-size: 13px;
  flex-shrink: 0;
}

/* 菜单右侧区域 */
.footer-info-area {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding: 0 20px;
  min-width: 0;
}

/* 视图模式切换 */
.view-mode-switcher {
  display: flex;
  gap: 4px;
}

.view-mode-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 26px;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-text-tertiary);
  font-size: 14px;
  cursor: pointer;
  transition: all var(--transition);
}

.view-mode-btn:hover {
  background: var(--color-bg-hover);
  color: var(--color-text);
}

.view-mode-btn.active {
  background: var(--color-primary-bg);
  border-color: rgba(0, 120, 212, 0.25);
  color: var(--color-primary-dark);
}

/* ─── 主题切换按钮 ─── */
.theme-btn {
  margin-left: 10px;
  border: 1px solid transparent;
}

.theme-btn:hover {
  background: var(--color-bg-hover);
}
</style>