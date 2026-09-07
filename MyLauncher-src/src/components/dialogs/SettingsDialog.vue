<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { state, saveSettings, resolveAssetUrl } from '@/store'
import { scan_orphan_assets, clean_orphan_assets, set_autorun, get_autorun } from '@/api'
import { useToast } from '@/composables/useToast'
import IconPicker from '@/components/IconPicker.vue'

const emit = defineEmits<{
  (e: 'close'): void
}>()


// 表单（从当前配置初始化）
const form = ref({
  logo_icon: state.config.logo_icon || '🪷',
  logo_icon_type: (state.config.logo_icon_type === 'custom' ? 'custom' : 'emoji') as 'emoji' | 'custom',
  logo_text: state.config.logo_text || 'MyLauncher',
  logo_image: state.config.logo_image || '',
  logo_image_enabled: state.config.logo_image_enabled || false,
  close_action: (state.config.close_action === 'close' ? 'close' : 'tray') as 'tray' | 'close',
  custom_emojis: [...(state.config.custom_emojis || [])] as string[],
  // Favicon 获取源（多行文本，textarea 用字符串编辑，保存时按行拆分）
  favicon_api_sources_text: (state.config.favicon_api_sources || []).join('\n'),
  // 开机自启（保存时写入注册表）
  autorun: false,
  autorun_silent: false,
})

// 加载时从注册表读实际自启状态（非前端内存）
onMounted(async () => {
  try {
    const st = await get_autorun()
    form.value.autorun = st.enabled
    form.value.autorun_silent = st.silent
  } catch {
    // 读取失败保持默认关闭
  }
})

// Favicon 获取源默认模板（与 Rust default_favicon_api_sources 保持一致）
const DEFAULT_FAVICON_API_SOURCES = [
  'https://a.favicon.im/{url}?larger=true',
  'https://faviconsnap.com/api/favicon?url={url}',
  '{scheme}://{domain}/favicon.ico',
]

// 恢复默认获取源
function resetFaviconApiSources() {
  form.value.favicon_api_sources_text = DEFAULT_FAVICON_API_SOURCES.join('\n')
}

// 自定义 emoji 输入框
const customEmojiInput = ref('')

// 添加自定义 emoji（从输入框粘贴/输入）
function addCustomEmoji() {
  const text = customEmojiInput.value.trim()
  if (!text) return
  // 支持粘贴多个 emoji（连续粘贴也逐个添加）
  // 用 Array.from 拆分多字符 emoji（如旗帜 emoji 由多个码点组成）
  const chars = Array.from(text)
  let added = 0
  for (const ch of chars) {
    // 去重：不在内置库也不在已有自定义列表中
    if (!form.value.custom_emojis.includes(ch)) {
      form.value.custom_emojis.push(ch)
      added++
    }
  }
  if (added > 0) {
    customEmojiInput.value = ''
  } else {
    // 已存在，清空输入
    customEmojiInput.value = ''
  }
}

// 删除自定义 emoji
function removeCustomEmoji(index: number) {
  form.value.custom_emojis.splice(index, 1)
}

// 从剪贴板粘贴 emoji
async function pasteFromClipboard() {
  try {
    const text = await navigator.clipboard.readText()
    if (text) {
      customEmojiInput.value = text.trim()
      addCustomEmoji()
    }
  } catch {
    // 剪贴板权限被拒绝，用户可手动 Ctrl+V 到输入框
  }
}

const saving = ref(false)
const error = ref('')

// 选择 emoji
function selectEmoji(emoji: string) {
  form.value.logo_icon_type = 'emoji'
  form.value.logo_icon = emoji
}

// 上传图标图片
function onIconUploaded(relPath: string) {
  form.value.logo_icon_type = 'custom'
  form.value.logo_icon = relPath
}

// LOGO 图片预览 URL
const logoImagePreview = computed(() => {
  if (!form.value.logo_image) return ''
  return resolveAssetUrl(form.value.logo_image)
})

// 选择 LOGO 图片（复制到 assets/logo/）
async function pickLogoImage() {
  error.value = ''
  try {
    const { pickAndCopyImage } = await import('@/composables/useAssetImagePicker')
    const rel = await pickAndCopyImage('logo')
    if (!rel) return
    form.value.logo_image = rel
    if (!form.value.logo_image_enabled) form.value.logo_image_enabled = true
  } catch (e) {
    error.value = `图片导入失败: ${e}`
  }
}

// 移除 LOGO 图片
async function removeLogoImage() {
  form.value.logo_image = ''
  form.value.logo_image_enabled = false
}

