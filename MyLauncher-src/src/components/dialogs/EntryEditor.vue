<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { state, addEntry, updateEntry, resolveAssetUrl, bumpIconVersion, createEntryDraft } from '@/store'
import * as api from '@/api'
import type { Entry, EntryType } from '@/types'
import { pickAndCopyImage } from '@/composables/useAssetImagePicker'
import PathModeFields from '@/components/PathModeFields.vue'
import SystemAppPicker, { type SystemAppSelection } from '@/components/dialogs/SystemAppPicker.vue'

const props = defineProps<{
  entry: Entry | null
  /** 新建时的初始类型（编辑时忽略，以条目自身类型为准） */
  entryType: EntryType
  currentCategoryId: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'saved'): void
}>()

/** 七种类型的展示信息 */
const TYPE_OPTIONS: { value: EntryType; label: string; icon: string }[] = [
  { value: 'program', label: '程序', icon: '📦' },
  { value: 'url', label: '网址', icon: '🌐' },
  { value: 'folder', label: '文件夹', icon: '📁' },
  { value: 'file', label: '文件', icon: '📄' },
  { value: 'system', label: '系统功能', icon: '🛠️' },
  { value: 'appx', label: 'APPX 应用', icon: '🏪' },
  { value: 'steam', label: 'Steam 游戏', icon: '🎮' },
]

function typeLabel(type: EntryType): string {
  return TYPE_OPTIONS.find(o => o.value === type)?.label || '项目'
}

// 表单数据 — 用 createEntryDraft 初始化默认值
const form = ref(createEntryDraft(props.currentCategoryId || 'cat_001', props.entryType))
form.value.type = props.entryType

const tagInput = ref('')
const saving = ref(false)
const error = ref('')
const loadingInfo = ref(false)
const refreshingIcon = ref(false)
const showSystemPicker = ref(false)

/** 系统功能/APPX：目标存于 relative_path，无路径区/启动设置/自定义图标/封面 */
const isUriType = computed(() => form.value.type === 'system' || form.value.type === 'appx')

/** 打开系统功能/APPX 选择器 */
function openSystemPicker() {
  showSystemPicker.value = true
}

// ─── 旧数据类型自动纠正 ───
// 早期版本把系统功能/APPX 存为 program 类型（目标存 relative_path），
// 编辑时按目标前缀识别真实类型并纠正，保存后即持久化为正确类型
// 旧版 Steam 扫描导入的条目为 url 类型（url 以 steam://rungameid/ 开头），同样纠正为 steam
function correctLegacyType(e: Entry): EntryType {
  if (e.type === 'url' && e.url.toLowerCase().startsWith('steam://rungameid/')) return 'steam'
  if (e.type !== 'program') return e.type
  const t = e.relative_path?.trim() || ''
  if (t.toLowerCase().startsWith('shell:appsfolder')) return 'appx'
  if (t.toLowerCase().startsWith('shell:')
    || t.toLowerCase().startsWith('ms-settings:')
    || t.toLowerCase().startsWith('ms-get-started:')
    || t.toLowerCase().startsWith('ms-contact-support:')) return 'system'
  // 裸命令名（如 taskmgr.exe / devmgmt.msc，由系统按 PATH 搜索）
  if (t && !t.includes('\\') && !t.includes('/') && !t.includes(':')) return 'system'
  return 'program'
}

// 选择器回填：系统功能（emoji 图标）/ APPX 应用（真实图标）
function applySystemSelection(sel: SystemAppSelection) {
  form.value.name = sel.name
  form.value.relative_path = sel.target
  form.value.type = sel.iconEmoji ? 'system' : 'appx'
  form.value.absolute_paths = {}
  form.value.path_mode = 'relative'
  form.value.launch_args = ''
  form.value.working_directory = ''
  form.value.run_as_admin = false
  if (sel.iconEmoji) {
    form.value.icon = { type: 'emoji', source: sel.iconEmoji }
  } else if (sel.iconPath) {
    form.value.icon = { type: 'extracted', source: sel.iconPath }
  }
}

const isEdit = computed(() => !!props.entry)
const titleText = computed(() => {
  const label = typeLabel(form.value.type)
  return isEdit.value ? `编辑${label}` : `添加${label}`
})

