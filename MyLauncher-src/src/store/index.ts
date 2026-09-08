// ════════════════════════════════════════════
// 全局状态管理 - 使用 Vue 3 reactive
// ════════════════════════════════════════════

import { reactive } from 'vue'
import type { AppConfig, Category, Entry, EntryType, Environment } from '@/types'
import * as api from '@/api'
import { convertFileSrc } from '@tauri-apps/api/core'

interface StoreState {
  loaded: boolean
  config: AppConfig
  environments: Environment[]
  categories: Category[]
  entries: Entry[]

  // exe 所在目录的绝对路径（用于拼接相对路径）
  exeDir: string
  // 当前选中的分类 ID
  currentCategoryId: string | null
  // 当前环境 ID
  currentEnvId: string
  // 搜索关键词
  searchQuery: string
  // 是否处于批量选择模式
  batchMode: boolean
  // 批量选中的条目 ID
  selectedIds: Set<string>
  // 拖拽添加的计数器
  idCounter: number
  // 图标版本号（key = 图标相对路径，value = 单调递增版本）：
  // 重置图标后文件内容更新但 URL 不变，WebView 用缓存的旧图；
  // bump 版本使 resolveAssetUrl 拼上 ?v= 参数强制重新加载
  iconVersions: Map<string, number>
}

const state = reactive<StoreState>({
  loaded: false,
  config: {
    version: '1.0.0',
    logo_icon: '🪷',
    logo_icon_type: 'emoji',
    logo_text: 'MyLauncher',
    logo_image: '',
    logo_image_enabled: false,
    close_action: 'tray',
    custom_emojis: [],
    favicon_api_sources: [
      'https://a.favicon.im/{url}?larger=true',
      'https://faviconsnap.com/api/favicon?url={url}',
      '{scheme}://{domain}/favicon.ico',
    ],
    theme: 'system',
  },
  environments: [],
  categories: [],
  entries: [],
  exeDir: '',
  currentCategoryId: null,
  currentEnvId: 'env_001',
  searchQuery: '',
  batchMode: false,
  selectedIds: new Set(),
  idCounter: 1,
  iconVersions: new Map(),
})

// ─── 初始化 ───

export async function initStore() {
  await api.init_data_dir()
  const data = await api.load_all_data()
  state.config = data.config
  state.environments = data.environments
  state.categories = data.categories
  state.entries = data.entries

  // 获取 exe 所在目录（用于拼接相对路径 → convertFileSrc）
  try {
    state.exeDir = await api.get_exe_dir()
  } catch {
    state.exeDir = ''
  }

  // 设置当前环境
  const currentEnv = state.environments.find(e => e.is_current)
  state.currentEnvId = currentEnv?.id || state.environments[0]?.id || 'env_001'

  // 设置默认选中第一个顶级分类
  const topCats = getTopCategories()
  if (topCats.length > 0) {
    state.currentCategoryId = topCats[0].id
  }

  // 更新 ID 计数器
  const maxId = state.entries.reduce((max, e) => {
    const match = e.id.match(/entry_(\d+)/)
    return match ? Math.max(max, parseInt(match[1])) : max
  }, 0)
  state.idCounter = maxId + 1

  state.loaded = true
}

// ─── Getters ───

export function getTopCategories(): Category[] {
  return state.categories
    .filter(c => c.is_top_level || c.parent_id === null)
    .sort((a, b) => a.sort_order - b.sort_order)
}

export function getSubCategories(parentId: string): Category[] {
  return state.categories
    .filter(c => c.parent_id === parentId)
    .sort((a, b) => a.sort_order - b.sort_order)
}

export function getCategoryById(id: string): Category | undefined {
  return state.categories.find(c => c.id === id)
}

// 解析分类图标：返回 { emoji: string } 或 { img: string } 或 null
export function resolveCategoryIcon(cat: Category | undefined): { emoji?: string; img?: string } | null {
  if (!cat || !cat.icon || !cat.icon.source) return null
  if (cat.icon.type === 'emoji') return { emoji: cat.icon.source }
  if (cat.icon.source.startsWith('data:')) return { img: cat.icon.source }
  // 自定义图片用 resolveAssetUrl 转换
  return { img: resolveAssetUrl(cat.icon.source) }
}

