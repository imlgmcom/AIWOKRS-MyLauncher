<script setup lang="ts">
import { ref, computed } from 'vue'
import { state, addCategory, addEntry, createEntryDraft } from '@/store'
import * as api from '@/api'
import type { BookmarkFolder, BookmarkLink, BookmarkParseResult, Category } from '@/types'

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'imported', count: number): void
}>()

const filePath = ref('')
const parsing = ref(false)
const parseError = ref('')
const result = ref<BookmarkParseResult | null>(null)

// 导入目标：选择现有分类或输入新分类名
const importMode = ref<'existing' | 'new'>('existing')
const targetCategoryId = ref(state.currentCategoryId || '')
const newCategoryName = ref('网址收藏')

// 导入选项
const importing = ref(false)
const importDone = ref(0)
const importTotal = ref(0)

// 树形展开状态（默认全部展开）
const expandedIds = ref<Set<string>>(new Set())

// ─── 选择状态 ───
// 用唯一路径 ID 标识每个链接："folderPath/linkIndex"
// selectedLinks 存储被选中的链接路径 ID
const selectedLinks = ref<Set<string>>(new Set())

// 分类下拉选项：所有分类（含子分类），显示层级缩进
const categoryOptions = computed(() => {
  const opts: { id: string; label: string; depth: number }[] = []
  const buildLabel = (cat: Category, depth: number) => {
    opts.push({ id: cat.id, label: cat.name, depth })
    state.categories
      .filter(c => c.parent_id === cat.id)
      .sort((a, b) => a.sort_order - b.sort_order)
      .forEach(c => buildLabel(c, depth + 1))
  }
  state.categories
    .filter(c => !c.parent_id)
    .sort((a, b) => a.sort_order - b.sort_order)
    .forEach(c => buildLabel(c, 0))
  return opts
})

async function pickFile() {
  const path = await api.pick_file([
    { name: '浏览器书签', extensions: ['html', 'htm'] },
    { name: '所有文件', extensions: ['*'] },
  ])
  if (path) {
    filePath.value = path
    await parse()
  }
}

async function parse() {
  if (!filePath.value) return
  parsing.value = true
  parseError.value = ''
  result.value = null
  try {
    const r = await api.parse_bookmarks(filePath.value)
    result.value = r
    // 默认展开全部文件夹
    expandedIds.value.clear()
    selectedLinks.value.clear()
    const walk = (f: BookmarkFolder, path: string) => {
      const id = path + '/' + f.title
      expandedIds.value.add(id)
      // 默认全选所有链接
      f.links.forEach((_, i) => {
        selectedLinks.value.add(id + '#' + i)
      })
      f.children.forEach(c => walk(c, id))
    }
    r.folders.forEach(f => walk(f, ''))
    // 根层级链接也全选
    r.root_links.forEach((_, i) => {
      selectedLinks.value.add('__root#' + i)
    })
    if (!targetCategoryId.value && state.categories.length > 0) {
      targetCategoryId.value = state.categories[0].id
    }
  } catch (e) {
    parseError.value = String(e)
  } finally {
    parsing.value = false
  }
}

// ─── 树节点数据：包含文件夹节点和链接节点 ───
interface TreeNode {
  type: 'folder' | 'link'
  id: string          // 唯一路径 ID
  title: string
  linkCount?: number  // 文件夹的总链接数
  url?: string        // 链接的 URL
  iconUrl?: string    // 链接的图标
  depth: number
  expanded?: boolean  // 文件夹是否展开
  folder?: BookmarkFolder  // 文件夹数据
  link?: BookmarkLink    // 链接数据
  selected?: boolean  // 是否选中
  partial?: boolean  // 文件夹：部分选中
  hasChildren?: boolean
}

