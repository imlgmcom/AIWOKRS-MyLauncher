<script setup lang="ts">
import { ref, computed, onMounted, nextTick, onBeforeUnmount } from 'vue'
import {
  state, getTopCategories, getSubCategories,
  addCategory, updateCategory,
  setCategoryIcon, moveCategory,
  getCategoryById, applyViewModeToDescendants,
} from '@/store'
import * as api from '@/api'
import type { Category, ViewMode } from '@/types'
import CategoryEditPanel from './CategoryEditPanel.vue'
import CategoryIcon from '@/components/CategoryIcon.vue'
import DeleteCategoryDialog from './DeleteCategoryDialog.vue'

const props = defineProps<{
  initialEditId?: string
  initialNewParentId?: string | null
}>()

const editing = ref<Category | null>(null)
const isNew = ref(false)
const newParentId = ref<string | null>(null)
const form = ref({
  name: '',
  parent_id: null as string | null,
  view_mode: 'icon_grid' as ViewMode,
  // 新建分类时暂存的图标选择（保存后应用）
  icon: null as { type: 'emoji' | 'custom'; source: string } | null,
})

// 同步显示模式到所有子分类的勾选状态
const syncChildren = ref(false)

// 当前编辑面板应该插入到哪个分类行下方（顶部 + 新建一级分类时为 null）
const panelAnchorId = ref<string | null>(null)

// 递归展平分类树（所有层级，按 sort_order 排序），用于渲染
const flatCategoryTree = computed(() => {
  const result: { cat: Category; depth: number }[] = []
  const visit = (parentId: string, depth: number) => {
    state.categories
      .filter(c => c.parent_id === parentId)
      .sort((a, b) => a.sort_order - b.sort_order)
      .forEach(c => {
        result.push({ cat: c, depth })
        visit(c.id, depth + 1)
      })
  }
  // 根节点：is_top_level 或 parent_id 为 null
  const roots = state.categories
    .filter(c => c.is_top_level || c.parent_id === null)
    .sort((a, b) => a.sort_order - b.sort_order)
  roots.forEach(r => {
    result.push({ cat: r, depth: 0 })
    visit(r.id, 1)
  })
  return result
})

// 判断某个分类行下方是否应该显示编辑面板
function showPanelFor(catId: string): boolean {
  return (editing.value?.id === catId || (isNew.value && newParentId.value === catId)) && panelAnchorId.value === catId
}

// 判断是否在底部显示面板（新建一级分类、无 anchor）
const showPanelAtBottom = computed(() => {
  return (editing.value || isNew.value) && panelAnchorId.value === null
})

// 挂载时根据 props 自动进入编辑/新建模式
onMounted(() => {
  if (props.initialEditId) {
    const cat = getCategoryById(props.initialEditId)
    if (cat) startEdit(cat)
  } else if (props.initialNewParentId !== undefined) {
    startNew(props.initialNewParentId)
  }
})

onBeforeUnmount(() => {
  document.removeEventListener('mousemove', onGlobalMouseMove)
  document.removeEventListener('mouseup', onGlobalMouseUp)
})

function startNew(parentId: string | null) {
  isNew.value = true
  editing.value = null
  newParentId.value = parentId
  panelAnchorId.value = parentId
  form.value = { name: '', parent_id: parentId ?? null, view_mode: 'icon_grid', icon: null }
  syncChildren.value = false
  scrollToPanel(parentId)
}

function startEdit(cat: Category) {
  isNew.value = false
  editing.value = cat
  newParentId.value = null
  panelAnchorId.value = cat.id
  form.value = {
    name: cat.name,
    parent_id: cat.parent_id,
    view_mode: cat.view_mode,
    icon: cat.icon ? { type: cat.icon.type as 'emoji' | 'custom', source: cat.icon.source } : null,
  }
  syncChildren.value = false
  scrollToPanel(cat.id)
}