// 加载已有数据
function loadEntry() {
  if (props.entry) {
    // 旧数据类型纠正（program → system/appx）
    const e: Entry = { ...props.entry, type: correctLegacyType(props.entry) }
    form.value = {
      name: e.name,
      type: e.type,
      category_id: e.category_id,
      relative_path: e.relative_path,
      absolute_paths: { ...e.absolute_paths },
      path_mode: e.path_mode,
      launch_args: e.launch_args,
      working_directory: e.working_directory,
      window_style: e.window_style as any,
      run_as_admin: e.run_as_admin,
      url: e.url,
      icon: { ...e.icon },
      cover: { ...e.cover },
      tags: [...e.tags],
      notes: e.notes,
    }
  }
}

onMounted(loadEntry)

// 选择程序文件（支持 exe/bat/cmd 和 .lnk 快捷方式）
async function pickExe() {
  const path = await api.pick_file([{ name: '程序/快捷方式', extensions: ['exe', 'bat', 'cmd', 'lnk'] }])
  if (!path) return

  loadingInfo.value = true
  try {
    if (/\.lnk$/i.test(path)) {
      const lnk = await api.resolve_lnk(path)
      form.value.name = lnk.name
      form.value.relative_path = lnk.relative_path
      form.value.absolute_paths = lnk.target_path ? { [state.currentEnvId]: lnk.target_path } : {}
      form.value.path_mode = lnk.suggested_path_mode as 'relative' | 'absolute'
      form.value.launch_args = lnk.arguments || ''
      form.value.working_directory = lnk.working_directory || ''
      form.value.window_style = (lnk.window_style as 'normal' | 'maximized' | 'minimized') || 'normal'
      form.value.notes = lnk.description || ''
      if (lnk.icon_path) {
        form.value.icon = { type: 'extracted', source: lnk.icon_path }
      }
    } else {
      const info = await api.get_exe_info(path)
      form.value.name = info.name
      form.value.relative_path = info.relative_path
      form.value.absolute_paths = { [state.currentEnvId]: info.absolute_path }
      form.value.path_mode = info.suggested_path_mode as 'relative' | 'absolute'
      if (info.icon_path) {
        form.value.icon = { type: 'extracted', source: info.icon_path }
      }
    }
  } catch (e) {
    error.value = `读取程序信息失败: ${e}`
  } finally {
    loadingInfo.value = false
  }
}

// 选择文件夹/文件（合并 pickFolder 和 pickFile）
async function pickPath() {
  const isFolder = form.value.type === 'folder'
  const path = isFolder
    ? await api.pick_directory()
    : await api.pick_file()
  if (!path) return

  loadingInfo.value = true
  try {
    const info = await api.get_file_info(path)
    if (!info.file_exists) {
      error.value = '文件不存在'
      return
    }
    form.value.name = info.name
    form.value.relative_path = info.relative_path
    form.value.absolute_paths = info.absolute_path ? { [state.currentEnvId]: info.absolute_path } : {}
    form.value.path_mode = info.suggested_path_mode as 'relative' | 'absolute'
    if (info.icon_path) {
      form.value.icon = { type: 'extracted', source: info.icon_path }
    }
  } catch (e) {
    error.value = `读取${isFolder ? '文件夹' : '文件'}信息失败: ${e}`
  } finally {
    loadingInfo.value = false
  }
}

// 自定义图标（可选图片或 exe/快捷方式，exe 提取其图标；初始目录定位到条目程序所在目录）
async function pickIcon() {
  const defaultDir = iconPickerDir.value
  const path = await api.pick_file(
    [{ name: '图片或程序', extensions: ['png', 'jpg', 'jpeg', 'ico', 'bmp', 'webp', 'exe', 'lnk'] }],
    defaultDir
  )
  if (!path) return
  if (/\.(exe|lnk)$/i.test(path)) {
    // exe：提取图标；lnk：解析目标后提取
    try {
      const target = /\.lnk$/i.test(path) ? (await api.resolve_lnk(path)).target_path : path
      if (!target) {
        error.value = '快捷方式未指向有效程序，无法提取图标'
        return
      }
      const info = await api.get_exe_info(target)
      if (info.icon_path) {
        form.value.icon = { type: 'extracted', source: info.icon_path }
      } else {
        error.value = '该程序未提取到图标'
      }
    } catch (e) {
      error.value = `提取程序图标失败: ${e}`
    }
  } else {
    const relPath = await api.copy_file_to_assets(path, 'icons/custom')
    if (relPath) {
      form.value.icon = { type: 'custom', source: relPath }
    }
  }
}

