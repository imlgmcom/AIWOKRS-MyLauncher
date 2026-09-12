// ════════════════════════════════════════════
// Tauri API 封装层
// ════════════════════════════════════════════

import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import type {
  AllData, AppConfig, AssetCleanResult, AutorunStatus, BatchIconSummary, Category, Entry, Environment,
  ExeInfo, FileInfo, IconRefreshResult, ImportScanSummary, InitResult, LnkInfo, PathInfo, ScanResult, ConvertPathResult,
  BookmarkParseResult, AppxAppInfo, SteamAppScanResult
} from '@/types'

// ─── 数据持久化 ───

export async function init_data_dir(): Promise<InitResult> {
  return invoke<InitResult>('init_data_dir')
}

export async function load_all_data(): Promise<AllData> {
  return invoke<AllData>('load_all_data')
}

export async function save_entries(entries: Entry[]): Promise<void> {
  await invoke('save_entries', { entries })
}

export async function save_categories(categories: Category[]): Promise<void> {
  await invoke('save_categories', { categories })
}

export async function save_environments(environments: Environment[]): Promise<void> {
  await invoke('save_environments', { environments })
}

export async function save_config(config: AppConfig): Promise<void> {
  await invoke('save_config', { config })
}

export async function get_exe_dir(): Promise<string> {
  return invoke<string>('get_exe_dir')
}

export async function get_now(): Promise<string> {
  return invoke<string>('get_now')
}

// ─── 路径解析 ───

export async function resolve_path(entry: Entry, currentEnvId: string): Promise<PathInfo> {
  return invoke<PathInfo>('resolve_path', { entry, currentEnvId })
}

export async function check_paths_batch(entries: Entry[], currentEnvId: string): Promise<boolean[]> {
  return invoke<boolean[]>('check_paths_batch', { entries, currentEnvId })
}

// ─── 路径模式转换 ───

export async function convert_path_mode(entry: Entry, currentEnvId: string, targetMode: string): Promise<ConvertPathResult> {
  return invoke<ConvertPathResult>('convert_path_mode', { entry, currentEnvId, targetMode })
}

// ─── 启动程序 ───

export async function launch_program(entry: Entry, currentEnvId: string): Promise<boolean> {
  return invoke<boolean>('launch_program', { entry, currentEnvId })
}

// ─── 打开所在目录 ───

export async function open_in_explorer(entry: Entry, currentEnvId: string): Promise<boolean> {
  return invoke<boolean>('open_in_explorer', { entry, currentEnvId })
}

// ─── 图标提取 ───

export async function get_exe_info(exePath: string): Promise<ExeInfo> {
  return invoke<ExeInfo>('get_exe_info', { exePath })
}

// ─── 文件/文件夹信息 ───

export async function get_file_info(filePath: string): Promise<FileInfo> {
  return invoke<FileInfo>('get_file_info', { filePath })
}

// ─── 快捷方式解析 ───

export async function resolve_lnk(lnkPath: string): Promise<LnkInfo> {
  return invoke<LnkInfo>('resolve_lnk', { lnkPath })
}

// ─── 文件夹扫描 ───

export async function scan_directory(dirPath: string, maxDepth?: number): Promise<ScanResult> {
  return invoke<ScanResult>('scan_directory', { dirPath, maxDepth })
}

// ─── Steam 游戏按 ID 扫描 ───

/** 按 Steam 应用 ID 扫描已安装游戏（注册表定位 Steam → 遍历所有库 → 解析 appmanifest），
 *  返回名称/exe 路径/封面/图标；游戏不存在或未安装时抛错 */
export async function scan_steam_app(appId: string): Promise<SteamAppScanResult> {
  return invoke<SteamAppScanResult>('scan_steam_app', { appId })
}

// ─── 扫描结果批量导入 ───