// 获取当前分类下的所有条目（递归包含子分类）
export function getCurrentEntries(): Entry[] {
  if (!state.currentCategoryId) return []

  // 收集当前分类及所有子分类 ID
  const categoryIds = new Set<string>()
  const collectChildren = (parentId: string) => {
    categoryIds.add(parentId)
    const subs = getSubCategories(parentId)
    subs.forEach(sub => collectChildren(sub.id))
  }
  collectChildren(state.currentCategoryId)

  // 如果有搜索关键词，全局搜索
  if (state.searchQuery.trim()) {
    const q = state.searchQuery.toLowerCase().trim()
    return state.entries.filter(e => {
      // 搜索时全局搜索，不限分类
      const matchName = e.name.toLowerCase().includes(q)
      const matchTags = e.tags.some(t => t.toLowerCase().includes(q))
      const matchNotes = e.notes.toLowerCase().includes(q)
      const matchUrl = e.url.toLowerCase().includes(q)
      return matchName || matchTags || matchNotes || matchUrl
    })
  }

  // 按分类 + 环境过滤
  return state.entries.filter(e => {
    if (!categoryIds.has(e.category_id)) return false

    // 环境过滤
    if (e.path_mode === 'absolute') {
      return e.absolute_paths && state.currentEnvId in e.absolute_paths
    }
    // 相对路径在所有环境都显示
    return true
  })
}

export function getCurrentCategory(): Category | undefined {
  if (!state.currentCategoryId) return undefined
  return getCategoryById(state.currentCategoryId)
}

// ─── Actions: 条目 ───

export async function addCategory(name: string, parentId: string | null): Promise<string> {
  const id = `cat_${String(state.categories.length + 1).padStart(3, '0')}_${Date.now()}`
  const maxOrder = parentId
    ? getSubCategories(parentId).reduce((max, c) => Math.max(max, c.sort_order), 0)
    : getTopCategories().reduce((max, c) => Math.max(max, c.sort_order), 0)

  const cat: Category = {
    id,
    name,
    parent_id: parentId,
    icon: { type: 'emoji', source: '📁' },
    view_mode: 'icon_grid',
    sort_order: maxOrder + 1,
    is_top_level: parentId === null,
  }
  state.categories.push(cat)
  await api.save_categories(state.categories)
  return id
}

export async function updateCategory(id: string, updates: Partial<Category>) {
  const idx = state.categories.findIndex(c => c.id === id)
  if (idx >= 0) {
    state.categories[idx] = { ...state.categories[idx], ...updates }
    await api.save_categories(state.categories)
  }
}

// 递归将所有子孙分类的 view_mode 设置为指定值，返回被修改的分类数
export function applyViewModeToDescendants(parentId: string, viewMode: Category['view_mode']): number {
  let count = 0
  const visit = (pid: string) => {
    getSubCategories(pid).forEach(c => {
      if (c.view_mode !== viewMode) {
        c.view_mode = viewMode
        count++
      }
      visit(c.id)
    })
  }
  visit(parentId)
  return count
}

export async function deleteCategory(id: string, strategy: 'orphan' | 'delete' | 'move', targetId?: string) {
  // 统一收集当前分类及所有子孙分类 ID
  const toProcess = new Set<string>([id])
  const collectChildren = (parentId: string) => {
    getSubCategories(parentId).forEach(c => {
      toProcess.add(c.id)
      collectChildren(c.id)
    })
  }
  collectChildren(id)

  if (strategy === 'delete') {
    // 删除分类和所有子分类，以及其下条目（联动清理条目资源）
    const deletedEntries = state.entries.filter(e => toProcess.has(e.category_id))
    state.entries = state.entries.filter(e => !toProcess.has(e.category_id))
    state.categories = state.categories.filter(c => !toProcess.has(c.id))
    await api.save_categories(state.categories)
    await api.save_entries(state.entries)
    await cleanupEntriesAssets(deletedEntries)
  } else if (strategy === 'move' && targetId) {
    // 移动条目到目标分类
    state.entries.forEach(e => {
      if (toProcess.has(e.category_id)) e.category_id = targetId
    })
    state.categories = state.categories.filter(c => c.id !== id)
    await api.save_categories(state.categories)
    await api.save_entries(state.entries)
  } else {
    // orphan: 条目移到未分类
    state.entries.forEach(e => {
      if (toProcess.has(e.category_id)) e.category_id = 'uncategorized'
    })
    state.categories = state.categories.filter(c => !toProcess.has(c.id))
    await api.save_categories(state.categories)
    await api.save_entries(state.entries)
  }

  // 切换到第一个分类
  const topCats = getTopCategories()
  if (topCats.length > 0) state.currentCategoryId = topCats[0].id
}