// 保存
async function save() {
  error.value = ''
  // 启用图片但没有图片时，自动回退为不启用
  if (form.value.logo_image_enabled && !form.value.logo_image) {
    form.value.logo_image_enabled = false
  }
  // LOGO 文字为空时回退默认
  if (!form.value.logo_text.trim()) {
    form.value.logo_text = 'MyLauncher'
  }
  // LOGO 图标为空时回退默认
  if (!form.value.logo_icon.trim()) {
    form.value.logo_icon = '🪷'
    form.value.logo_icon_type = 'emoji'
  }
  // Favicon 获取源：按行拆分、去空白；全空时回落默认
  let favicon_sources = form.value.favicon_api_sources_text
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
  if (favicon_sources.length === 0) {
    favicon_sources = [...DEFAULT_FAVICON_API_SOURCES]
    form.value.favicon_api_sources_text = favicon_sources.join('\n')
  }

  saving.value = true
  try {
    // 先写自启注册表（失败则中断，避免配置显示已开但实际未生效）
    try {
      await set_autorun(form.value.autorun, form.value.autorun_silent)
    } catch (e) {
      error.value = `开机自启设置失败: ${e}`
      saving.value = false
      return
    }
    await saveSettings({
      logo_icon: form.value.logo_icon.trim(),
      logo_icon_type: form.value.logo_icon_type,
      logo_text: form.value.logo_text.trim(),
      logo_image: form.value.logo_image,
      logo_image_enabled: form.value.logo_image_enabled,
      close_action: form.value.close_action,
      custom_emojis: form.value.custom_emojis,
      favicon_api_sources: favicon_sources,
    })
    emit('close')
  } catch (e) {
    error.value = `保存失败: ${e}`
  } finally {
    saving.value = false
  }
}

// ─── 过期资源清理 ───

const { showToast } = useToast()

// 扫描/清理进行中
const cleaning = ref(false)
// 扫描结果（null 表示尚未扫描）
const scanResult = ref<{ count: number; sizeText: string } | null>(null)

// 字节数转可读大小
function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}

// 扫描过期资源（无副作用）
async function scanAssets() {
  error.value = ''
  cleaning.value = true
  scanResult.value = null
  try {
    const r = await scan_orphan_assets()
    scanResult.value = { count: r.orphan_count, sizeText: formatBytes(r.orphan_bytes) }
  } catch (e) {
    error.value = `扫描失败: ${e}`
  } finally {
    cleaning.value = false
  }
}

// 清理过期资源（先确认再执行，移入回收站）
async function cleanAssets() {
  if (!scanResult.value || scanResult.value.count === 0) return
  if (!confirm(`确认清理 ${scanResult.value.count} 个过期文件（${scanResult.value.sizeText}）？\n文件将移入回收站，如误删可从回收站还原。`)) return
  error.value = ''
  cleaning.value = true
  try {
    const r = await clean_orphan_assets()
    showToast(`已清理 ${r.deleted_count} 个文件，释放 ${formatBytes(r.deleted_bytes)}`)
    // 清理后重新扫描显示最新状态
    const s = await scan_orphan_assets()
    scanResult.value = { count: s.orphan_count, sizeText: formatBytes(s.orphan_bytes) }
  } catch (e) {
    error.value = `清理失败: ${e}`
  } finally {
    cleaning.value = false
  }
}

</script>

