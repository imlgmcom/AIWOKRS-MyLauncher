<script setup lang="ts">
import { computed, ref } from 'vue'
import { state, getCurrentEntries, toggleBatchMode, selectAllCurrent, invertSelection, clearSelection, deleteEntries, moveEntriesToCategory, convertEntriesPathMode, refreshEntriesIcons } from '@/store'
import BatchIconProgressDialog from '@/components/dialogs/BatchIconProgressDialog.vue'

const selectedCount = computed(() => state.selectedIds.size)
const currentEntries = getCurrentEntries()
const converting = ref(false)
const convertMsg = ref('')
/** 批量重置图标进度对话框（选中数 > 10 时启用） */
const showBatchIconDialog = ref(false)

const BATCH_ICON_THRESHOLD = 10

async function batchDelete() {
  if (selectedCount.value === 0) return
  if (!confirm(`确认删除 ${selectedCount.value} 个条目？`)) return
  const ids = Array.from(state.selectedIds)
  await deleteEntries(ids)
  state.selectedIds.clear()
}

async function batchMove(categoryId: string) {
  if (selectedCount.value === 0) return
  const ids = Array.from(state.selectedIds)
  await moveEntriesToCategory(ids, categoryId)
  state.selectedIds.clear()
}

async function batchConvertPath(targetMode: 'relative' | 'absolute') {
  if (selectedCount.value === 0) return
  converting.value = true
  convertMsg.value = ''
  const ids = Array.from(state.selectedIds)
  const result = await convertEntriesPathMode(ids, targetMode)
  showResult(`所选条目均为${targetMode === 'relative' ? '相对' : '绝对'}路径，无需转换`, result)
  converting.value = false
}

// 批量重新提取图标（重新读取目标文件/网址，提取最新图标）
// 选中数超过阈值：打开进度对话框走后端多线程（列表逐条展示成功/跳过/失败）；
// 未超阈值维持原串行逻辑（无需弹窗等待）。
async function batchRefreshIcons() {
  if (selectedCount.value === 0) return
  if (selectedCount.value > BATCH_ICON_THRESHOLD) {
    showBatchIconDialog.value = true
    return
  }
  converting.value = true
  convertMsg.value = ''
  const ids = Array.from(state.selectedIds)
  const result = await refreshEntriesIcons(ids)
  showResult('所选条目均无需重新提取', result)
  converting.value = false
}

/** 进度对话框关闭后的汇总提示 */
function onBatchIconDialogClosed() {
  showBatchIconDialog.value = false
}

/** 统一展示批量操作结果（3 秒后消失） */
function showResult(allSkippedText: string, result: { success: number; failed: number; skipped: number; messages: string[] }) {
  if (result.failed > 0) {
    convertMsg.value = `成功 ${result.success} 项，跳过 ${result.skipped} 项，失败 ${result.failed} 项：\n${result.messages.join('\n')}`
  } else if (result.success === 0 && result.skipped > 0) {
    convertMsg.value = `${allSkippedText}（${result.skipped} 项）`
  } else {
    convertMsg.value = `已处理 ${result.success} 项，跳过 ${result.skipped} 项`
  }
  setTimeout(() => { convertMsg.value = '' }, 3000)
}

function exitBatchMode() {
  toggleBatchMode()
}
</script>

<template>
  <div class="batch-toolbar">
    <div class="batch-left">
      <button class="btn" @click="selectAllCurrent(currentEntries)">全选</button>
      <button class="btn" @click="invertSelection(currentEntries)">反选</button>
      <button class="btn" @click="clearSelection">清空选择</button>
      <span class="batch-count">已选择 {{ selectedCount }} 项</span>
    </div>
    <div class="batch-right">
      <select class="batch-select" @change="batchMove(($event.target as HTMLSelectElement).value); ($event.target as HTMLSelectElement).value = ''">
        <option value="">移动到分类...</option>
        <option v-for="cat in state.categories" :key="cat.id" :value="cat.id">{{ cat.name }}</option>
      </select>
      <select class="batch-select" @change="batchConvertPath(($event.target as HTMLSelectElement).value as 'relative' | 'absolute'); ($event.target as HTMLSelectElement).value = ''" :disabled="converting || selectedCount === 0">
        <option value="">转换路径模式...</option>
        <option value="relative">转为相对路径</option>
        <option value="absolute">转为绝对路径</option>
      </select>
      <button class="btn" @click="batchRefreshIcons" :disabled="converting || selectedCount === 0">🔄 重置图标</button>
      <button class="btn btn-danger" @click="batchDelete" :disabled="selectedCount === 0">删除</button>
      <button class="btn" @click="exitBatchMode">退出批量</button>
    </div>
    <div v-if="convertMsg" class="batch-convert-msg">{{ convertMsg }}</div>

    <!-- 批量重置图标进度对话框（多线程可视化） -->
    <BatchIconProgressDialog
      v-if="showBatchIconDialog"
      @close="onBatchIconDialogClosed"
    />
  </div>
</template>

<style scoped>
.batch-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  background: var(--color-primary-bg);
  border: 1px solid var(--color-primary);
  border-radius: var(--radius-md);
  margin-bottom: 12px;
  flex-shrink: 0;
  position: relative;
  flex-wrap: wrap;
  gap: 8px;
}

.batch-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.batch-count {
  font-size: 13px;
  color: var(--color-primary-dark);
  font-weight: 600;
}

.batch-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.batch-select {
  padding: 5px 10px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-bg-card);
  font-size: 12px;
  cursor: pointer;
  outline: none;
}

.batch-convert-msg {
  width: 100%;
  font-size: 12px;
  color: var(--color-primary-dark);
  background: var(--color-bg-card);
  padding: 6px 10px;
  border-radius: var(--radius-sm);
  white-space: pre-line;
}
</style>