// 设置分类图标（emoji 字符或自定义图片路径）
export async function setCategoryIcon(id: string, type: 'emoji' | 'custom', source: string) {
  const cat = state.categories.find(c => c.id === id)
  if (cat) {
    cat.icon = { type, source }
    await api.save_categories(state.categories)
  }
}

// 移动分类到新父分类下（拖拽改变层级）
export async function moveCategory(catId: string, newParentId: string | null, newSortOrder?: number) {
  const cat = state.categories.find(c => c.id === catId)
  if (!cat) return
  // 防止移到自己的子分类下（循环）
  if (newParentId) {
    let p: string | null = newParentId
    while (p) {
      if (p === catId) return // 不能移到自己或自己的子孙下
      const parent = state.categories.find(c => c.id === p)
      p = parent?.parent_id ?? null
    }
  }
  cat.parent_id = newParentId
  cat.is_top_level = newParentId === null
  // sort_order
  const siblings = newParentId
    ? getSubCategories(newParentId)
    : getTopCategories()
  if (newSortOrder !== undefined) {
    cat.sort_order = newSortOrder
  } else {
    const maxOrder = siblings.reduce((max, c) => Math.max(max, c.sort_order), 0)
    cat.sort_order = maxOrder + 1
  }
  await api.save_categories(state.categories)
}

// ─── Actions: 条目 ───

export async function addEntry(entry: Omit<Entry, 'id' | 'add_time' | 'last_used'>): Promise<string> {
  const id = `entry_${state.idCounter++}`
  const now = await api.get_now()
  const newEntry: Entry = {
    ...entry,
    id,
    add_time: now,
    last_used: '',
  }
  state.entries.push(newEntry)
  await api.save_entries(state.entries)
  return id
}

export async function updateEntry(id: string, updates: Partial<Entry>) {
  const idx = state.entries.findIndex(e => e.id === id)
  if (idx >= 0) {
    state.entries[idx] = { ...state.entries[idx], ...updates }
    await api.save_entries(state.entries)
  }
}

// 收集条目的资源文件引用（图标 + 封面，assets 相对路径），供删除后联动清理
function collectEntryAssetSources(entries: Entry[]): string[] {
  const sources: string[] = []
  for (const e of entries) {
    if (e.icon?.source) sources.push(e.icon.source)
    if (e.cover?.source) sources.push(e.cover.source)
  }
  return sources
}

// 删除条目后联动清理不再被引用的图标/封面文件（后端移入回收站，可还原）；
// domain 缓存图标仍被其他链接共用时由后端引用判断自动跳过。
// 清理失败不影响删除主流程（仅打印警告）。
async function cleanupEntriesAssets(deletedEntries: Entry[]) {
  const sources = collectEntryAssetSources(deletedEntries)
  if (sources.length === 0) return
  try {
    await api.cleanup_deleted_assets(sources)
  } catch (e) {
    console.warn('删除后资源清理失败:', e)
  }
}

export async function deleteEntries(ids: string[]) {
  const idSet = new Set(ids)
  // 先收集被删条目（含资源引用），再过滤删除
  const deletedEntries = state.entries.filter(e => idSet.has(e.id))
  state.entries = state.entries.filter(e => !idSet.has(e.id))
  await api.save_entries(state.entries)
  // 联动清理不再被引用的图标/封面
  await cleanupEntriesAssets(deletedEntries)
}

export async function moveEntriesToCategory(ids: string[], categoryId: string) {
  const idSet = new Set(ids)
  state.entries.forEach(e => {
    if (idSet.has(e.id)) e.category_id = categoryId
  })
  await api.save_entries(state.entries)
}

