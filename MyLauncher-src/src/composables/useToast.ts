// ════════════════════════════════════════════
// 全局 Toast 提示
// ════════════════════════════════════════════

import { ref } from 'vue'

const toastMsg = ref('')
const toastVisible = ref(false)
let timer: ReturnType<typeof setTimeout> | null = null

export function useToast() {
  function showToast(msg: string) {
    toastMsg.value = msg
    toastVisible.value = true
    if (timer) clearTimeout(timer)
    timer = setTimeout(() => { toastVisible.value = false }, 2000)
  }

  return { toastMsg, toastVisible, showToast }
}
