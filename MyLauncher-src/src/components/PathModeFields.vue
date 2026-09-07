<script setup lang="ts">
const pathMode = defineModel<'relative' | 'absolute'>('pathMode', { required: true })
const relativePath = defineModel<string>('relativePath', { required: true })
const absolutePath = defineModel<string>('absolutePath', { required: true })
defineProps<{
  isDifferentDrive: boolean
}>()
</script>

<template>
  <div class="form-row">
    <label class="form-label">路径模式</label>
    <div class="radio-group">
      <label :class="{ 'radio-disabled': isDifferentDrive }">
        <input type="radio" v-model="pathMode" value="relative" :disabled="isDifferentDrive" /> 相对路径
      </label>
      <label><input type="radio" v-model="pathMode" value="absolute" /> 绝对路径</label>
    </div>
    <span v-if="isDifferentDrive" class="drive-hint">不同分区，只能使用绝对路径</span>
  </div>

  <div class="form-row" v-if="!isDifferentDrive">
    <label class="form-label">相对路径</label>
    <input
      v-model="relativePath"
      class="input"
      placeholder="相对于 exe 目录的路径"
    />
  </div>

  <div class="form-row">
    <label class="form-label">绝对路径</label>
    <input
      v-model="absolutePath"
      class="input"
      placeholder="当前环境的绝对路径"
    />
  </div>
</template>

<style scoped>
/* PathModeFields 的 form-label 比全局宽 10px（与 EntryEditor 一致） */
.form-label {
  width: 80px;
}

.radio-group {
  display: flex;
  gap: 16px;
  padding-top: 7px;
}

.radio-group label {
  font-size: 13px;
  cursor: pointer;
}

.radio-disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.radio-disabled input {
  cursor: not-allowed;
}

.drive-hint {
  font-size: 11px;
  color: var(--color-warning, #d97706);
  padding-top: 7px;
}
</style>