// 批量转换路径模式（相对↔绝对）
// 已经是目标模式的条目直接忽略
export async function convertEntriesPathMode(ids: string[], targetMode: 'relative' | 'absolute'): Promise<{ success: number; failed: number; skipped: number; messages: string[] }> {
  const messages: string[] = []
  let success = 0
  let failed = 0
  let skipped = 0

  for (const id of ids) {
    const idx = state.entries.findIndex(e => e.id === id)
    if (idx < 0) continue
    const entry = state.entries[idx]
    // 网址/系统功能/APPX 不转换路径
    if (entry.type === 'url' || entry.type === 'system' || entry.type === 'appx') continue

    // 已经是目标模式，直接忽略
    if (entry.path_mode === targetMode) {
      skipped++
      continue
    }

    try {
      const result = await api.convert_path_mode(entry, state.currentEnvId, targetMode)
      if (result.success) {
        state.entries[idx].path_mode = result.new_path_mode as 'relative' | 'absolute'
        if (targetMode === 'relative') {
          state.entries[idx].relative_path = result.new_relative_path
          // 转为相对路径后，清除当前环境的绝对路径
          delete state.entries[idx].absolute_paths[state.currentEnvId]
        } else {
          // 转为绝对路径后，设置当前环境的绝对路径
          state.entries[idx].absolute_paths[state.currentEnvId] = result.new_absolute_path
        }
        success++
      } else {
        failed++
        messages.push(`${entry.name}: ${result.message}`)
      }
    } catch (e) {
      failed++
      messages.push(`${entry.name}: ${e}`)
    }
  }

  await api.save_entries(state.entries)
  return { success, failed, skipped, messages }
}

// 批量重新提取图标
// 系统功能为 emoji 图标无需提取，跳过；其余类型调用后端按目标重新提取，
// 成功后更新条目图标（type 统一记为 extracted——favicon 也存于 assets 内的文件资源）。
// url 类型多源 favicon 链全部失败时保持原图标不变（跳过），不计失败。
export async function refreshEntriesIcons(ids: string[]): Promise<{ success: number; failed: number; skipped: number; messages: string[] }> {
  const messages: string[] = []
  let success = 0
  let failed = 0
  let skipped = 0

  for (const id of ids) {
    const idx = state.entries.findIndex(e => e.id === id)
    if (idx < 0) continue
    const entry = state.entries[idx]
    // 系统功能使用 emoji 图标，无需提取
    if (entry.type === 'system') {
      skipped++
      continue
    }

    try {
      const r = await api.refresh_entry_icon(entry, state.currentEnvId)
      if (r.success) {
        state.entries[idx].icon = r.icon_path
          ? { type: 'extracted', source: r.icon_path }
          : { type: 'default', source: '' }
        // bump 图标版本：URL 拼上 ?v= 强制 WebView 重载新图（文件路径不变时会命中缓存）
        bumpIconVersion(r.icon_path)
        success++
      } else if (entry.type === 'url') {
        // url 类型 favicon 链全失败：跳过（保持原图标不变）
        skipped++
      } else {
        failed++
        messages.push(`${entry.name}: ${r.message || '提取失败'}`)
      }
    } catch (e) {
      failed++
      messages.push(`${entry.name}: ${e}`)
    }
  }

  if (success > 0) await api.save_entries(state.entries)
  return { success, failed, skipped, messages }
}

export async function updateLastUsed(id: string) {
  const idx = state.entries.findIndex(e => e.id === id)
  if (idx >= 0) {
    state.entries[idx].last_used = await api.get_now()
    await api.save_entries(state.entries)
  }
}

// 批量启动：逐条调用后端启动，成功后更新最后使用时间
// 路径类条目先 resolve_path 校验，路径失效则计入失败并记录提示；网址/系统/APPX 直接启动
export async function launchEntries(ids: string[]): Promise<{ success: number; failed: number; skipped: number; messages: string[] }> {
  const messages: string[] = []
  let success = 0
  let failed = 0
  let skipped = 0

  for (const id of ids) {
    const entry = state.entries.find(e => e.id === id)
    if (!entry) continue
    try {
      let ok = false
      if (entry.type === 'url' || entry.type === 'system' || entry.type === 'appx') {
        ok = await api.launch_program(entry, state.currentEnvId)
      } else {
        const pathInfo = await api.resolve_path(entry, state.currentEnvId)
        if (!pathInfo.exists) {
          failed++
          messages.push(`${entry.name}: 路径失效`)
          continue
        }
        ok = await api.launch_program(entry, state.currentEnvId)
      }
      if (ok) {
        success++
        state.entries[state.entries.findIndex(e => e.id === id)].last_used = await api.get_now()
      } else {
        failed++
        messages.push(`${entry.name}: 启动失败`)
      }
    } catch (e) {
      failed++
      messages.push(`${entry.name}: ${e}`)
    }
  }

  if (success > 0) await api.save_entries(state.entries)
  return { success, failed, skipped, messages }
}