const treeNodes = computed<TreeNode[]>(() => {
  if (!result.value) return []
  const nodes: TreeNode[] = []
  const pushFolder = (f: BookmarkFolder, path: string, depth: number) => {
    const id = path + '/' + f.title
    const count = countLinks(f)
    const selectedCount = countSelectedInFolder(f, id)
    nodes.push({
      type: 'folder',
      id,
      title: f.title,
      linkCount: count,
      depth,
      expanded: expandedIds.value.has(id),
      folder: f,
      selected: selectedCount === count && count > 0,
      partial: selectedCount > 0 && selectedCount < count,
      hasChildren: f.children.length > 0 || f.links.length > 0,
    })
    if (expandedIds.value.has(id)) {
      // 直属链接
      f.links.forEach((link, i) => {
        const lid = id + '#' + i
        nodes.push({
          type: 'link',
          id: lid,
          title: link.title,
          url: link.url,
          iconUrl: link.icon_path,
          depth: depth + 1,
          link,
          selected: selectedLinks.value.has(lid),
        })
      })
      // 子文件夹
      f.children.forEach(c => pushFolder(c, id, depth + 1))
    }
  }
  result.value.folders.forEach(f => pushFolder(f, '', 0))
  // 根层级链接
  if (result.value.root_links.length > 0) {
    result.value.root_links.forEach((link, i) => {
      const lid = '__root#' + i
      nodes.push({
        type: 'link',
        id: lid,
        title: link.title,
        url: link.url,
        iconUrl: link.icon_path,
        depth: 0,
        link,
        selected: selectedLinks.value.has(lid),
      })
    })
  }
  return nodes
})

function countLinks(f: BookmarkFolder): number {
  let n = f.links.length
  f.children.forEach(c => { n += countLinks(c) })
  return n
}

function countSelectedInFolder(f: BookmarkFolder, folderId: string): number {
  let n = 0
  f.links.forEach((_, i) => {
    if (selectedLinks.value.has(folderId + '#' + i)) n++
  })
  f.children.forEach(c => {
    const childId = folderId + '/' + c.title
    n += countSelectedInFolder(c, childId)
  })
  return n
}

function toggleExpand(id: string) {
  if (expandedIds.value.has(id)) {
    expandedIds.value.delete(id)
  } else {
    expandedIds.value.add(id)
  }
}

function folderIcon(depth: number): string {
  const icons = ['📂', '📁', '📁', '📁']
  return icons[Math.min(depth, icons.length - 1)]
}

// ─── 选择操作 ───

function toggleLinkSelection(node: TreeNode) {
  if (selectedLinks.value.has(node.id)) {
    selectedLinks.value.delete(node.id)
  } else {
    selectedLinks.value.add(node.id)
  }
}

function toggleFolderSelection(node: TreeNode) {
  if (!node.folder) return
  const folderId = node.id
  const allLinkIds = collectLinkIds(node.folder, folderId)
  const allSelected = allLinkIds.every(id => selectedLinks.value.has(id))
  if (allSelected) {
    // 全部取消
    allLinkIds.forEach(id => selectedLinks.value.delete(id))
  } else {
    // 全部选中
    allLinkIds.forEach(id => selectedLinks.value.add(id))
  }
}

function collectLinkIds(f: BookmarkFolder, folderId: string): string[] {
  const ids: string[] = []
  f.links.forEach((_, i) => ids.push(folderId + '#' + i))
  f.children.forEach(c => {
    const childId = folderId + '/' + c.title
    ids.push(...collectLinkIds(c, childId))
  })
  return ids
}

function selectAll() {
  if (!result.value) return
  selectedLinks.value.clear()
  const all = collectAllLinkIds()
  all.forEach(id => selectedLinks.value.add(id))
}

function deselectAll() {
  selectedLinks.value.clear()
}

function invertSelection() {
  if (!result.value) return
  const all = collectAllLinkIds()
  all.forEach(id => {
    if (selectedLinks.value.has(id)) {
      selectedLinks.value.delete(id)
    } else {
      selectedLinks.value.add(id)
    }
  })
}

function collectAllLinkIds(): string[] {
  if (!result.value) return []
  const ids: string[] = []
  const walk = (f: BookmarkFolder, path: string) => {
    const id = path + '/' + f.title
    f.links.forEach((_, i) => ids.push(id + '#' + i))
    f.children.forEach(c => walk(c, id))
  }
  result.value.folders.forEach(f => walk(f, ''))
  result.value.root_links.forEach((_, i) => ids.push('__root#' + i))
  return ids
}

const selectedCount = computed(() => selectedLinks.value.size)

// ─── 导入逻辑 ───

