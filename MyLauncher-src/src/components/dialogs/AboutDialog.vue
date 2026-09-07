<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getVersion } from '@tauri-apps/api/app'

const emit = defineEmits<{
  (e: 'close'): void
}>()

const appVersion = ref('')

onMounted(async () => {
  try {
    appVersion.value = await getVersion()
  } catch {
    appVersion.value = '1.0.0'
  }
})
</script>

<template>
  <div class="dialog-overlay" @click.self="emit('close')">
    <div class="dialog about-dialog">
      <div class="dialog-header">
        <span>关于</span>
        <button class="btn btn-icon" @click="emit('close')">✕</button>
      </div>

      <div class="dialog-body about-body">
        <div class="about-logo">
          <div class="about-logo-icon">🪷</div>
        </div>
        <div class="about-name">MyLauncher</div>
        <div class="about-version">版本 {{ appVersion }}</div>
        <div class="about-slogan">纯绿色 无广告 无后门 不自动更新</div>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-primary" @click="emit('close')">确定</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.about-dialog {
  width: 380px;
}

.about-body {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 28px 24px 20px;
  gap: 6px;
}

.about-logo {
  margin-bottom: 8px;
}

.about-logo-icon {
  width: 64px;
  height: 64px;
  border-radius: 16px;
  background: linear-gradient(135deg, #4f83f5 0%, #0078d4 100%);
  box-shadow: 0 4px 16px rgba(79, 131, 245, 0.35);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 32px;
}

.about-name {
  font-size: 20px;
  font-weight: 700;
  letter-spacing: 0.3px;
  margin-top: 4px;
}

.about-version {
  font-size: 13px;
  color: var(--color-text-secondary);
}

.about-slogan {
  font-size: 12px;
  color: var(--color-text-tertiary);
  margin-top: 6px;
  letter-spacing: 0.5px;
}
</style>
