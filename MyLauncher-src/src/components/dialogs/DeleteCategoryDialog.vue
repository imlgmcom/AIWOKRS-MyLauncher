<script setup lang="ts">
import { ref, computed } from 'vue'
import { state, getSubCategories, getTopCategories, getCategoryById, deleteCategory } from '@/store'
import type { Category } from '@/types'

const props = defineProps<{
  categoryId: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'deleted'): void
}>()

const category = computed(() => getCategoryById(props.categoryId))

// 收集该分类及其所有子分类的条目数
const entryCount = computed(() => {
  const ids = new Set<string>([props.categoryId])
  const collect = (pid: string) => {
    getSubCategories(pid).forEach(c => {
      ids.add(c.id)
      collect(c.id)
    })
  }
  collect(props.categoryId)
  return state.entries.filter(e => ids.has(e.category_id)).length
})

const subCategoryCount = computed(() => {
  let n = 0
  const count = (pid: string) => {
    getSubCategories(pid).forEach(c => {
      n++
      count(c.id)
    })
  }
  count(props.categoryId)
  return n
})

// 选择模式
const mode = ref<'delete' | 'move'>('move')

// 移动目标分类选项：排除自身及子分类
const moveTargetOptions = computed(() => {
  const exclude = new Set<string>([props.categoryId])
  const collect = (pid: string) => {
    getSubCategories(pid).forEach(c => {
      exclude.add(c.id)
      collect(c.id)
    })
  }
  collect(props.categoryId)

  const opts: { id: string; label: string; depth: number }[] = []
  const buildLabel = (cat: Category, depth: number) => {
    if (!exclude.has(cat.id)) {
      opts.push({ id: cat.id, label: cat.name, depth })
    }
    if (!exclude.has(cat.id)) {
      state.categories
        .filter(c => c.parent_id === cat.id && !exclude.has(c.id))
        .sort((a, b) => a.sort_order - b.sort_order)
        .forEach(c => buildLabel(c, depth + 1))
    }
  }
  getTopCategories()
    .filter(c => !exclude.has(c.id))
    .sort((a, b) => a.sort_order - b.sort_order)
    .forEach(c => buildLabel(c, 0))
  return opts
})

const moveTargetId = ref('')

// 自动选择第一个可选分类
const initMoveTarget = () => {
  if (moveTargetOptions.value.length > 0) {
    moveTargetId.value = moveTargetOptions.value[0].id
  }
}

// 在组件挂载时初始化
initMoveTarget()

const deleting = ref(false)

async function confirmDelete() {
  deleting.value = true
  try {
    if (mode.value === 'delete') {
      await deleteCategory(props.categoryId, 'delete')
    } else if (moveTargetId.value) {
      await deleteCategory(props.categoryId, 'move', moveTargetId.value)
    }
    emit('deleted')
    emit('close')
  } catch (e) {
    console.error('删除分类失败:', e)
  } finally {
    deleting.value = false
  }
}

const canConfirm = computed(() => {
  if (mode.value === 'delete') return true
  return !!moveTargetId.value
})
</script>

<template>
  <div class="dialog-overlay" @click.self="!deleting && $emit('close')">
    <div class="dialog delete-cat-dialog">
      <div class="dialog-header">
        <span>删除分类</span>
        <button class="btn btn-icon" :disabled="deleting" @click="$emit('close')">✕</button>
      </div>

      <div class="dialog-body">
        <div class="dc-info">
          <div class="dc-name">{{ category?.name || '未知分类' }}</div>
          <div class="dc-stats">
            包含 <b>{{ subCategoryCount }}</b> 个子分类、<b>{{ entryCount }}</b> 个条目
          </div>
        </div>

        <div class="dc-question">删除此分类后，其中的条目如何处理？</div>

        <div class="dc-options">
          <label class="dc-option" :class="{ active: mode === 'move' }">
            <input type="radio" value="move" v-model="mode" />
            <div class="dc-option-content">
              <div class="dc-option-title">移动到其他分类</div>
              <div class="dc-option-desc">将条目移动到选定的分类，不丢失数据</div>
            </div>
          </label>

          <div v-if="mode === 'move'" class="dc-move-target">
            <select v-model="moveTargetId" class="select">
              <option v-for="opt in moveTargetOptions" :key="opt.id" :value="opt.id">
                {{ '\u00A0\u00A0\u00A0'.repeat(opt.depth) }}{{ opt.label }}
              </option>
            </select>
          </div>

          <label class="dc-option" :class="{ active: mode === 'delete', danger: mode === 'delete' }">
            <input type="radio" value="delete" v-model="mode" />
            <div class="dc-option-content">
              <div class="dc-option-title">连同条目一起删除</div>
              <div class="dc-option-desc">永久删除所有条目，不可恢复</div>
            </div>
          </label>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn" :disabled="deleting" @click="$emit('close')">取消</button>
        <button
          class="btn"
          :class="mode === 'delete' ? 'btn-danger' : 'btn-primary'"
          :disabled="!canConfirm || deleting"
          @click="confirmDelete"
        >
          {{ deleting ? '删除中...' : (mode === 'delete' ? '删除分类及条目' : '移动并删除分类') }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.delete-cat-dialog {
  width: 440px;
  max-height: 80vh;
}

.dc-info {
  margin-bottom: 16px;
}

.dc-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: 4px;
}

.dc-stats {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.dc-stats b {
  color: var(--color-primary-dark);
}

.dc-question {
  font-size: 13px;
  color: var(--color-text);
  margin-bottom: 12px;
}

.dc-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.dc-option {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 10px 12px;
  border: 2px solid var(--color-border);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition);
}

.dc-option.active {
  border-color: var(--color-primary);
  background: var(--color-primary-bg);
}

.dc-option.danger.active {
  border-color: var(--color-danger);
  background: var(--color-danger-bg);
}

.dc-option input[type=radio] {
  margin-top: 2px;
  cursor: pointer;
  accent-color: var(--color-primary);
}

.dc-option.danger input[type=radio] {
  accent-color: var(--color-danger);
}

.dc-option-content {
  flex: 1;
}

.dc-option-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text);
}

.dc-option-desc {
  font-size: 12px;
  color: var(--color-text-tertiary);
  margin-top: 2px;
}

.dc-move-target {
  margin-left: 24px;
  margin-bottom: 8px;
}

.dc-move-target .select {
  width: 100%;
}

.btn-danger {
  background: var(--color-danger);
  color: #fff;
  border-color: var(--color-danger);
}

.btn-danger:hover:not(:disabled) {
  filter: brightness(1.1);
  border-color: var(--color-danger);
}
</style>
