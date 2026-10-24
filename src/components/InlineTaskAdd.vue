<script setup lang="ts">
import { ref } from 'vue'
import { createTask, parseQuick } from '../api/commands'

const props = defineProps<{ placeholder?: string }>()
const emit = defineEmits<{ added: [] }>()

const text = ref('')
const busy = ref(false)

async function submit() {
  const raw = text.value.trim()
  if (!raw || busy.value) return
  busy.value = true
  try {
    const d = await parseQuick(raw, 'task')
    await createTask({
      title: d.title,
      projectId: d.projectMatched,
      priority: d.priority ?? 2,
      dueDate: d.date,
      repeatRule: null,
    })
    text.value = ''
    emit('added')
  } finally {
    busy.value = false
  }
}

const placeholderText = props.placeholder ?? '＋ 添加任务，支持 #课题 !高 明天'
</script>

<template>
  <input
    v-model="text"
    class="inline-add"
    :placeholder="placeholderText"
    @keydown.enter.prevent="submit"
  />
</template>

<style scoped>
.inline-add {
  width: 100%;
  padding: 8px 12px;
  border: 1px dashed var(--line);
  border-radius: 8px;
  background: transparent;
  outline: none;
  color: var(--muted-fg);
  margin-bottom: 6px;
}

.inline-add:focus {
  border-style: solid;
  border-color: var(--accent);
  color: var(--fg);
}
</style>