async function doImport() {
  if (!result.value) return
  importing.value = true
  importDone.value = 0
  importTotal.value = selectedLinks.value.size

  try {
    // 1. 确定根分类
    let rootCategoryId: string
    if (importMode.value === 'new') {
      const name = newCategoryName.value.trim()
      if (!name) {
        parseError.value = '请输入新分类名称'
        importing.value = false
        return
      }
      rootCategoryId = await addCategory(name, null)
    } else {
      rootCategoryId = targetCategoryId.value
      if (!rootCategoryId) {
        parseError.value = '请选择目标分类'
        importing.value = false
        return
      }
    }

    let createdCount = 0

    // 递归导入：文件夹 → 子分类，链接 → url 条目（仅导入选中的）
    const importFolder = async (folder: BookmarkFolder, folderId: string, parentCategoryId: string) => {
      // 收集该文件夹直属选中的链接
      const selectedInFolder = folder.links
        .map((link, i) => ({ link, selected: selectedLinks.value.has(folderId + '#' + i) }))
        .filter(x => x.selected)

      // 收集有选中内容的子文件夹
      const selectedChildren = folder.children.filter(c => {
        const childId = folderId + '/' + c.title
        return hasSelectedInFolder(c, childId)
      })

      // 如果没有选中的链接也没有选中的子文件夹，跳过
      if (selectedInFolder.length === 0 && selectedChildren.length === 0) return

      const categoryId = await addCategory(folder.title, parentCategoryId)
      for (const { link } of selectedInFolder) {
        await addUrlEntry(link, categoryId)
        createdCount++
        importDone.value = createdCount
      }
      for (const child of selectedChildren) {
        const childId = folderId + '/' + child.title
        await importFolder(child, childId, categoryId)
      }
    }

    // 检查文件夹下是否有选中的链接（递归）
    function hasSelectedInFolder(f: BookmarkFolder, folderId: string): boolean {
      for (let i = 0; i < f.links.length; i++) {
        if (selectedLinks.value.has(folderId + '#' + i)) return true
      }
      for (const c of f.children) {
        if (hasSelectedInFolder(c, folderId + '/' + c.title)) return true
      }
      return false
    }

    // 顶层结构处理
    const topFolders = result.value.folders.filter(f => {
      const fid = '/' + f.title
      return hasSelectedInFolder(f, fid)
    })

    // 根层级选中链接
    const rootSelectedLinks = result.value.root_links
      .map((link, i) => ({ link, selected: selectedLinks.value.has('__root#' + i) }))
      .filter(x => x.selected)

    if (topFolders.length === 1 && topFolders[0].children.length > 0) {
      // 单一非空根文件夹（如"收藏栏"）：不重复建分类，子项直接挂到目标分类下
      const bar = topFolders[0]
      const barId = '/' + bar.title
      // 直属选中链接导入根分类
      for (let i = 0; i < bar.links.length; i++) {
        if (selectedLinks.value.has(barId + '#' + i)) {
          await addUrlEntry(bar.links[i], rootCategoryId)
          createdCount++
          importDone.value = createdCount
        }
      }
      // 选中的子文件夹
      for (const child of bar.children) {
        const childId = barId + '/' + child.title
        await importFolder(child, childId, rootCategoryId)
      }
    } else if (topFolders.length === 1 && topFolders[0].children.length === 0) {
      // 单一顶层文件夹但无子文件夹：选中链接直接导入根分类
      const f = topFolders[0]
      const fid = '/' + f.title
      for (let i = 0; i < f.links.length; i++) {
        if (selectedLinks.value.has(fid + '#' + i)) {
          await addUrlEntry(f.links[i], rootCategoryId)
          createdCount++
          importDone.value = createdCount
        }
      }
    } else if (topFolders.length > 1) {
      // 多个非空顶层文件夹：每个作为根分类的子分类
      for (const f of topFolders) {
        const fid = '/' + f.title
        await importFolder(f, fid, rootCategoryId)
      }
    }

    // 根层级选中的直属链接（不属于任何文件夹）
    if (rootSelectedLinks.length > 0) {
      // 自动归入"未分类"子分类
      const uncategorizedCat = await addCategory('未分类', rootCategoryId)
      for (const { link } of rootSelectedLinks) {
        await addUrlEntry(link, uncategorizedCat)
        createdCount++
        importDone.value = createdCount
      }
    }

    emit('imported', createdCount)
    emit('close')
  } catch (e) {
    parseError.value = `导入失败: ${e}`
  } finally {
    importing.value = false
  }
}

async function addUrlEntry(link: { title: string; url: string; icon_path: string }, categoryId: string) {
  const entryData = createEntryDraft(categoryId, 'url')
  entryData.name = link.title
  entryData.url = link.url
  if (link.icon_path) {
    entryData.icon = { type: 'custom', source: link.icon_path }
  }
  await addEntry(entryData)
}