/** 图标浏览初始目录：条目当前解析路径的父目录（程序/文件夹/文件/Steam 游戏 exe） */
const iconPickerDir = computed(() => {
  const abs = form.value.absolute_paths[state.currentEnvId] || ''
  if (abs && /^[A-Za-z]:[\\/]/.test(abs)) {
    const idx = Math.max(abs.lastIndexOf('\\'), abs.lastIndexOf('/'))
    return idx > 2 ? abs.slice(0, idx) : ''
  }
  if (form.value.relative_path) {
    // 相对路径 ../ 前缀基于 exe 目录解析
    const rp = form.value.relative_path.replace(/^([\\/])+/, '')
    if (rp.startsWith('..')) {
      const joined = state.exeDir + '\\' + rp
      const idx = Math.max(joined.lastIndexOf('\\'), joined.lastIndexOf('/'))
      return idx > 2 ? joined.slice(0, idx) : ''
    }
  }
  return ''
})

// 封面图
async function pickCover() {
  const relPath = await pickAndCopyImage('covers/custom', ['png', 'jpg', 'jpeg', 'bmp', 'webp'])
  if (relPath) {
    form.value.cover = { enabled: true, source: relPath }
  }
}

function clearCover() {
  form.value.cover = { enabled: false, source: '' }
}

// ─── Steam 游戏：按 ID 扫描 ───

const steamAppId = ref('')
const scanningSteam = ref(false)
const loadingSteamInfo = ref(false)
/** 扫描成功后回显的游戏 exe 路径（来自 absolute_paths 当前环境值） */
const steamExePath = computed(() => form.value.absolute_paths[state.currentEnvId] || '')

// 编辑已有条目时，从启动链接 steam://rungameid/{appid} 提取 ID 回显
watch(() => form.value.url, (url) => {
  const m = url?.match(/^steam:\/\/rungameid\/(\d+)/i)
  if (m) steamAppId.value = m[1]
}, { immediate: true })

/** 按 ID 扫描 Steam 游戏：回填名称/启动链接/游戏 exe 路径/封面/图标 */
async function scanSteamApp() {
  const id = steamAppId.value.trim()
  if (!id) {
    error.value = '请输入 Steam 游戏 ID'
    return
  }
  if (!/^\d+$/.test(id)) {
    error.value = '游戏 ID 应为纯数字（Steam 商店页 URL 中的 appid）'
    return
  }
  error.value = ''
  scanningSteam.value = true
  try {
    const r = await api.scan_steam_app(id)
    form.value.name = r.name || form.value.name
    form.value.url = `steam://rungameid/${id}`
    form.value.absolute_paths = r.exe_path
      ? { [state.currentEnvId]: r.exe_path }
      : {}
    form.value.path_mode = 'absolute'
    if (r.cover_path) {
      form.value.cover = { enabled: true, source: r.cover_path }
    }
    if (r.icon_path) {
      form.value.icon = { type: 'extracted', source: r.icon_path }
    }
  } catch (e) {
    error.value = `扫描失败: ${e}`
  } finally {
    scanningSteam.value = false
  }
}

/** 手动修改启动链接时，同步更新 ID 输入框（保持两者一致） */
function onSteamUrlInput() {
  const m = form.value.url.match(/^steam:\/\/rungameid\/(\d+)/i)
  if (m) steamAppId.value = m[1]
}

/** 浏览选择 Steam 游戏主程序（仅设置 exe 路径与提取图标，不覆盖名称/启动链接） */
async function pickSteamExe() {
  const path = await api.pick_file(
    [{ name: '程序文件', extensions: ['exe'] }],
    steamExeDir.value || undefined
  )
  if (!path) return
  loadingSteamInfo.value = true
  try {
    const info = await api.get_exe_info(path)
    form.value.absolute_paths = { [state.currentEnvId]: info.absolute_path }
    form.value.path_mode = 'absolute'
    if (info.icon_path) {
      form.value.icon = { type: 'extracted', source: info.icon_path }
    }
  } catch (e) {
    error.value = `读取游戏程序信息失败: ${e}`
  } finally {
    loadingSteamInfo.value = false
  }
}