<template>
  <div class="dialog-overlay" @click.self="!saving && $emit('close')">
    <div class="dialog settings-dialog">
      <div class="dialog-header">
        <span>设置</span>
        <button class="btn btn-icon" @click="$emit('close')">✕</button>
      </div>

      <div class="dialog-body">
        <!-- LOGO 设置 -->
        <div class="section-title">LOGO 设置</div>
        <div class="setting-group">
          <div class="form-row">
            <label class="form-label">LOGO 图标</label>
            <div class="form-control">
              <IconPicker
                :current-type="form.logo_icon_type"
                :current-source="form.logo_icon"
                upload-dir="icons/custom"
                :custom-emojis="form.custom_emojis"
                @select-emoji="selectEmoji"
                @upload="onIconUploaded"
              />
            </div>
          </div>
          <div class="form-row">
            <label class="form-label">LOGO 文字</label>
            <div class="form-control">
              <input v-model="form.logo_text" class="input" placeholder="MyLauncher" maxlength="20" />
              <div class="form-hint">留空恢复默认 "MyLauncher"</div>
            </div>
          </div>
          <div class="form-row">
            <label class="form-label">LOGO 图片</label>
            <div class="form-control">
              <div class="logo-image-row">
                <button class="btn" @click="pickLogoImage">📁 选择图片</button>
                <button v-if="form.logo_image" class="btn btn-danger" @click="removeLogoImage">移除</button>
                <label class="checkbox-label">
                  <input v-model="form.logo_image_enabled" type="checkbox" :disabled="!form.logo_image" />
                  启用图片作为 LOGO
                </label>
              </div>
              <div class="form-hint">启用后 LOGO 区域整体显示为图片；未启用则显示图标 + 文字</div>
              <!-- 预览（单图铺满，模拟 LOGO 区域实际效果） -->
              <div v-if="form.logo_image" class="logo-preview" :class="{ disabled: !form.logo_image_enabled }">
                <div class="logo-preview-box">
                  <img v-if="logoImagePreview" :src="logoImagePreview" class="logo-preview-img" alt="LOGO 预览" />
                </div>
                <div class="logo-preview-filename">{{ form.logo_image }}</div>
              </div>
            </div>
          </div>
        </div>

        <!-- 自定义 emoji 图标库 -->
        <div class="section-title">自定义图标库</div>
        <div class="setting-group">
          <div class="custom-emoji-hint">从别处复制 emoji，粘贴到下方输入框即可添加到自定义标签页</div>
          <div class="custom-emoji-input-row">
            <input
              v-model="customEmojiInput"
              class="input"
              placeholder="在此粘贴 emoji"
              @keydown.enter.prevent="addCustomEmoji"
            />
            <button class="btn" @click="pasteFromClipboard">📋 粘贴</button>
            <button class="btn btn-primary" @click="addCustomEmoji">添加</button>
          </div>
          <div v-if="form.custom_emojis.length > 0" class="custom-emoji-list">
            <div
              v-for="(emoji, i) in form.custom_emojis"
              :key="i"
              class="custom-emoji-item"
              @click="selectEmoji(emoji)"
            >
              <span class="custom-emoji-char">{{ emoji }}</span>
              <button class="custom-emoji-remove" @click.stop="removeCustomEmoji(i)">✕</button>
            </div>
          </div>
          <div v-else class="custom-emoji-empty">暂无自定义 emoji</div>
        </div>

        <!-- 关闭按钮行为 -->
        <div class="section-title">关闭按钮</div>
        <div class="setting-group">
          <div class="radio-group">
            <label class="radio-option" :class="{ active: form.close_action === 'tray' }">
              <input v-model="form.close_action" type="radio" value="tray" />
              <span class="radio-text">
                <span class="radio-title">最小化到托盘</span>
                <span class="radio-desc">点击关闭按钮时隐藏窗口，程序保留在系统托盘</span>
              </span>
            </label>
            <label class="radio-option" :class="{ active: form.close_action === 'close' }">
              <input v-model="form.close_action" type="radio" value="close" />
              <span class="radio-text">
                <span class="radio-title">直接退出程序</span>
                <span class="radio-desc">点击关闭按钮时彻底关闭 MyLauncher</span>
              </span>
            </label>
          </div>
        </div>

        <!-- 开机自启 -->
        <div class="section-title">开机自启</div>
        <div class="setting-group">
          <label class="checkbox-label">
            <input v-model="form.autorun" type="checkbox" />
            开机自动启动 MyLauncher
          </label>
          <label class="checkbox-label autorun-sub" :class="{ disabled: !form.autorun }">
            <input v-model="form.autorun_silent" type="checkbox" :disabled="!form.autorun" />
            自启时不显示主界面，隐藏在系统托盘
          </label>
          <div class="form-hint">写入当前用户注册表 Run 键，无需管理员权限；程序移动位置后会自动校正路径</div>
        </div>

        <!-- Favicon 获取源 -->
        <div class="section-title">Favicon 获取源</div>
        <div class="setting-group">
          <div class="form-hint favicon-hint">
            获取网址图标使用的 API 模板，一行一个，按顺序尝试（任一成功即用）。占位符：{url} 完整链接、{domain} 域名、{scheme} 协议。API 失效时可自行替换
          </div>
          <textarea
            v-model="form.favicon_api_sources_text"
            class="textarea favicon-textarea"
            rows="4"
            spellcheck="false"
            placeholder="https://a.favicon.im/{url}?larger=true"
          ></textarea>
          <div class="favicon-actions">
            <button class="btn" @click="resetFaviconApiSources">恢复默认</button>
          </div>
        </div>

        <!-- 过期资源清理 -->
        <div class="section-title">资源清理</div>
        <div class="setting-group">
          <div class="form-hint clean-hint">
            清理图标、封面等资源中未被任何条目、分类或 LOGO 引用的过期文件（如目录扫描、书签导入后未使用的图标），文件将移入回收站
          </div>
          <div class="clean-actions">
            <button class="btn" :disabled="cleaning" @click="scanAssets">
              {{ cleaning ? '处理中...' : '🔍 扫描' }}
            </button>
            <button
              v-if="scanResult && scanResult.count > 0"
              class="btn btn-danger"
              :disabled="cleaning"
              @click="cleanAssets"
            >
              🗑 清理
            </button>
          </div>
          <div v-if="scanResult" class="clean-result" :class="{ empty: scanResult.count === 0 }">
            <template v-if="scanResult.count > 0">
              发现 <b>{{ scanResult.count }}</b> 个过期文件，共 <b>{{ scanResult.sizeText }}</b>
            </template>
            <template v-else>
              没有过期文件，资源均已使用
            </template>
          </div>
        </div>
      </div>

      <div class="dialog-footer">
        <span v-if="error" class="error-text">{{ error }}</span>
        <button class="btn" :disabled="saving" @click="$emit('close')">取消</button>
        <button class="btn btn-primary" :disabled="saving" @click="save">{{ saving ? '保存中...' : '保存' }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-dialog {
  width: 480px;
}

.setting-group {
  margin-bottom: 20px;
}

.form-control {
  flex: 1;
  min-width: 0;
}

/* ─── LOGO 图片行 ─── */
.logo-image-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.checkbox-label {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  cursor: pointer;
  color: var(--color-text);
}

.checkbox-label input[type='checkbox'] {
  cursor: pointer;
  accent-color: var(--color-primary);
}

.checkbox-label input[type='checkbox']:disabled {
  cursor: not-allowed;
}

/* ─── LOGO 预览 ─── */
.logo-preview {
  margin-top: 10px;
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo-preview.disabled {
  opacity: 0.5;
}

.logo-preview-box {
  width: 180px;
  height: 48px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--gradient-top);
  display: flex;
  align-items: center;
  flex-shrink: 0;
  position: relative;
  overflow: hidden;
}

.logo-preview-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.logo-preview-filename {
  font-size: 11.5px;
  color: var(--color-text-tertiary);
  word-break: break-all;
}

/* ─── 开机自启 ─── */
.autorun-sub {
  margin-top: 8px;
  margin-left: 22px;
}

.autorun-sub.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* ─── 单选组 ─── */
.radio-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.radio-option {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 10px 14px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition);
}