// 导入按钮的可点击条件
const canImport = computed(() => {
  if (!result.value || selectedLinks.value.size === 0) return false
  if (importMode.value === 'existing') return !!targetCategoryId.value
  return newCategoryName.value.trim().length > 0
})
</script>

<template>
  <div class="dialog-overlay" @click.self="!importing && $emit('close')">
    <div class="dialog bookmark-dialog">
      <div class="dialog-header">
        <span>导入浏览器书签</span>
        <button class="btn btn-icon" :disabled="importing" @click="$emit('close')">✕</button>
      </div>

      <div class="dialog-body">
        <!-- 步骤 1：选择文件 -->
        <div class="bm-input-row">
          <input v-model="filePath" class="input" placeholder="选择浏览器导出的书签 HTML 文件" @keyup.enter="parse" />
          <button class="btn" :disabled="parsing" @click="pickFile">浏览...</button>
          <button class="btn btn-primary" :disabled="parsing || !filePath" @click="parse">
            {{ parsing ? '解析中...' : '解析' }}
          </button>
        </div>

        <!-- 错误提示 -->
        <div v-if="parseError" class="bm-error">{{ parseError }}</div>

        <!-- 解析结果预览 -->
        <template v-if="result">
          <div class="bm-summary">
            共 <b>{{ result.total_links }}</b> 个书签、<b>{{ result.total_folders }}</b> 个文件夹
            <span class="bm-selected-count">已选 <b>{{ selectedCount }}</b></span>
          </div>

          <!-- 选择工具栏 -->
          <div class="bm-select-bar">
            <button class="bm-select-btn" @click="selectAll">全选</button>
            <button class="bm-select-btn" @click="deselectAll">全不选</button>
            <button class="bm-select-btn" @click="invertSelection">反选</button>
          </div>

          <div class="bm-tree">
            <div
              v-for="node in treeNodes"
              :key="node.id"
              class="bm-tree-node"
              :class="{ 'bm-is-folder': node.type === 'folder', 'bm-is-link': node.type === 'link' }"
              :style="{ paddingLeft: (12 + node.depth * 20) + 'px' }"
            >
              <!-- 文件夹节点 -->
              <template v-if="node.type === 'folder'">
                <span class="bm-tree-arrow" :class="{ open: node.expanded }" @click.stop="toggleExpand(node.id)">
                  {{ node.hasChildren ? (node.expanded ? '▾' : '▸') : '·' }}
                </span>
                <input
                  type="checkbox"
                  class="bm-checkbox"
                  :checked="node.selected"
                  :indeterminate.prop="node.partial"
                  @click.stop="toggleFolderSelection(node)"
                />
                <span class="bm-tree-icon">{{ folderIcon(node.depth) }}</span>
                <span class="bm-tree-title">{{ node.title }}</span>
                <span class="bm-tree-count">{{ node.linkCount }} 项</span>
              </template>
              <!-- 链接节点 -->
              <template v-else>
                <span class="bm-tree-arrow-placeholder"></span>
                <input
                  type="checkbox"
                  class="bm-checkbox"
                  :checked="node.selected"
                  @click.stop="toggleLinkSelection(node)"
                />
                <span class="bm-tree-link-icon">🔗</span>
                <span class="bm-tree-title" :title="node.url">{{ node.title }}</span>
              </template>
            </div>
            <div v-if="treeNodes.length === 0" class="bm-empty">无文件夹结构</div>
          </div>

          <!-- 导入设置 -->
          <div class="bm-import-settings">
            <label class="form-label">导入到</label>
            <div class="bm-mode-switch">
              <label class="bm-radio">
                <input type="radio" value="existing" v-model="importMode" />
                现有分类
              </label>
              <label class="bm-radio">
                <input type="radio" value="new" v-model="importMode" />
                新建分类
              </label>
            </div>
          </div>

          <div v-if="importMode === 'existing'" class="bm-import-row">
            <label class="form-label">目标分类</label>
            <select v-model="targetCategoryId" class="select">
              <option v-for="opt in categoryOptions" :key="opt.id" :value="opt.id">
                {{ '\u00A0\u00A0\u00A0'.repeat(opt.depth) }}{{ opt.label }}
              </option>
            </select>
          </div>
          <div v-else class="bm-import-row">
            <label class="form-label">分类名称</label>
            <input v-model="newCategoryName" class="input" placeholder="新分类名称" />
          </div>

          <div class="bm-hint">
            书签中的文件夹将创建为子分类，书签链接创建为网址条目。未归入文件夹的书签自动放入"未分类"子分类
          </div>
        </template>

        <!-- 空状态 -->
        <div v-else-if="!parsing && !parseError" class="empty-state">
          <div class="empty-icon">🔖</div>
          <div class="empty-text">选择浏览器导出的书签 HTML 文件</div>
          <div class="empty-sub">支持 Chrome / Edge / 360 等浏览器导出的书签文件</div>
        </div>
      </div>

      <div class="dialog-footer" v-if="result">
        <button class="btn" :disabled="importing" @click="$emit('close')">取消</button>
        <button class="btn btn-primary" :disabled="!canImport || importing" @click="doImport">
          {{ importing ? `导入中 ${importDone}/${importTotal}...` : `导入选中的 ${selectedCount} 个书签` }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.bookmark-dialog {
  width: 640px;
  max-height: 85vh;
}

.bm-input-row {
  display: flex;
  gap: 4px;
  margin-bottom: 12px;
}

.bm-input-row .input {
  flex: 1;
}

.bm-error {
  padding: 8px 12px;
  margin-bottom: 12px;
  background: var(--color-danger-bg);
  color: var(--color-danger);
  border-radius: var(--radius-sm);
  font-size: 12px;
}

.bm-summary {
  font-size: 13px;
  color: var(--color-text-secondary);
  margin-bottom: 8px;
  display: flex;
  align-items: center;
  gap: 16px;
}

.bm-summary b {
  color: var(--color-primary-dark);
}

.bm-selected-count {
  font-size: 12px;
}

.bm-select-bar {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
}

.bm-select-btn {
  padding: 3px 12px;
  font-size: 12px;
  border: 1px solid var(--color-border);
  background: var(--color-bg);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition);
}