/** 游戏程序浏览初始目录：当前 exe 路径的父目录 */
const steamExeDir = computed(() => {
  const abs = steamExePath.value
  if (!abs || !/^[A-Za-z]:[\\/]/.test(abs)) return ''
  const idx = Math.max(abs.lastIndexOf('\\'), abs.lastIndexOf('/'))
  return idx > 2 ? abs.slice(0, idx) : ''
})

// Favicon 获取
async function fetchFavicon() {
  if (!form.value.url) {
    error.value = '请先输入网址'
    return
  }
  try {
    const iconPath = await api.fetch_favicon(form.value.url)
    if (iconPath) {
      form.value.icon = { type: 'custom', source: iconPath }
    }
  } catch (e) {
    error.value = `获取 favicon 失败: ${e}`
  }
}

// 重新提取图标（按条目类型分派：程序/文件夹/文件提取真实图标，网址多源链获取 favicon）
async function refreshIcon() {
  error.value = ''
  refreshingIcon.value = true
  try {
    // 以当前表单内容构造临时条目传给后端（未保存的新条目也能提取）
    // Rust 端 Entry 严格反序列化，需补齐 id/add_time/last_used 三个必填字段
    const draft = {
      ...props.entry,
      ...form.value,
      id: props.entry?.id || 'draft',
      add_time: props.entry?.add_time || '',
      last_used: props.entry?.last_used || '',
    } as Entry
    const r = await api.refresh_entry_icon(draft, state.currentEnvId)
    if (r.success && r.icon_path) {
      form.value.icon = { type: 'extracted', source: r.icon_path }
      // bump 版本：URL 拼上 ?v= 绕过 WebView 缓存，预览图立即更新
      bumpIconVersion(r.icon_path)
    } else if (r.success) {
      form.value.icon = { type: 'default', source: '' }
    } else if (form.value.type === 'url') {
      // url 类型 favicon 链全失败：保持原图标不变，提示跳过
      error.value = `${r.message || '无法获取 favicon'}，已跳过（保持原图标）`
    } else {
      error.value = r.message || '重新提取图标失败'
    }
  } catch (e) {
    error.value = `重新提取图标失败: ${e}`
  } finally {
    refreshingIcon.value = false
  }
}

// 起始位置浏览
async function pickWorkingDir() {
  const path = await api.pick_directory()
  if (path) {
    form.value.working_directory = path
  }
}

// 标签管理
function addTag() {
  const tag = tagInput.value.trim()
  if (tag && !form.value.tags.includes(tag)) {
    form.value.tags.push(tag)
    tagInput.value = ''
  }
}

function removeTag(index: number) {
  form.value.tags.splice(index, 1)
}

// 保存
async function save() {
  if (!form.value.name.trim()) {
    error.value = '请输入名称'
    return
  }
  if (form.value.type === 'program' && !form.value.relative_path && Object.keys(form.value.absolute_paths).length === 0) {
    error.value = '请选择程序文件'
    return
  }
  if (form.value.type === 'url' && !form.value.url.trim()) {
    error.value = '请输入网址'
    return
  }
  if (form.value.type === 'steam' && !form.value.url.trim()) {
    error.value = '请先输入游戏 ID 并扫描，或手动填写 Steam 启动链接'
    return
  }
  if ((form.value.type === 'folder' || form.value.type === 'file') && !form.value.relative_path && Object.keys(form.value.absolute_paths).length === 0) {
    error.value = `请选择${form.value.type === 'folder' ? '文件夹' : '文件'}`
    return
  }
  if (isUriType.value && !form.value.relative_path.trim()) {
    error.value = typeLabel(form.value.type) + '需要先选择目标'
    return
  }

  saving.value = true
  error.value = ''

  try {
    if (isEdit.value && props.entry) {
      await updateEntry(props.entry.id, { ...form.value })
    } else {
      await addEntry({ ...form.value } as Omit<Entry, 'id' | 'add_time' | 'last_used'>)
    }
    emit('saved')
    emit('close')
  } catch (e) {
    error.value = `保存失败: ${e}`
  } finally {
    saving.value = false
  }
}

