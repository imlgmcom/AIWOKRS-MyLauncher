// ════════════════════════════════════════════
// 数据模型类型定义
// ════════════════════════════════════════════

export interface ImageResource {
  type: 'custom' | 'default' | 'extracted' | 'emoji'
  source: string
}

export interface CoverResource {
  enabled: boolean
  source: string
}

/** 条目类型：程序/网址/文件夹/文件/系统功能/APPX应用/Steam游戏 */
export type EntryType = 'program' | 'url' | 'folder' | 'file' | 'system' | 'appx' | 'steam'

export interface Entry {
  id: string
  name: string
  type: EntryType
  category_id: string
  relative_path: string
  absolute_paths: Record<string, string>
  path_mode: 'relative' | 'absolute'
  launch_args: string
  working_directory: string
  window_style: 'normal' | 'maximized' | 'minimized'
  run_as_admin: boolean
  url: string
  icon: ImageResource
  cover: CoverResource
  tags: string[]
  notes: string
  last_used: string
  add_time: string
}

export interface Category {
  id: string
  name: string
  parent_id: string | null
  icon: ImageResource
  view_mode: ViewMode
  sort_order: number
  is_top_level: boolean
}

export interface Environment {
  id: string
  name: string
  description: string
  drive_mapping: Record<string, string>
  is_current: boolean
}

export interface AppConfig {
  version: string
  /** LOGO 图标（emoji 字符或自定义图片相对路径，按 logo_icon_type 区分），默认 🪷 */
  logo_icon: string
  /** LOGO 图标类型: "emoji"（字符） | "custom"（上传的图片相对路径） */
  logo_icon_type: 'emoji' | 'custom'
  /** LOGO 文字，默认 "MyLauncher" */
  logo_text: string
  /** LOGO 图片相对路径（如 "assets/logo/xxx.png"），空表示未设置 */
  logo_image: string
  /** 是否启用 LOGO 图片（启用后 LOGO 区域整体显示为图片） */
  logo_image_enabled: boolean
  /** 关闭按钮行为: "tray"（隐藏到托盘） | "close"（直接退出） */
  close_action: 'tray' | 'close'
  /** 用户自定义 emoji 列表（在设置中粘贴添加） */
  custom_emojis: string[]
  /** Favicon 获取源 API 模板列表（一行一个），支持占位符：{url} 完整链接 | {domain} 域名 | {scheme} 协议；按序尝试任一成功即用 */
  favicon_api_sources: string[]
  /** 界面主题: "light" 明亮 | "dark" 暗黑 | "system" 跟随系统 */
  theme: 'light' | 'dark' | 'system'
  /** 侧导航是否折叠（隐藏） */
  sidebar_collapsed: boolean
  /** 竖向卡片封面宽高比例，如 "16:9" | "4:3" | "1:1" | "3:4" | "9:16" */
  card_cover_ratio: string
  /** 瀑布流封面图最大高度（px），默认 480 */
  masonry_max_height: number
}

/** 开机自启状态（读注册表实际值） */
export interface AutorunStatus {
  enabled: boolean
  silent: boolean
}

export interface AllData {
  config: AppConfig
  environments: Environment[]
  categories: Category[]
  entries: Entry[]
}

/** Steam 游戏信息（识别为 Steam 游戏时附带） */
export interface SteamGameInfo {
  /** Steam 应用 ID（appmanifest_*.acf 的 appid） */
  app_id: string
  /** 游戏安装目录名（appmanifest 的 installdir） */
  install_dir: string
  /** 游戏 exe 绝对路径（可能为空） */
  exe_path: string
  /** 封面图相对路径（assets/covers/steam/{appid}.jpg，空表示未获取到；扫描阶段为空，导入时填充） */
  cover_path: string
  /** Steam 安装目录（导入阶段定位本地封面缓存用，可能为空） */
  steam_dir: string
}

export interface ExeInfo {
  name: string
  icon_path: string
  suggested_path_mode: string
  relative_path: string
  absolute_path: string
  file_exists: boolean
  /** Steam 游戏信息（识别为 Steam 游戏时填充，否则为 null） */
  steam?: SteamGameInfo | null
  /** 该 Steam 游戏发现多个候选 exe（后端已展开为多条，红色提示用户勾选） */
  multi_exe?: boolean
  /** 是否主程序 exe（多候选时的第一个，默认选中；单 exe 时为 true） */
  is_primary?: boolean
}