.bm-select-btn:hover {
  background: var(--color-primary-bg);
  border-color: var(--color-primary);
  color: var(--color-primary-dark);
}

.bm-tree {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  max-height: 300px;
  overflow-y: auto;
  margin-bottom: 16px;
}

.bm-tree-node {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 12px;
  padding-right: 12px;
  font-size: 13px;
  cursor: default;
  border-bottom: 1px solid var(--color-border-light);
  transition: background var(--transition);
}

.bm-tree-node:last-child {
  border-bottom: none;
}

.bm-tree-node:hover {
  background: var(--color-bg-hover);
}

.bm-is-folder {
  cursor: pointer;
}

.bm-tree-arrow {
  width: 14px;
  font-size: 10px;
  color: var(--color-text-tertiary);
  text-align: center;
  cursor: pointer;
  flex-shrink: 0;
}

.bm-tree-arrow-placeholder {
  width: 14px;
  flex-shrink: 0;
}

.bm-checkbox {
  width: 14px;
  height: 14px;
  cursor: pointer;
  flex-shrink: 0;
  accent-color: var(--color-primary);
}

.bm-tree-icon {
  font-size: 14px;
  flex-shrink: 0;
}

.bm-tree-link-icon {
  font-size: 12px;
  flex-shrink: 0;
}

.bm-tree-title {
  flex: 1;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.bm-tree-count {
  font-size: 11px;
  color: var(--color-text-tertiary);
  background: var(--color-bg-sidebar);
  padding: 1px 8px;
  border-radius: 8px;
  flex-shrink: 0;
}

.bm-empty {
  padding: 24px;
  text-align: center;
  color: var(--color-text-tertiary);
  font-size: 12px;
}

.bm-import-settings {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.bm-mode-switch {
  display: flex;
  gap: 16px;
}

.bm-radio {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  cursor: pointer;
}

.bm-radio input {
  cursor: pointer;
}

.bm-import-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.bm-import-row .select,
.bm-import-row .input {
  flex: 1;
}

.bm-hint {
  margin-top: 10px;
  font-size: 12px;
  color: var(--color-text-tertiary);
  line-height: 1.5;
}

/* BookmarkImportDialog 的 form-label 比全局窄 */
.form-label {
  width: 60px;
}

.empty-sub {
  font-size: 12px;
  color: var(--color-text-tertiary);
  margin-top: 4px;
}
</style>