// 分类选项：按层级树显示
const categoryOptions = computed(() => {
  const result: { id: string; name: string; depth: number }[] = []
  const visit = (parentId: string | null, depth: number) => {
    state.categories
      .filter(c => c.parent_id === parentId)
      .sort((a, b) => a.sort_order - b.sort_order)
      .forEach(c => {
        result.push({ id: c.id, name: c.name, depth })
        visit(c.id, depth + 1)
      })
  }
  visit(null, 0)
  return result
})

// ─── 当前图标/封面存储路径（文件型资源显示程序目录内相对路径） ───

const iconPathText = computed(() => {
  const icon = form.value.icon
  if (icon.type === 'emoji' && icon.source) return `Emoji 图标：${icon.source}`
  if (icon.source) return icon.source
  return ''
})

/** 重新提取按钮的悬浮提示（按类型说明提取来源） */
const refreshIconTitle = computed(() => {
  switch (form.value.type) {
    case 'program': return '从程序文件重新提取图标'
    case 'folder': return '从文件夹重新提取图标'
    case 'file': return '从文件重新提取图标'
    case 'url': return '多源获取网址图标（获取失败时保持原图标）'
    case 'steam': return '从游戏主程序重新提取图标'
    default: return '重新提取图标'
  }
})

const coverPathText = computed(() => {
  const cover = form.value.cover
  return cover.enabled && cover.source ? cover.source : ''
})

// ─── 盘符判断：不同分区时只能使用绝对路径 ───

const exeDrive = computed(() => {
  const m = state.exeDir.match(/^([A-Za-z]):/)
  return m ? m[1].toUpperCase() : ''
})

const currentAbsPath = computed(() => form.value.absolute_paths[state.currentEnvId] || '')

const absPathDrive = computed(() => {
  const m = currentAbsPath.value.match(/^([A-Za-z]):/)
  return m ? m[1].toUpperCase() : ''
})

const isDifferentDrive = computed(() => {
  if (!exeDrive.value) return false
  if (!absPathDrive.value) return false
  return absPathDrive.value !== exeDrive.value
})

watch(isDifferentDrive, val => {
  if (val) form.value.path_mode = 'absolute'
})
watch(currentAbsPath, () => {
  if (isDifferentDrive.value) form.value.path_mode = 'absolute'
})

// ─── 类型切换：清理不适用于新类型的字段 ───

watch(() => form.value.type, (newType, oldType) => {
  if (newType === oldType) return

  // 切到系统功能/APPX：清空路径类字段（目标走选择器回填）
  if (newType === 'system' || newType === 'appx') {
    form.value.launch_args = ''
    form.value.working_directory = ''
    form.value.run_as_admin = false
    if (oldType !== 'system' && oldType !== 'appx' && oldType !== 'url') {
      form.value.relative_path = ''
      form.value.absolute_paths = {}
      form.value.path_mode = 'relative'
    }
  }

  // 切到网址：清空路径类字段
  if (newType === 'url') {
    form.value.relative_path = ''
    form.value.absolute_paths = {}
    form.value.path_mode = 'relative'
    form.value.launch_args = ''
    form.value.working_directory = ''
    form.value.run_as_admin = false
  }

  // 切到 Steam 游戏：仅清相对路径类字段（启动链接/exe 路径由 ID 扫描或手动填写回填；启动设置保留，便于从程序类切换时继承）
  if (newType === 'steam') {
    form.value.relative_path = ''
    form.value.path_mode = 'absolute'
    if (oldType !== 'url' || !form.value.url.toLowerCase().startsWith('steam://rungameid/')) {
      form.value.url = ''
    }
  }

  // 从 Steam 切走：清启动链接与 ID（避免残留 steam:// 到其他类型）
  if (oldType === 'steam' && newType !== 'steam') {
    if (form.value.url.toLowerCase().startsWith('steam://rungameid/')) {
      form.value.url = ''
    }
  }
})
</script>