/** 按 Steam 应用 ID 扫描单个游戏的结果 */
export interface SteamAppScanResult {
  /** 游戏名称（appmanifest 的 name） */
  name: string
  /** 游戏 exe 绝对路径（可能为空） */
  exe_path: string
  /** 封面图相对路径（assets/covers/steam/{appid}.jpg，空表示未获取到） */
  cover_path: string
  /** 图标相对路径（从游戏 exe 提取，可能为空） */
  icon_path: string
}

/** 文件/文件夹信息 */
export interface FileInfo {
  name: string
  entry_type: string  // "folder" | "file"
  icon_path: string
  suggested_path_mode: string
  relative_path: string
  absolute_path: string
  file_exists: boolean
}

/** 快捷方式解析结果 */
export interface LnkInfo {
  name: string
  target_path: string
  target_exists: boolean
  working_directory: string
  arguments: string
  window_style: string
  description: string
  icon_path: string
  suggested_path_mode: string
  relative_path: string
}

export interface ScanResult {
  results: ExeInfo[]
  total: number
}

export interface PathInfo {
  resolved: string
  exists: boolean
  path_mode: string
}

export interface ConvertPathResult {
  success: boolean
  message: string
  new_path_mode: string
  new_relative_path: string
  new_absolute_path: string
}

export interface InitResult {
  data_dir: string
}

/** 浏览器书签链接 */
export interface BookmarkLink {
  title: string
  url: string
  /** 已保存的图标相对路径（空字符串表示无图标） */
  icon_path: string
}

/** 浏览器书签文件夹（可嵌套） */
export interface BookmarkFolder {
  title: string
  links: BookmarkLink[]
  children: BookmarkFolder[]
}

/** 书签解析结果 */
export interface BookmarkParseResult {
  folders: BookmarkFolder[]
  root_links: BookmarkLink[]
  total_links: number
  total_folders: number
}

// 视图模式
export type ViewMode = 'vertical_card' | 'horizontal_card' | 'icon_grid' | 'table' | 'waterfall'

/** APPX / UWP 应用信息（来自 Get-StartApps 枚举） */
export interface AppxAppInfo {
  /** 显示名称 */
  name: string
  /** shell:AppsFolder\{family_name}!{app_id} */
  shell_uri: string
  /** 应用唯一标识（安全字符版 AUMID），用于图标缓存命名 */
  app_key: string
  /** 提取的图标相对路径（空表示未提取） */
  icon_path: string
}

/** 过期资源扫描/清理结果 */
export interface AssetCleanResult {
  /** 过期文件数 */
  orphan_count: number
  /** 过期文件总字节数 */
  orphan_bytes: number
  /** 实际删除的文件数（仅清理时有值） */
  deleted_count: number
  /** 实际释放的字节数（仅清理时有值） */
  deleted_bytes: number
}

/** 图标重提取结果 */
export interface IconRefreshResult {
  success: boolean
  /** 新图标相对路径（失败时为空） */
  icon_path: string
  /** 失败原因（成功时为空） */
  message: string
}

/** 批量重置图标 - 单条状态 */
export type BatchIconStatus = 'success' | 'skipped' | 'failed'

/** 批量重置图标 - 事件推送的单条进度 */
export interface BatchIconProgressEvent {
  done: number
  total: number
  entry_id: string
  entry_name: string
  entry_type: string
  status: BatchIconStatus
  /** 跳过或失败原因（成功时为空） */
  message: string
  /** 成功时的新图标相对路径 */
  icon_path: string
}

/** 批量重置图标 - 单条结果 */
export interface BatchIconItemResult {
  entry_id: string
  entry_name: string
  entry_type: string
  status: BatchIconStatus
  /** 跳过或失败原因（成功时为空） */
  message: string
  /** 成功时的新图标相对路径 */
  icon_path: string
}

/** 批量重置图标 - 整体统计（后端批量命令返回值） */
export interface BatchIconSummary {
  total: number
  success: number
  skipped: number
  failed: number
  results: BatchIconItemResult[]
}

/** 批量导入扫描结果 - 事件推送的单条进度 */
export interface ImportProgressEvent {
  done: number
  total: number
  /** 当前正在导入的条目名 */
  name: string
  /** success | failed */
  status: 'success' | 'failed'
  /** 失败或提示信息（成功且无提示时为空） */
  message: string
}

/** 批量导入扫描结果 - 整体统计（后端命令返回值） */
export interface ImportScanSummary {
  total: number
  success: number
  failed: number
  /** 新导入并已写盘的 Entry（含 id，前端直接同步内存状态） */
  entries: Entry[]
}