function scrollToPanel(anchorId: string | null) {
  nextTick(() => {
    const el = document.querySelector(anchorId ? `[data-cat-id="${anchorId}"]` : '[data-panel-bottom]')
    if (el) el.scrollIntoView({ behavior: 'smooth', block: 'nearest' })
  })
}

function cancelEdit() {
  editing.value = null
  isNew.value = false
  panelAnchorId.value = null
}

async function save() {
  if (!form.value.name.trim()) return
  if (isNew.value) {
    const newId = await addCategory(form.value.name.trim(), form.value.parent_id)
    // 新建时如果有暂存的图标选择，保存后应用
    if (form.value.icon) {
      await setCategoryIcon(newId, form.value.icon.type, form.value.icon.source)
    }
  } else if (editing.value) {
    const parentId = form.value.parent_id
    const isNowTopLevel = parentId === null
    // 父分类变化时，重新计算 sort_order 和 is_top_level
    const updates: Partial<Category> = {
      name: form.value.name.trim(),
      parent_id: parentId,
      view_mode: form.value.view_mode,
      is_top_level: isNowTopLevel,
    }
    if (parentId !== editing.value.parent_id) {
      // 移到新父级下，排到末尾
      const siblings = parentId
        ? getSubCategories(parentId).filter(c => c.id !== editing.value!.id)
        : getTopCategories().filter(c => c.id !== editing.value!.id)
      const maxOrder = siblings.reduce((max, c) => Math.max(max, c.sort_order), 0)
      updates.sort_order = maxOrder + 1
    }
    await updateCategory(editing.value.id, updates)
    // 勾选同步时，将所有子孙分类的显示模式一并更新
    if (syncChildren.value) {
      const count = applyViewModeToDescendants(editing.value.id, form.value.view_mode)
      if (count > 0) {
        await api.save_categories(state.categories)
      }
    }
  }
  cancelEdit()
}

async function remove(id: string) {
  deleteTargetId.value = id
}

// 删除分类对话框
const deleteTargetId = ref<string | null>(null)

function handleDeleted() {
  if (editing.value?.id === deleteTargetId.value) cancelEdit()
  deleteTargetId.value = null
}

// ─── 图标编辑 ───
// 编辑已有分类：直接应用图标；新建分类：暂存到 form.icon，保存后应用
async function setEmoji(emoji: string) {
  if (editing.value) {
    await setCategoryIcon(editing.value.id, 'emoji', emoji)
  } else {
    form.value.icon = { type: 'emoji', source: emoji }
  }
}

// 图标上传：IconPicker 已完成文件选择和复制，这里只需应用
async function uploadIcon(relPath: string) {
  if (editing.value) {
    await setCategoryIcon(editing.value.id, 'custom', relPath)
  } else {
    form.value.icon = { type: 'custom', source: relPath }
  }
}

// ─── 自定义鼠标拖拽排序 + 拖拽变子分类 ───
// 说明：HTML5 DnD API（draggable + dragstart/dragover/drop）在 WebView2 中不可靠
// （行内的 img、按钮、文本选择都会干扰），改用指针事件手动实现，稳定跨版本。
const dragId = ref<string | null>(null)
const dragOverId = ref<string | null>(null)
const dragOverAsChild = ref(false) // true=作为子分类，false=排序
let dragStartX = 0
let dragStartY = 0
let dragMoved = false
let justDragged = false

function onRowMouseDown(e: MouseEvent, catId: string) {
  // 仅左键；从按钮/表单控件上按下时不启动拖拽（避免与点击操作冲突）
  if (e.button !== 0) return
  const t = e.target as HTMLElement
  if (t.closest('button, input, select, textarea, .cat-row-actions')) return

  justDragged = false
  dragId.value = catId
  dragMoved = false
  dragStartX = e.clientX
  dragStartY = e.clientY
  document.addEventListener('mousemove', onGlobalMouseMove)
  document.addEventListener('mouseup', onGlobalMouseUp)
}

