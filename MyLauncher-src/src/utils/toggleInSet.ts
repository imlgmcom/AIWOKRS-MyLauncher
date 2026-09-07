// ════════════════════════════════════════════
// Set 切换工具
// ════════════════════════════════════════════

/**
 * 切换 Set 中某元素的存在状态：存在则删除，不存在则添加。
 */
export function toggleInSet<T>(set: Set<T>, value: T): void {
  if (set.has(value)) set.delete(value)
  else set.add(value)
}