// ─── Actions: 环境 ───

export async function addEnvironment(env: Omit<Environment, 'id'>): Promise<string> {
  const id = `env_${String(state.environments.length + 1).padStart(3, '0')}_${Date.now()}`
  const newEnv: Environment = { ...env, id }
  state.environments.push(newEnv)
  await api.save_environments(state.environments)
  return id
}

export async function updateEnvironment(id: string, updates: Partial<Environment>) {
  const idx = state.environments.findIndex(e => e.id === id)
  if (idx >= 0) {
    state.environments[idx] = { ...state.environments[idx], ...updates }
    await api.save_environments(state.environments)
  }
}

export async function deleteEnvironment(id: string, deleteEntries: boolean) {
  if (deleteEntries) {
    // 收集被删条目并联动清理资源
    const deletedEntries = state.entries.filter(e => {
      return e.path_mode === 'absolute' && e.absolute_paths && id in e.absolute_paths
    })
    state.entries = state.entries.filter(e => {
      // 删除该环境下的绝对路径条目
      if (e.path_mode === 'absolute' && e.absolute_paths && id in e.absolute_paths) {
        return false
      }
      return true
    })
    await cleanupEntriesAssets(deletedEntries)
  } else {
    // 仅从条目的 absolute_paths 中移除该环境
    state.entries.forEach(e => {
      if (e.absolute_paths && id in e.absolute_paths) {
        delete e.absolute_paths[id]
      }
    })
  }

  state.environments = state.environments.filter(e => e.id !== id)
  await api.save_environments(state.environments)
  await api.save_entries(state.entries)

  // 切换到第一个环境
  if (state.currentEnvId === id && state.environments.length > 0) {
    state.currentEnvId = state.environments[0].id
    state.environments[0].is_current = true
    await api.save_environments(state.environments)
  }
}

export async function switchEnvironment(id: string) {
  state.environments.forEach(e => {
    e.is_current = e.id === id
  })
  state.currentEnvId = id
  await api.save_environments(state.environments)
}

// ─── Actions: 批量操作 ───

export function toggleBatchMode() {
  state.batchMode = !state.batchMode
  if (!state.batchMode) {
    state.selectedIds.clear()
  }
}

export function toggleSelectEntry(id: string) {
  if (state.selectedIds.has(id)) {
    state.selectedIds.delete(id)
  } else {
    state.selectedIds.add(id)
  }
}

export function selectAllCurrent(entries: Entry[]) {
  entries.forEach(e => state.selectedIds.add(e.id))
}

export function invertSelection(entries: Entry[]) {
  entries.forEach(e => {
    if (state.selectedIds.has(e.id)) {
      state.selectedIds.delete(e.id)
    } else {
      state.selectedIds.add(e.id)
    }
  })
}

export function clearSelection() {
  state.selectedIds.clear()
}

// ─── Actions: 设置 ───

/** 保存设置（合并到 state.config 并持久化） */
export async function saveSettings(updates: Partial<AppConfig>) {
  state.config = { ...state.config, ...updates }
  await api.save_config(state.config)
}

// ─── 主题（明/暗/随系统） ───

/** 系统暗色偏好媒体查询（惰性创建，模块级单例） */
let systemDarkQuery: MediaQueryList | null = null

/** 当前系统是否暗色 */
function systemPrefersDark(): boolean {
  if (!systemDarkQuery) {
    systemDarkQuery = window.matchMedia('(prefers-color-scheme: dark)')
  }
  return systemDarkQuery.matches
}

/** 按主题设置给 html 根元素挂/摘 dark 类 */
function applyThemeClass(theme: 'light' | 'dark' | 'system') {
  const dark = theme === 'dark' || (theme === 'system' && systemPrefersDark())
  document.documentElement.classList.toggle('dark', dark)
}

/**
 * 切换主题并持久化；system 模式注册系统偏好变化监听（变化时即时切换）。
 * 应用启动时也应调用一次以应用已保存的主题。
 */