function onGlobalMouseMove(e: MouseEvent) {
  if (!dragId.value) return
  // 移动阈值，避免单击抖动
  const dx = e.clientX - dragStartX
  const dy = e.clientY - dragStartY
  if (!dragMoved && Math.abs(dx) + Math.abs(dy) < 5) return
  dragMoved = true

  // 实时计算鼠标所在行
  const el = document.elementFromPoint(e.clientX, e.clientY)
  const row = el?.closest?.('[data-cat-id]') as HTMLElement | null
  if (row) {
    const id = row.dataset.catId
    if (id && id !== dragId.value) {
      dragOverId.value = id
      // 上半部分→作为子分类，下半部分→排序插入
      const rect = row.getBoundingClientRect()
      dragOverAsChild.value = e.clientY < rect.top + rect.height * 0.5
      return
    }
  }
  dragOverId.value = null
  dragOverAsChild.value = false
}

async function onGlobalMouseUp() {
  document.removeEventListener('mousemove', onGlobalMouseMove)
  document.removeEventListener('mouseup', onGlobalMouseUp)

  const draggedId = dragId.value
  const targetId = dragOverId.value
  const asChild = dragOverAsChild.value

  dragId.value = null
  dragOverId.value = null
  dragOverAsChild.value = false

  if (!dragMoved) return // 未移动：视为普通点击
  justDragged = true
  if (!draggedId || !targetId || draggedId === targetId) return

  const target = state.categories.find(c => c.id === targetId)
  if (!target) return

  if (asChild) {
    // 作为目标分类的子分类
    await moveCategory(draggedId, targetId)
  } else {
    // 作为同层级排序：移到目标同级，排在目标后面
    const newParentId = target.parent_id
    await moveCategory(draggedId, newParentId, target.sort_order + 0.5)
    // 重新编号同层级 sort_order
    const siblings = newParentId
      ? getSubCategories(newParentId)
      : getTopCategories()
    siblings.sort((a, b) => a.sort_order - b.sort_order)
    siblings.forEach((c, i) => { c.sort_order = i + 1 })
    await api.save_categories(state.categories)
  }
}

// 点击名称编辑：拖拽刚结束时抑制 click 触发的编辑
function onCatNameClick(cat: Category) {
  if (justDragged) {
    justDragged = false
    return
  }
  startEdit(cat)
}
</script>

