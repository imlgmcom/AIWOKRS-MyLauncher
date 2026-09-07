<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import { state, getCategoryById } from '@/store'
import type { Category } from '@/types'
import CategoryIcon from '@/components/CategoryIcon.vue'

const emit = defineEmits<{
  (e: 'edit-category', catId: string): void
  (e: 'add-sub-category', parentId: string | null): void
  (e: 'delete-category', catId: string): void
}>()

// 展开状态
const expandedIds = ref<Set<string>>(new Set())

// 当前一级分类：从当前选中分类向上追溯到顶级
const currentTopCategory = computed(() => {
  let cat = state.currentCategoryId ? getCategoryById(state.currentCategoryId) : undefined
  while (cat && cat.parent_id) {
    cat = getCategoryById(cat.parent_id)
  }
  return cat || null
})

// 当前一级分类的直接子分类
const subCategories = computed(() => {
  const top = currentTopCategory.value
  if (!top) return []
  return state.categories
    .filter(c => c.parent_id === top.id)
    .sort((a, b) => a.sort_order - b.sort_order)
})

// 切换一级分类时：重置展开状态，并默认展开第一层子分类
watch(currentTopCategory, (top) => {
  expandedIds.value.clear()
  if (top) {
    state.categories
      .filter(c => c.parent_id === top.id)
      .forEach(c => expandedIds.value.add(c.id))
  }
})

function getChildren(parentId: string): Category[] {
  return state.categories
    .filter(c => c.parent_id === parentId)
    .sort((a, b) => a.sort_order - b.sort_order)
}

function hasChildren(catId: string): boolean {
  return state.categories.some(c => c.parent_id === catId)
}

function toggleExpand(id: string) {
  if (expandedIds.value.has(id)) {
    expandedIds.value.delete(id)
  } else {
    expandedIds.value.add(id)
  }
}

function selectCategory(id: string) {
  state.currentCategoryId = id
}

// ─── 侧导航：鼠标按住上下拖动滚动 ───
// 二级分类过多时可拖动查看，隐藏滚动条保持美观
const sideNavRef = ref<HTMLElement | null>(null)
let dragState: { downY: number; scrollTop: number; dragging: boolean } | null = null
let suppressClick = false

function onNavPointerDown(e: PointerEvent) {
  // 只响应鼠标左键；右键菜单/箭头点击不受影响
  if (e.button !== 0) return
  const el = sideNavRef.value
  if (!el) return
  dragState = { downY: e.clientY, scrollTop: el.scrollTop, dragging: false }
}

function onNavPointerMove(e: PointerEvent) {
  const el = sideNavRef.value
  if (!el || !dragState) return
  const dy = e.clientY - dragState.downY
  // 位移超过阈值才视为拖动，避免和点击冲突
  if (!dragState.dragging && Math.abs(dy) > 4) {
    dragState.dragging = true
  }
  if (dragState.dragging) {
    el.scrollTop = dragState.scrollTop - dy
    e.preventDefault()
  }
}

function onNavPointerUp() {
  // 拖动结束后短暂抑制点击，防止拖完误触发分类切换
  if (dragState?.dragging) {
    suppressClick = true
    window.setTimeout(() => { suppressClick = false }, 50)
  }
  dragState = null
}

function onNavClick(id: string) {
  if (suppressClick) return
  selectCategory(id)
}

onMounted(() => {
  window.addEventListener('pointermove', onNavPointerMove)
  window.addEventListener('pointerup', onNavPointerUp)
})

onBeforeUnmount(() => {
  window.removeEventListener('pointermove', onNavPointerMove)
  window.removeEventListener('pointerup', onNavPointerUp)
})

// 右键菜单
const contextMenu = ref<{ x: number; y: number; catId: string } | null>(null)

function onContextMenu(e: MouseEvent, catId: string) {
  e.preventDefault()
  e.stopPropagation()
  contextMenu.value = { x: e.clientX, y: e.clientY, catId }
}

function handleContextAction(action: string) {
  if (!contextMenu.value) return
  const catId = contextMenu.value.catId
  contextMenu.value = null

  if (action === 'rename') {
    emit('edit-category', catId)
  } else if (action === 'add-sub') {
    emit('add-sub-category', catId)
  } else if (action === 'delete') {
    emit('delete-category', catId)
  }
}

// 点击外部关闭右键菜单
function closeContextMenu() {
  contextMenu.value = null
}
</script>

