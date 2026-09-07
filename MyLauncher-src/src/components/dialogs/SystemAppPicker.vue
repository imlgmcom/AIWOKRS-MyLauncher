<script setup lang="ts">
// ════════════════════════════════════════════
// 系统功能 / APPX 应用选择器对话框
// 系统功能：emoji 图标（与添加后的条目图标完全一致）
// APPX 应用：提取系统真实图标
// ════════════════════════════════════════════

import { ref, computed } from 'vue'
import { SYSTEM_TOOLS, type SystemTool } from '@/data/systemTools'
import type { AppxAppInfo } from '@/types'
import * as api from '@/api'
import { resolveAssetUrl } from '@/store'

/** 选择结果（交给 EntryEditor 填表） */
export interface SystemAppSelection {
  /** 显示名称 */
  name: string
  /** 启动目标：裸命令（taskmgr.exe）或 URI（ms-settings:）/ shell:AppsFolder */
  target: string
  /** 图标：emoji 字符（系统功能）或图标相对路径（APPX），空则不设置 */
  iconEmoji: string
  iconPath: string
}

const props = defineProps<{
  /** 初始选中的标签页（默认 system） */
  initialTab?: 'system' | 'appx'
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'select', payload: SystemAppSelection): void
}>()

const activeTab = ref<'system' | 'appx'>(props.initialTab || 'system')
const searchQuery = ref('')
const loading = ref(false)
const error = ref('')

// ─── 系统功能 ───

const toolGroups = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  const groups: { name: string; tools: SystemTool[] }[] = []
  const groupMap = new Map<string, SystemTool[]>()

  for (const tool of SYSTEM_TOOLS) {
    if (q && !tool.name.toLowerCase().includes(q) && !tool.desc.toLowerCase().includes(q)) {
      continue
    }
    if (!groupMap.has(tool.group)) {
      const list: SystemTool[] = []
      groupMap.set(tool.group, list)
      groups.push({ name: tool.group, tools: list })
    }
    groupMap.get(tool.group)!.push(tool)
  }
  return groups
})

/** 点击系统功能：emoji 图标直接回传，与添加后列表显示一致 */
function pickTool(tool: SystemTool) {
  emit('select', { name: tool.name, target: tool.target, iconEmoji: tool.emoji, iconPath: '' })
  emit('close')
}

// ─── APPX 应用 ───

const appxList = ref<AppxAppInfo[]>([])
const appxLoaded = ref(false)
const appxIcons = ref<Record<string, string>>({})

const filteredAppx = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  if (!q) return appxList.value
  return appxList.value.filter(a => a.name.toLowerCase().includes(q))
})

/** 首次切到 APPX 标签时懒加载 */
async function loadAppx() {
  if (appxLoaded.value || loading.value) return
  loading.value = true
  error.value = ''
  try {
    appxList.value = await api.list_appx_apps()
    appxLoaded.value = true
    // 后台逐个提取图标（有缓存，第二次秒回）
    for (const app of appxList.value) {
      extractAppxIconSilently(app)
    }
  } catch (e) {
    error.value = `枚举应用失败: ${e}`
  } finally {
    loading.value = false
  }
}

async function extractAppxIconSilently(app: AppxAppInfo) {
  if (appxIcons.value[app.app_key]) return
  try {
    const p = await api.extract_appx_icon(app.app_key, app.shell_uri)
    if (p) {
      appxIcons.value[app.app_key] = p
    }
  } catch {
    // 忽略单个图标提取失败
  }
}

function pickAppx(app: AppxAppInfo) {
  emit('select', {
    name: app.name,
    target: app.shell_uri,
    iconEmoji: '',
    iconPath: appxIcons.value[app.app_key] || '',
  })
  emit('close')
}

// ─── 标签切换 ───

function switchTab(tab: 'system' | 'appx') {
  activeTab.value = tab
  searchQuery.value = ''
  if (tab === 'appx') loadAppx()
}

// 指定从 APPX 标签打开时，立即懒加载应用列表
if (activeTab.value === 'appx') {
  loadAppx()
}
</script>