<template>
  <div class="dialog-overlay" @click.self="$emit('close')" @contextmenu.prevent>
    <div class="dialog cat-manager-dialog">
      <div class="dialog-header">
        <span>分类管理</span>
        <button class="btn btn-icon" @click="$emit('close')">✕</button>
      </div>

      <div class="dialog-body">
        <div class="drag-hint">提示：拖拽分类行可排序或拖到其他分类上变为子分类</div>

        <!-- 分类树：递归展平渲染所有层级 -->
        <div class="cat-tree">
          <template v-for="node in flatCategoryTree" :key="node.cat.id">
            <div
              class="cat-row"
              :class="{
                editing: editing?.id === node.cat.id,
                'drag-over': dragOverId === node.cat.id && dragOverAsChild,
                'drag-over-sort': dragOverId === node.cat.id && !dragOverAsChild,
                'dragging': dragId === node.cat.id,
                'sub-level': node.depth > 0,
              }"
              :data-cat-id="node.cat.id"
              :style="node.depth > 0 ? { marginLeft: (node.depth * 20) + 'px' } : undefined"
              @mousedown="onRowMouseDown($event, node.cat.id)"
            >
              <span class="drag-handle">⠿</span>
              <span v-if="node.depth > 0" class="cat-tree-prefix">└</span>
              <CategoryIcon :category="node.cat" class="cat-icon-slot" />
              <span class="cat-name" @click="onCatNameClick(node.cat)">{{ node.cat.name }}</span>
              <span class="cat-view-mode">{{ node.cat.view_mode }}</span>
              <div class="cat-row-actions">
                <button class="btn btn-icon" title="新建子分类" @click.stop="startNew(node.cat.id)">+</button>
                <button class="btn btn-icon btn-danger" title="删除" @click.stop="remove(node.cat.id)">🗑</button>
              </div>
            </div>
            <!-- 编辑面板：编辑/新建子分类时内联在当前分类下方 -->
            <CategoryEditPanel
              v-if="showPanelFor(node.cat.id)"
              :editing="editing"
              :is-new="isNew"
              :form="form"
              :pending-icon="form.icon"
              :categories="state.categories"
              v-model:sync-children="syncChildren"
              @save="save"
              @cancel="cancelEdit"
              @set-emoji="setEmoji"
              @upload-icon="uploadIcon"
            />
          </template>
        </div>

        <button class="btn btn-primary cat-add-top" @click="startNew(null)">+ 新建一级分类</button>

        <!-- 底部编辑面板：新建一级分类时在此显示 -->
        <div v-if="showPanelAtBottom" data-panel-bottom>
          <CategoryEditPanel
            :editing="editing"
            :is-new="isNew"
            :form="form"
            :pending-icon="form.icon"
            :categories="state.categories"
            v-model:sync-children="syncChildren"
            @save="save"
            @cancel="cancelEdit"
            @set-emoji="setEmoji"
            @upload-icon="uploadIcon"
          />
        </div>
      </div>
    </div>

    <DeleteCategoryDialog
      v-if="deleteTargetId"
      :category-id="deleteTargetId"
      @close="deleteTargetId = null"
      @deleted="handleDeleted"
    />
  </div>
</template>

<style scoped>
.cat-manager-dialog {
  width: 560px;
}

.drag-hint {
  font-size: 11px;
  color: var(--color-text-tertiary);
  margin-bottom: 8px;
  padding: 4px 8px;
  background: var(--color-bg-sidebar);
  border-radius: var(--radius-sm);
}

.cat-tree {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 12px;
}

.cat-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  transition: all 0.15s;
  user-select: none;
  -webkit-user-select: none;
  cursor: grab;
  position: relative;
}

.cat-row:active {
  cursor: grabbing;
}

.cat-row:hover {
  background: var(--color-bg-hover);
}

.cat-row.editing {
  border-color: var(--color-primary);
  background: var(--color-primary-bg);
}

.cat-row.sub-level {
  border-style: dashed;
}

.cat-tree-prefix {
  color: var(--color-text-tertiary);
  font-size: 12px;
  flex-shrink: 0;
  margin-left: -4px;
}

/* 拖拽视觉反馈 */
.cat-row.dragging {
  opacity: 0.4;
  border-style: dashed;
}

.cat-row.drag-over {
  border-color: var(--color-primary);
  border-style: solid;
  background: var(--color-primary-bg);
  box-shadow: 0 0 0 2px var(--color-primary-light);
}

.cat-row.drag-over-sort {
  border-bottom: 3px solid var(--color-primary);
}

.drag-handle {
  cursor: grab;
  color: var(--color-text-tertiary);
  font-size: 14px;
  flex-shrink: 0;
  user-select: none;
}

.drag-handle:active {
  cursor: grabbing;
}

.cat-icon-slot {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  font-size: 15px;
}

.cat-name {
  flex: 1;
  cursor: pointer;
  font-size: 13px;
  user-select: none;
  -webkit-user-select: none;
}

.cat-view-mode {
  font-size: 10px;
  color: var(--color-text-tertiary);
  background: var(--color-bg-sidebar);
  padding: 2px 6px;
  border-radius: 8px;
}

.cat-row-actions {
  display: flex;
  gap: 2px;
}

.cat-add-top {
  width: 100%;
  justify-content: center;
  margin-bottom: 16px;
}
</style>