<template>
  <div class="dialog-overlay" @click.self="$emit('close')">
    <div class="dialog entry-editor-dialog">
      <div class="dialog-header">
        <span>{{ titleText }}</span>
        <button class="btn btn-icon" @click="$emit('close')">✕</button>
      </div>

      <div class="dialog-body">
        <div v-if="error" class="error-msg">{{ error }}</div>

        <!-- 名称 -->
        <div class="form-row">
          <label class="form-label">名称</label>
          <input v-model="form.name" class="input" placeholder="输入名称" />
        </div>

        <!-- 类型（必选） -->
        <div class="form-row">
          <label class="form-label">类型</label>
          <select v-model="form.type" class="select">
            <option v-for="opt in TYPE_OPTIONS" :key="opt.value" :value="opt.value">{{ opt.icon }} {{ opt.label }}</option>
          </select>
        </div>

        <!-- 分类 -->
        <div class="form-row">
          <label class="form-label">分类</label>
          <select v-model="form.category_id" class="select">
            <option v-for="cat in categoryOptions" :key="cat.id" :value="cat.id">
              {{ '　'.repeat(cat.depth) }}{{ cat.depth > 0 ? '└ ' : '' }}{{ cat.name }}
            </option>
          </select>
        </div>

        <!-- 程序路径 -->
        <template v-if="form.type === 'program'">
          <div class="form-section">路径</div>

          <div class="form-row">
            <label class="form-label">程序文件</label>
            <div style="display: flex; gap: 4px;">
              <input v-model="form.relative_path" class="input" placeholder="选择 exe 文件" style="flex: 1;" />
              <button class="btn" @click="pickExe" :disabled="loadingInfo">{{ loadingInfo ? '...' : '浏览...' }}</button>
            </div>
          </div>

          <PathModeFields
            v-model:path-mode="form.path_mode"
            v-model:relative-path="form.relative_path"
            v-model:absolute-path="form.absolute_paths[state.currentEnvId]"
            :is-different-drive="isDifferentDrive"
          />

          <div class="form-section">启动设置</div>

          <div class="form-row">
            <label class="form-label">启动参数</label>
            <input v-model="form.launch_args" class="input" placeholder="如: -multiInst" />
          </div>

          <div class="form-row">
            <label class="form-label">起始位置</label>
            <div style="display: flex; gap: 4px;">
              <input v-model="form.working_directory" class="input" placeholder="留空默认 exe 所在目录" style="flex: 1;" />
              <button class="btn" @click="pickWorkingDir">浏览...</button>
            </div>
          </div>

          <div class="form-row">
            <label class="form-label">运行方式</label>
            <select v-model="form.window_style" class="select">
              <option value="normal">常规窗口</option>
              <option value="maximized">最大化</option>
              <option value="minimized">最小化</option>
            </select>
          </div>

          <div class="form-row">
            <label class="form-label">权限</label>
            <label class="checkbox-label">
              <input type="checkbox" v-model="form.run_as_admin" /> 以管理员身份运行
            </label>
          </div>
        </template>

        <!-- 网址 -->
        <template v-if="form.type === 'url'">
          <div class="form-section">网址</div>
          <div class="form-row">
            <label class="form-label">网址</label>
            <input v-model="form.url" class="input" placeholder="https://..." />
          </div>
        </template>

        <!-- Steam 游戏 -->
        <template v-if="form.type === 'steam'">
          <div class="form-section">Steam 游戏</div>

          <div class="form-row">
            <label class="form-label">游戏 ID</label>
            <div style="display: flex; gap: 4px;">
              <input v-model="steamAppId" class="input" placeholder="Steam 商店页 URL 中的数字 ID" style="flex: 1;" @keyup.enter="scanSteamApp" />
              <button class="btn" @click="scanSteamApp" :disabled="scanningSteam">
                {{ scanningSteam ? '扫描中...' : '扫描' }}
              </button>
            </div>
          </div>

          <div class="form-row">
            <label class="form-label">启动链接</label>
            <input v-model="form.url" class="input" placeholder="steam://rungameid/..." @input="onSteamUrlInput" />
          </div>

          <div class="form-row">
            <label class="form-label">游戏程序</label>
            <div style="display: flex; gap: 4px;">
              <input :value="steamExePath" class="input" placeholder="输入 ID 扫描后自动回填，或浏览选择游戏主程序" style="flex: 1;" readonly />
              <button class="btn" @click="pickSteamExe" :disabled="loadingSteamInfo">{{ loadingSteamInfo ? '...' : '浏览...' }}</button>
            </div>
          </div>

          <div class="form-section">启动设置</div>

          <div class="form-row">
            <label class="form-label">启动参数</label>
            <input v-model="form.launch_args" class="input" placeholder="如: -nolauncher（直接启动 exe 方式时生效）" />
          </div>

          <div class="form-row">
            <label class="form-label">起始位置</label>
            <div style="display: flex; gap: 4px;">
              <input v-model="form.working_directory" class="input" placeholder="留空默认游戏 exe 所在目录" style="flex: 1;" />
              <button class="btn" @click="pickWorkingDir">浏览...</button>
            </div>
          </div>

          <div class="form-row">
            <label class="form-label">运行方式</label>
            <select v-model="form.window_style" class="select">
              <option value="normal">常规窗口</option>
              <option value="maximized">最大化</option>
              <option value="minimized">最小化</option>
            </select>
          </div>

          <div class="form-row">
            <label class="form-label">权限</label>
            <label class="checkbox-label">
              <input type="checkbox" v-model="form.run_as_admin" /> 以管理员身份运行
            </label>
          </div>
        </template>

        <!-- 文件夹/文件路径 -->
        <template v-if="form.type === 'folder' || form.type === 'file'">
          <div class="form-section">路径</div>

          <div class="form-row">
            <label class="form-label">{{ form.type === 'folder' ? '文件夹' : '文件' }}</label>
            <div style="display: flex; gap: 4px;">
              <input v-model="form.relative_path" class="input" :placeholder="form.type === 'folder' ? '选择文件夹' : '选择文件'" style="flex: 1;" />
              <button class="btn" @click="pickPath" :disabled="loadingInfo">{{ loadingInfo ? '...' : '浏览...' }}</button>
            </div>
          </div>

          <PathModeFields
            v-model:path-mode="form.path_mode"
            v-model:relative-path="form.relative_path"
            v-model:absolute-path="form.absolute_paths[state.currentEnvId]"
            :is-different-drive="isDifferentDrive"
          />
        </template>

        <!-- 系统功能/APPX：选择目标 -->
        <template v-if="isUriType">
          <div class="form-section">目标</div>

          <div class="form-row">
            <label class="form-label">{{ typeLabel(form.type) }}</label>
            <div style="display: flex; gap: 4px; align-items: center;">
              <input v-if="form.relative_path" class="input" :value="form.name ? `${form.name}` : form.relative_path" disabled style="flex: 1;" />
              <span v-else class="target-empty-hint">尚未选择</span>
              <button class="btn" @click="openSystemPicker">{{ form.relative_path ? '重新选择...' : '选择' + typeLabel(form.type) + '...' }}</button>
            </div>
          </div>

          <div class="form-row" v-if="form.relative_path">
            <label class="form-label">启动目标</label>
            <span class="target-uri-text">{{ form.relative_path }}</span>
          </div>
        </template>

        <!-- 图标（系统功能/APPX 不显示，图标随所选项目） -->
        <template v-if="!isUriType">
          <div class="form-section">图标</div>
          <div class="form-row">
            <label class="form-label">当前图标</label>
            <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
              <div class="icon-preview">
                <img v-if="form.icon.source && form.icon.type !== 'emoji'" :src="resolveAssetUrl(form.icon.source)" alt="icon" @error="($event.target as HTMLImageElement).style.display='none'" @load="($event.target as HTMLImageElement).style.display=''" />
                <span v-else-if="form.icon.type === 'emoji' && form.icon.source" class="icon-emoji-preview">{{ form.icon.source }}</span>
                <span v-else class="icon-placeholder-text">{{ form.name.charAt(0) || '?' }}</span>
              </div>
              <button class="btn" @click="pickIcon">自定义图标...</button>
              <button v-if="form.type === 'url'" class="btn" @click="fetchFavicon">获取 Favicon</button>
              <button class="btn" @click="refreshIcon" :disabled="refreshingIcon" :title="refreshIconTitle">
                {{ refreshingIcon ? '提取中...' : '重新提取图标' }}
              </button>
            </div>
          </div>
          <div class="form-row" v-if="iconPathText">
            <label class="form-label">存储路径</label>
            <span class="resource-path-text">{{ iconPathText }}</span>
          </div>
        </template>

        <!-- 封面图（系统功能/APPX 不显示） -->
        <template v-if="!isUriType">
          <div class="form-section">封面图</div>
          <div class="form-row">
            <label class="form-label">封面</label>
            <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
              <div class="cover-preview">
                <img v-if="form.cover.enabled && form.cover.source" :src="resolveAssetUrl(form.cover.source)" alt="cover" @error="($event.target as HTMLImageElement).style.display='none'" @load="($event.target as HTMLImageElement).style.display=''" />
                <span v-else class="cover-placeholder-text">无封面</span>
              </div>
              <button class="btn" @click="pickCover">选择图片...</button>
              <button v-if="form.cover.enabled" class="btn btn-danger" @click="clearCover">清除</button>
            </div>
          </div>
          <div class="form-row" v-if="coverPathText">
            <label class="form-label">存储路径</label>
            <span class="resource-path-text">{{ coverPathText }}</span>
          </div>
        </template>

        <!-- 其他 -->
        <div class="form-section">其他</div>
        <div class="form-row">
          <label class="form-label">标签</label>
          <div style="display: flex; gap: 4px; flex-wrap: wrap; align-items: center;">
            <span v-for="(tag, i) in form.tags" :key="i" class="tag tag-removable" @click="removeTag(i)">{{ tag }} ✕</span>
            <input v-model="tagInput" class="input tag-input" placeholder="输入标签后回车" @keyup.enter="addTag" />
          </div>
        </div>
        <div class="form-row">
          <label class="form-label">备注</label>
          <textarea v-model="form.notes" class="textarea" placeholder="备注信息"></textarea>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn" @click="$emit('close')">取消</button>
        <button class="btn btn-primary" @click="save" :disabled="saving">{{ saving ? '保存中...' : '确定' }}</button>
      </div>

      <!-- 系统功能 / APPX 应用选择器（按当前类型锁定标签页） -->
      <SystemAppPicker
        v-if="showSystemPicker"
        :initial-tab="form.type === 'appx' ? 'appx' : 'system'"
        @close="showSystemPicker = false"
        @select="applySystemSelection"
      />
    </div>
  </div>