<template>
  <div class="dialog-overlay" @click.self="$emit('close')">
    <div class="dialog picker-dialog">
      <div class="dialog-header">
        <span>选择系统功能或应用</span>
        <button class="btn btn-icon" @click="$emit('close')">✕</button>
      </div>

      <div class="picker-tabs">
        <button class="picker-tab" :class="{ active: activeTab === 'system' }" @click="switchTab('system')">系统功能</button>
        <button class="picker-tab" :class="{ active: activeTab === 'appx' }" @click="switchTab('appx')">APPX 应用</button>
        <input v-model="searchQuery" class="input picker-search" placeholder="搜索..." />
      </div>

      <div v-if="error" class="error-msg">{{ error }}</div>

      <div class="picker-body">
        <!-- 加载中 -->
        <div v-if="loading" class="picker-hint">正在枚举应用，首次约需几秒...</div>

        <!-- 系统功能（emoji 图标） -->
        <template v-else-if="activeTab === 'system'">
          <div v-for="group in toolGroups" :key="group.name" class="tool-group">
            <div class="group-title">{{ group.name }}</div>
            <div class="tool-grid">
              <div
                v-for="tool in group.tools"
                :key="tool.target + tool.name"
                class="tool-item"
                :title="tool.desc"
                @click="pickTool(tool)"
              >
                <span class="tool-emoji">{{ tool.emoji }}</span>
                <span class="tool-name">{{ tool.name }}</span>
              </div>
            </div>
          </div>
          <div v-if="toolGroups.length === 0" class="picker-hint">无匹配项</div>
        </template>

        <!-- APPX 应用（真实图标） -->
        <template v-else>
          <div class="tool-grid">
            <div
              v-for="app in filteredAppx"
              :key="app.shell_uri"
              class="tool-item"
              :title="app.shell_uri"
              @click="pickAppx(app)"
            >
              <img v-if="appxIcons[app.app_key]" :src="resolveAssetUrl(appxIcons[app.app_key])" class="tool-icon" alt="" />
              <span v-else class="tool-emoji">📦</span>
              <span class="tool-name">{{ app.name }}</span>
            </div>
          </div>
          <div v-if="appxLoaded && filteredAppx.length === 0" class="picker-hint">无匹配项</div>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.picker-dialog {
  width: 640px;
  max-height: 76vh;
  display: flex;
  flex-direction: column;
}

.picker-tabs {
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 12px 16px 0;
}

.picker-tab {
  padding: 7px 18px;
  border: none;
  background: transparent;
  border-radius: var(--radius-sm) var(--radius-sm) 0 0;
  font-size: 13px;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition);
}

.picker-tab:hover {
  background: var(--color-bg-hover);
  color: var(--color-text);
}

.picker-tab.active {
  background: var(--color-primary-bg);
  color: var(--color-primary-dark);
  font-weight: 600;
}

.picker-search {
  margin-left: auto;
  width: 180px;
}

.error-msg {
  margin: 8px 16px 0;
  background: var(--color-danger-bg);
  color: var(--color-danger);
  padding: 8px 12px;
  border-radius: var(--radius-sm);
  font-size: 12px;
}

.picker-body {
  flex: 1;
  overflow-y: auto;
  padding: 12px 16px 16px;
}

.picker-hint {
  text-align: center;
  color: var(--color-text-tertiary);
  font-size: 13px;
  padding: 32px 0;
}

.tool-group {
  margin-bottom: 14px;
}

.group-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary);
  margin-bottom: 8px;
  padding-left: 2px;
}

.tool-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 8px;
}

.tool-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 10px 6px;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background var(--transition);
  text-align: center;
}

.tool-item:hover {
  background: var(--color-bg-hover);
}

.tool-icon {
  width: 36px;
  height: 36px;
  object-fit: contain;
}

.tool-emoji {
  font-size: 30px;
  line-height: 36px;
}

.tool-name {
  font-size: 12px;
  color: var(--color-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 100%;
}
</style>
