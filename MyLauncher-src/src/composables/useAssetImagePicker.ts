// ════════════════════════════════════════════
// 图片资源上传（选择文件 → 复制到 assets → 返回相对路径）
// ════════════════════════════════════════════

import * as api from '@/api'

/** 默认图片过滤器 */
const DEFAULT_IMAGE_FILTERS = [{ name: '图片', extensions: ['png', 'jpg', 'jpeg', 'webp', 'ico', 'bmp', 'gif'] }]

/**
 * 选择图片文件并复制到 assets 指定子目录，返回相对路径。
 * @param targetSubdir assets 下的子目录（如 'icons/custom'）
 * @param extensions 可选的自定义扩展名列表，默认使用通用图片格式
 * 用户取消选择时返回 null。
 */
export async function pickAndCopyImage(targetSubdir: string, extensions?: string[]): Promise<string | null> {
  const filters = extensions
    ? [{ name: '图片', extensions }]
    : DEFAULT_IMAGE_FILTERS
  const path = await api.pick_file(filters)
  if (!path) return null
  return await api.copy_file_to_assets(path, targetSubdir)
}
