<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'

// 抽屉外壳：遮罩 + 标题/关闭 + 滚动表单体 + 底部「删除/保存」，
// 破坏性操作内置两段式确认（DESIGN.md §4/§5）。表单字段放默认插槽，
// 用 .form-grid / .field（全局 ui.css）排两列网格。
withDefaults(defineProps<{ title: string; deletable?: boolean; saveLabel?: string }>(), {
  deletable: true,
  saveLabel: '保存',
})

const emit = defineEmits<{ close: []; save: []; delete: [] }>()
const confirming = ref(false)

function onDelete() {
  if (!confirming.value) {
    confirming.value = true
    return
  }
  emit('delete')
}

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close')
}

onMounted(() => window.addEventListener('keydown', onKey))
onUnmounted(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <div class="mask" @click.self="emit('close')">
    <div class="drawer">
      <div class="d-head">
        <span>{{ title }}</span>
        <button class="close" type="button" @click="emit('close')">✕</button>
      </div>
      <div class="d-body">
        <slot />
      </div>
      <div class="d-foot">
        <button v-if="deletable" class="btn danger" type="button" @click="onDelete">
          {{ confirming ? '确认删除？' : '删除' }}
        </button>
        <span v-else />
        <button class="btn primary" type="button" @click="emit('save')">{{ saveLabel }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.mask {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  justify-content: flex-end;
  z-index: 50;
}

.drawer {
  width: 380px;
  max-width: 100%;
  height: 100%;
  background: var(--card);
  border-left: 1px solid var(--line);
  display: flex;
  flex-direction: column;
}

.d-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 13px 16px;
  border-bottom: 1px solid var(--line);
  font-weight: 500;
}

.close {
  border: none;
  background: none;
  color: var(--muted-fg);
  cursor: pointer;
}

.d-body {
  flex: 1;
  overflow-y: auto;
  padding: 14px 16px;
}

.d-foot {
  display: flex;
  justify-content: space-between;
  padding: 12px 16px;
  border-top: 1px solid var(--line);
}
</style>