<template>
  <div
    ref="sideNavRef"
    class="side-nav"
    :class="{ 'side-nav-dragging': dragState?.dragging }"
    @click="closeContextMenu"
    @contextmenu.prevent
    @pointerdown="onNavPointerDown"
  >
    <!-- 子分类树：只显示当前一级分类下的子分类 -->
    <template v-for="sub in subCategories" :key="sub.id">
      <div class="tree-node">
        <div
          class="tree-item"
          :class="{ active: state.currentCategoryId === sub.id }"
          @click="onNavClick(sub.id)"
          @contextmenu="onContextMenu($event, sub.id)"
        >
          <CategoryIcon :category="sub" class="tree-icon" />
          <span class="tree-label">{{ sub.name }}</span>
          <span
            v-if="hasChildren(sub.id)"
            class="expand-arrow"
            @click.stop="toggleExpand(sub.id)"
          >{{ expandedIds.has(sub.id) ? '▼' : '▶' }}</span>
        </div>

        <!-- 孙分类（第 3 层） -->
        <div class="tree-children" v-if="expandedIds.has(sub.id)">
          <template v-for="sub2 in getChildren(sub.id)" :key="sub2.id">
            <div
              class="tree-item sub-level"
              :class="{ active: state.currentCategoryId === sub2.id }"
              :style="{ paddingLeft: (12 + 16) + 'px' }"
              @click="onNavClick(sub2.id)"
              @contextmenu="onContextMenu($event, sub2.id)"
            >
              <CategoryIcon :category="sub2" class="tree-icon" />
              <span class="tree-label">{{ sub2.name }}</span>
              <span
                v-if="hasChildren(sub2.id)"
                class="expand-arrow"
                @click.stop="toggleExpand(sub2.id)"
              >{{ expandedIds.has(sub2.id) ? '▼' : '▶' }}</span>
            </div>
            <!-- 第 4 层 -->
            <template v-if="expandedIds.has(sub2.id)">
              <template v-for="sub3 in getChildren(sub2.id)" :key="sub3.id">
                <div
                  class="tree-item sub-level"
                  :class="{ active: state.currentCategoryId === sub3.id }"
                  :style="{ paddingLeft: (12 + 32) + 'px' }"
                  @click="onNavClick(sub3.id)"
                  @contextmenu="onContextMenu($event, sub3.id)"
                >
                  <CategoryIcon :category="sub3" class="tree-icon" />
                  <span class="tree-label">{{ sub3.name }}</span>
                </div>
              </template>
            </template>
          </template>
        </div>
      </div>
    </template>

    <!-- 当前一级分类下无子分类时的提示 -->
    <div v-if="subCategories.length === 0" class="side-nav-empty">暂无子分类</div>

    <!-- 右键菜单 -->
    <div
      v-if="contextMenu"
      class="context-menu"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
      @click.stop
    >
      <div class="context-menu-item" @click="handleContextAction('add-sub')">新建子分类</div>
      <div class="context-menu-item" @click="handleContextAction('rename')">编辑</div>
      <div class="context-menu-divider"></div>
      <div class="context-menu-item danger" @click="handleContextAction('delete')">删除</div>
    </div>
  </div>
</template>

<style scoped>
.side-nav {
  padding: 10px 8px 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  /* 隐藏滚动条，靠拖动滚动 */
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.side-nav::-webkit-scrollbar {
  display: none;
}

/* 拖动中：显示抓取光标 */
.side-nav-dragging {
  cursor: grabbing;
}

/* ─── 分类树（Win11 NavigationView 风格）─── */
.tree-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  font-size: 15px;
  line-height: 1;
  flex-shrink: 0;
}

.side-nav-empty {
  padding: 20px 12px;
  text-align: center;
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.tree-node {
  display: flex;
  flex-direction: column;
}

.tree-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  margin: 3px 0;
  cursor: pointer;
  border-radius: var(--radius-md);
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text);
  letter-spacing: 0.3px;
  transition: background var(--transition);
  user-select: none;
  position: relative;
}

/* 左侧圆角竖条指示器 */
.tree-item::before {
  content: '';
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%) scaleY(0);
  width: 3px;
  height: 16px;
  border-radius: 2px;
  background: var(--color-primary);
  transition: transform 0.18s cubic-bezier(0.4, 0, 0.2, 1);
}

.tree-item:hover {
  background: var(--color-bg-hover);
  cursor: pointer;
}

.tree-item.active {
  background: var(--color-primary-bg);
  color: var(--color-primary-dark);
}

.tree-item.active::before {
  transform: translateY(-50%) scaleY(1);
}

.expand-arrow {
  width: 14px;
  text-align: center;
  font-size: 9px;
  cursor: pointer;
  color: var(--color-text-tertiary);
  flex-shrink: 0;
  margin-left: auto;
  transition: transform var(--transition);
}

.expand-arrow:hover {
  color: var(--color-text);
}

.tree-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tree-children {
  display: flex;
  flex-direction: column;
}

.sub-level {
  font-size: 12.5px;
  color: var(--color-text-secondary);
}
</style>