export async function setTheme(theme: 'light' | 'dark' | 'system') {
  state.config.theme = theme
  applyThemeClass(theme)
  await api.save_config(state.config)
}

/** 应用启动时初始化主题（应用配置并监听系统明暗变化，仅 system 模式生效） */
export function initTheme() {
  applyThemeClass(state.config.theme)
  if (!systemDarkQuery) {
    systemDarkQuery = window.matchMedia('(prefers-color-scheme: dark)')
    systemDarkQuery.addEventListener('change', () => {
      // 仅随系统模式下响应（用户手动指定明/暗时不跟随）
      if (state.config.theme === 'system') {
        applyThemeClass('system')
      }
    })
  }
}

// ─── 资源路径解析 ───

/** 全局单调递增的图标版本计数器（iconVersions 各项共用） */
let iconVersionCounter = 0

/** 标记某图标文件已更新：bump 版本号，resolveAssetUrl 拼上 ?v= 强制 WebView 重载 */
export function bumpIconVersion(path: string) {
  if (!path) return
  iconVersionCounter++
  state.iconVersions.set(path, iconVersionCounter)
}

/**
 * 将后端返回的相对路径（如 "assets/icons/extracted/foo.png"）转为 WebView 可访问的 URL。
 * - 如果传入的是 http(s) URL，直接返回。
 * - 如果 exeDir 未就绪，返回原始路径（降级处理）。
 * - 若该路径被 bump 过版本（重置图标后内容更新），拼上 ?v= 查询参数绕过 WebView 缓存
 *   （Tauri asset 协议只取 URL 的 path 部分读文件，查询参数不影响解析）。
 */
export function resolveAssetUrl(path: string): string {
  if (!path) return ''
  // 已经是 URL 的直接返回
  if (/^https?:\/\//i.test(path)) return path
  // 已经是 convertFileSrc 格式
  if (path.startsWith('asset://') || path.startsWith('tauri://')) return path
  // 版本参数（asset 协议忽略查询串，仅用于绕过 WebView 图片缓存）
  const version = state.iconVersions.get(path)
  const versionQuery = version ? `?v=${version}` : ''
  // 绝对路径直接转换
  if (/^[A-Za-z]:[\\/]/.test(path)) {
    return convertFileSrc(path) + versionQuery
  }
  // 相对路径：拼接 exeDir 后转换
  if (state.exeDir) {
    const sep = state.exeDir.endsWith('\\') || state.exeDir.endsWith('/') ? '' : '/'
    const abs = state.exeDir + sep + path.replace(/^\.\//, '')
    return convertFileSrc(abs) + versionQuery
  }
  return path + versionQuery
}

// ─── 工具函数 ───

/** 获取分类的顶级祖先 ID（向上遍历 parent_id 链） */
export function getTopAncestor(id: string): string | null {
  let cat = state.categories.find(c => c.id === id)
  if (!cat) return null
  while (cat && cat.parent_id) {
    const parent = state.categories.find(c => c.id === cat!.parent_id)
    if (!parent) break
    cat = parent
  }
  return cat?.id ?? null
}

/** 获取所有分类的扁平列表（可排除指定 ID 及其子孙） */
export function getFlatCategories(excludeId?: string): Category[] {
  if (!excludeId) return [...state.categories]
  // 收集排除项的所有子孙
  const excluded = new Set<string>([excludeId])
  const collectChildren = (parentId: string) => {
    getSubCategories(parentId).forEach(c => {
      excluded.add(c.id)
      collectChildren(c.id)
    })
  }
  collectChildren(excludeId)
  return state.categories.filter(c => !excluded.has(c.id))
}

/** 条目工厂：创建一个带有默认值的新条目草稿（不含 id/add_time/last_used） */
export function createEntryDraft(categoryId: string, type: EntryType): Omit<Entry, 'id' | 'add_time' | 'last_used'> {
  return {
    name: '',
    type,
    category_id: categoryId,
    relative_path: '',
    absolute_paths: {},
    path_mode: 'relative',
    launch_args: '',
    working_directory: '',
    window_style: 'normal',
    run_as_admin: false,
    url: '',
    icon: { type: 'default', source: '' },
    cover: { enabled: false, source: '' },
    tags: [],
    notes: '',
  }
}

// ─── 导出 store ───

export { state }