/** 批量导入扫描结果（后端多线程提取图标/下载封面/构造 Entry 后一次写盘，
 *  逐条推送 import-progress 事件，完成后返回整体统计与已写盘的新 Entry） */
export async function import_scan_items(items: ExeInfo[], categoryId: string, currentEnvId: string): Promise<ImportScanSummary> {
  return invoke<ImportScanSummary>('import_scan_items', { items, categoryId, currentEnvId })
}

// ─── 文件操作 ───

export async function copy_file_to_assets(sourcePath: string, targetSubdir: string): Promise<string> {
  return invoke<string>('copy_file_to_assets', { sourcePath, targetSubdir })
}

// ─── Favicon ───

export async function fetch_favicon(url: string): Promise<string> {
  return invoke<string>('fetch_favicon', { url })
}

// ─── 浏览器书签解析 ───

export async function parse_bookmarks(filePath: string): Promise<BookmarkParseResult> {
  return invoke<BookmarkParseResult>('parse_bookmarks', { filePath })
}

// ─── APPX / 系统功能 ───

export async function list_appx_apps(): Promise<AppxAppInfo[]> {
  return invoke<AppxAppInfo[]>('list_appx_apps')
}

export async function extract_appx_icon(appKey: string, shellUri: string): Promise<string> {
  return invoke<string>('extract_appx_icon', { appKey, shellUri })
}

export async function extract_uri_icon(target: string): Promise<string> {
  return invoke<string>('extract_uri_icon', { target })
}

// ─── 过期资源清理 ───

/** 扫描未被引用的过期图标/封面（无副作用，仅统计） */
export async function scan_orphan_assets(): Promise<AssetCleanResult> {
  return invoke<AssetCleanResult>('scan_orphan_assets')
}

/** 删除未被引用的过期图标/封面（移入回收站） */
export async function clean_orphan_assets(): Promise<AssetCleanResult> {
  return invoke<AssetCleanResult>('clean_orphan_assets')
}

/** 删除条目后的资源联动清理：传入被删条目的图标/封面相对路径，仍被引用的跳过，未被引用的移入回收站 */
export async function cleanup_deleted_assets(sources: string[]): Promise<number> {
  return invoke<number>('cleanup_deleted_assets', { sources })
}

// ─── 图标重新提取 ───

/** 重新提取条目图标（按类型分派：程序/文件夹/文件/网址/APPX） */
export async function refresh_entry_icon(entry: Entry, currentEnvId: string): Promise<IconRefreshResult> {
  return invoke<IconRefreshResult>('refresh_entry_icon', { entry, currentEnvId })
}

/** 批量重新提取图标（后端多线程，逐条推送 batch-icon-progress 事件，完成后返回整体统计） */
export async function batch_refresh_icons(entries: Entry[], currentEnvId: string): Promise<BatchIconSummary> {
  return invoke<BatchIconSummary>('batch_refresh_icons', { entries, currentEnvId })
}

// ─── 开机自启 ───

/** 设置开机自启（enabled=false 删除注册表值；silent 控制自启时是否隐藏主窗口） */
export async function set_autorun(enabled: boolean, silent: boolean): Promise<void> {
  await invoke('set_autorun', { enabled, silent })
}

/** 查询当前自启状态（读注册表实际值） */
export async function get_autorun(): Promise<AutorunStatus> {
  return invoke<AutorunStatus>('get_autorun')
}

// ─── 文件选择对话框 ───

export async function pick_file(filters?: { name: string; extensions: string[] }[], defaultPath?: string): Promise<string | null> {
  const result = await open({ filters, multiple: false, defaultPath: defaultPath || undefined })
  return result as string | null
}

export async function pick_directory(): Promise<string | null> {
  const result = await open({ directory: true, multiple: false })
  return result as string | null
}

export async function save_file(defaultName: string, filters?: { name: string; extensions: string[] }[]): Promise<string | null> {
  const result = await save({ defaultPath: defaultName, filters })
  return result as string | null
}