.radio-option:hover {
  background: var(--color-bg-hover);
}

.radio-option.active {
  border-color: var(--color-primary);
  background: var(--color-primary-bg);
}

.radio-option input[type='radio'] {
  margin-top: 3px;
  cursor: pointer;
  accent-color: var(--color-primary);
}

.radio-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.radio-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text);
}

.radio-desc {
  font-size: 11.5px;
  color: var(--color-text-tertiary);
}

.error-text {
  margin-right: auto;
  font-size: 12px;
  color: var(--color-danger);
}

/* ─── 自定义 emoji 图标库 ─── */
.custom-emoji-hint {
  font-size: 12px;
  color: var(--color-text-tertiary);
  margin-bottom: 8px;
}

.custom-emoji-input-row {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 10px;
}

.custom-emoji-input-row .input {
  flex: 1;
  min-width: 0;
}

.custom-emoji-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.custom-emoji-item {
  position: relative;
  width: 36px;
  height: 36px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-bg-card);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all var(--transition);
}

.custom-emoji-item:hover {
  border-color: var(--color-primary);
  background: var(--color-primary-bg);
}

.custom-emoji-char {
  font-size: 18px;
  line-height: 1;
}

.custom-emoji-remove {
  position: absolute;
  top: -4px;
  right: -4px;
  width: 16px;
  height: 16px;
  border: none;
  border-radius: 50%;
  background: var(--color-danger);
  color: #fff;
  font-size: 9px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  padding: 0;
  opacity: 0;
  transition: opacity var(--transition);
}

.custom-emoji-item:hover .custom-emoji-remove {
  opacity: 1;
}

.custom-emoji-empty {
  font-size: 12px;
  color: var(--color-text-tertiary);
  padding: 8px 0;
}

/* ─── Favicon 获取源 ─── */
.favicon-hint {
  margin-bottom: 8px;
  line-height: 1.6;
}

.favicon-textarea {
  width: 100%;
  font-size: 12.5px;
  font-family: 'Cascadia Code', 'Consolas', monospace;
  line-height: 1.7;
  resize: vertical;
  min-height: 92px;
}

.favicon-actions {
  margin-top: 8px;
  display: flex;
  align-items: center;
  gap: 8px;
}

/* ─── 过期资源清理 ─── */
.clean-hint {
  margin-bottom: 10px;
}

.clean-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.clean-result {
  margin-top: 10px;
  padding: 8px 12px;
  border-radius: var(--radius-sm);
  background: var(--color-primary-bg);
  border: 1px solid var(--color-primary);
  font-size: 12.5px;
  color: var(--color-text);
}

.clean-result.empty {
  background: var(--color-bg-hover);
  border-color: var(--color-border);
  color: var(--color-text-tertiary);
}

.clean-result b {
  color: var(--color-primary);
  font-weight: 600;
}

.clean-result.empty b {
  color: var(--color-text-tertiary);
}

</style>

