// ════════════════════════════════════════════
// 外部点击/Escape 关闭
// ════════════════════════════════════════════

import { ref, onMounted, onBeforeUnmount, type Ref } from 'vue'

/**
 * 管理可关闭浮层（下拉菜单、右键菜单等）。
 * 调用方传入 triggerRef（触发器 DOM 引用），返回 isOpen 和关闭逻辑。
 */
export function useDismissable(triggerRef?: Ref<HTMLElement | null>) {
  const isOpen = ref(false)

  function open() { isOpen.value = true }
  function close() { isOpen.value = false }
  function toggle() { isOpen.value = !isOpen.value }

  function onGlobalClick(e: MouseEvent) {
    if (!isOpen.value) return
    const el = triggerRef?.value
    if (el && el.contains(e.target as Node)) return
    close()
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close()
  }

  onMounted(() => {
    window.addEventListener('click', onGlobalClick)
    window.addEventListener('keydown', onKeydown)
  })
  onBeforeUnmount(() => {
    window.removeEventListener('click', onGlobalClick)
    window.removeEventListener('keydown', onKeydown)
  })

  return { isOpen, open, close, toggle }
}