</template>

<style scoped>
.entry-editor-dialog {
  width: 560px;
  max-height: 80vh;
}

/* EntryEditor 的 form-label 比全局宽 10px */
.form-label {
  width: 80px;
}

.checkbox-label {
  font-size: 13px;
  cursor: pointer;
  padding-top: 7px;
}

.error-msg {
  background: var(--color-danger-bg);
  color: var(--color-danger);
  padding: 8px 12px;
  border-radius: var(--radius-sm);
  font-size: 12px;
  margin-bottom: 12px;
}

.icon-preview {
  width: 40px;
  height: 40px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  flex-shrink: 0;
}

.icon-preview img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.icon-emoji-preview {
  font-size: 26px;
  line-height: 1;
}

.icon-placeholder-text {
  font-size: 16px;
  color: var(--color-text-tertiary);
}

.cover-preview {
  width: 64px;
  height: 48px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  flex-shrink: 0;
  background: var(--color-bg-sidebar);
}

.cover-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.cover-placeholder-text {
  font-size: 11px;
  color: var(--color-text-tertiary);
}

.target-empty-hint {
  font-size: 13px;
  color: var(--color-text-tertiary);
  padding-top: 7px;
}

.target-uri-text {
  font-size: 12px;
  color: var(--color-text-secondary);
  padding-top: 8px;
  word-break: break-all;
}

.resource-path-text {
  font-size: 12px;
  color: var(--color-text-tertiary);
  padding-top: 8px;
  word-break: break-all;
  user-select: text;
}

.tag-removable {
  cursor: pointer;
  transition: background var(--transition);
}

.tag-removable:hover {
  background: var(--color-danger-bg);
  color: var(--color-danger);
}

.tag-input {
  width: 120px;
  flex: 0 0 120px;
}
</style>
