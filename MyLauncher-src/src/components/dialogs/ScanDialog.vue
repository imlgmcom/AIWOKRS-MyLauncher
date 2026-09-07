<script setup lang="ts">
import { ref } from 'vue'
import { state, addEntry } from '@/store'
import * as api from '@/api'
import type { ExeInfo, Entry } from '@/types'

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'scanned'): void
}>()

const scanPath = ref('')
const scanning = ref(false)
const scanResults = ref<ExeInfo[]>([])
const selectedIds = ref<Set<string>>(new Set())
const targetCategoryId = ref(state.currentCategoryId || 'cat_001')

async function pickDir() {
  const path = await api.pick_directory()
  if (path) scanPath.value = path
}

async function scan() {
  if (!scanPath.value) return
  scanning.value = true
  scanResults.value = []
  selectedIds.value.clear()
  try {
    const result = await api.scan_directory(scanPath.value, 3)
    scanResults.value = result.results
    // 默认全选
    result.results.forEach(r => {
      const id = `${r.name}_${r.absolute_path}`
      selectedIds.value.add(id)
    })
  } catch (e) {
    console.error('扫描失败', e)
  } finally {
    scanning.value = false
  }
}

function getId(r: ExeInfo): string {
  return `${r.name}_${r.absolute_path}`
}

function toggleSelect(id: string) {
  if (selectedIds.value.has(id)) {
    selectedIds.value.delete(id)
  } else {
    selectedIds.value.add(id)
  }
}

function selectAll() {
  scanResults.value.forEach(r => selectedIds.value.add(getId(r)))
}

function invertSelection() {
  scanResults.value.forEach(r => {
    const id = getId(r)
    if (selectedIds.value.has(id)) {
      selectedIds.value.delete(id)
    } else {
      selectedIds.value.add(id)
    }
  })
}

async function importSelected() {
  let count = 0
  for (const r of scanResults.value) {
    if (!selectedIds.value.has(getId(r))) continue
    const entryData: Omit<Entry, 'id' | 'add_time' | 'last_used'> = {
      name: r.name,
      type: 'program',
      category_id: targetCategoryId.value,
      relative_path: r.relative_path,
      absolute_paths: r.suggested_path_mode === 'absolute'
        ? { [state.currentEnvId]: r.absolute_path }
        : {},
      path_mode: r.suggested_path_mode as 'relative' | 'absolute',
      launch_args: '',
      working_directory: '',
      window_style: 'normal',
      run_as_admin: false,
      url: '',
      icon: r.icon_path
        ? { type: 'extracted', source: r.icon_path }
        : { type: 'default', source: '' },
      cover: { enabled: false, source: '' },
      tags: [],
      notes: '',
    }
    await addEntry(entryData)
    count++
  }
  emit('scanned')
  emit('close')
}
</script>

<template>
  <div class="dialog-overlay" @click.self="$emit('close')">
    <div class="dialog scan-dialog">
      <div class="dialog-header">
        <span>扫描文件夹</span>
        <button class="btn btn-icon" @click="$emit('close')">✕</button>
      </div>

      <div class="dialog-body">
        <!-- 扫描路径选择 -->
        <div class="scan-input-row">
          <input v-model="scanPath" class="input" placeholder="选择要扫描的文件夹" />
          <button class="btn" @click="pickDir">浏览...</button>
          <button class="btn btn-primary" @click="scan" :disabled="scanning || !scanPath">
            {{ scanning ? '扫描中...' : '扫描' }}
          </button>
        </div>

        <!-- 扫描结果 -->
        <div v-if="scanResults.length > 0" class="scan-results">
          <div class="scan-results-toolbar">
            <button class="btn" @click="selectAll">全选</button>
            <button class="btn" @click="invertSelection">反选</button>
            <span class="selected-count">已选 {{ selectedIds.size }}/{{ scanResults.length }} 项</span>
          </div>

          <div class="scan-results-list">
            <div
              v-for="r in scanResults"
              :key="getId(r)"
              class="scan-item"
              :class="{ selected: selectedIds.has(getId(r)) }"
              @click="toggleSelect(getId(r))"
            >
              <input type="checkbox" :checked="selectedIds.has(getId(r))" @click.stop="toggleSelect(getId(r))" />
              <img v-if="r.icon_path" :src="r.icon_path" class="scan-item-icon" alt="icon" @error="($event.target as HTMLImageElement).style.display='none'" />
              <div v-else class="scan-item-icon-placeholder">{{ r.name.charAt(0) }}</div>
              <div class="scan-item-info">
                <span class="scan-item-name">{{ r.name }}</span>
                <span class="scan-item-path">{{ r.absolute_path }}</span>
              </div>
              <span class="scan-item-mode">{{ r.suggested_path_mode === 'relative' ? '相对' : '绝对' }}</span>
            </div>
          </div>

          <!-- 导入目标分类 -->
          <div class="scan-import-row">
            <label class="form-label">导入到分类</label>
            <select v-model="targetCategoryId" class="select">
              <option v-for="cat in state.categories" :key="cat.id" :value="cat.id">{{ cat.name }}</option>
            </select>
          </div>
        </div>

        <!-- 空状态 -->
        <div v-else-if="!scanning" class="empty-state">
          <div class="empty-icon">📁</div>
          <div class="empty-text">选择文件夹后点击扫描</div>
        </div>
      </div>

      <div class="dialog-footer" v-if="scanResults.length > 0">
        <button class="btn" @click="$emit('close')">取消</button>
        <button class="btn btn-primary" @click="importSelected" :disabled="selectedIds.size === 0">
          导入 {{ selectedIds.size }} 项
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.scan-dialog {
  width: 640px;
  max-height: 80vh;
}

.scan-input-row {
  display: flex;
  gap: 4px;
  margin-bottom: 16px;
}

.scan-input-row .input {
  flex: 1;
}

.scan-results {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.scan-results-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
}

.selected-count {
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-left: auto;
}

.scan-results-list {
  max-height: 360px;
  overflow-y: auto;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
}

.scan-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  border-bottom: 1px solid var(--color-border-light);
  cursor: pointer;
  transition: background var(--transition);
}

.scan-item:hover {
  background: var(--color-bg-hover);
}

.scan-item.selected {
  background: var(--color-primary-bg);
}

.scan-item-icon {
  width: 24px;
  height: 24px;
  object-fit: contain;
}

.scan-item-icon-placeholder {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-bg-sidebar);
  border-radius: 4px;
  font-size: 12px;
}

.scan-item-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow: hidden;
}

.scan-item-name {
  font-size: 13px;
  font-weight: 500;
}

.scan-item-path {
  font-size: 11px;
  color: var(--color-text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.scan-item-mode {
  font-size: 10px;
  color: var(--color-primary-dark);
  background: var(--color-primary-bg);
  padding: 2px 6px;
  border-radius: 8px;
  flex-shrink: 0;
}

.scan-import-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

/* ScanDialog 的 form-label 比 70px 更宽 */
.form-label {
  width: 80px;
}

.scan-import-row .select {
  flex: 1;
}
</style>
