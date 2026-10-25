<script setup lang="ts">
import { reactive, ref, watch } from 'vue'
import type { Task } from '../api/commands'
import FormDrawer from './FormDrawer.vue'
import { useProjectsStore } from '../stores/projects'
import { useTasksStore } from '../stores/tasks'

const props = defineProps<{ task: Task | null }>()
const emit = defineEmits<{ close: []; saved: [] }>()

const projects = useProjectsStore()
const tasks = useTasksStore()

const form = reactive({
  title: '',
  note: '',
  projectId: null as number | null,
  priority: 2,
  dueDate: '',
  repeatRule: '',
  status: '待办',
})
const error = ref('')

watch(
  () => props.task,
  (t) => {
    if (t) {
      form.title = t.title
      form.note = t.note ?? ''
      form.projectId = t.projectId
      form.priority = t.priority
      form.dueDate = t.dueDate ?? ''
      form.repeatRule = t.repeatRule ?? ''
      form.status = t.status
      error.value = ''
    }
  },
  { immediate: true },
)

const REPEAT_OPTIONS = [
  { value: '', label: '不重复' },
  { value: 'daily', label: '每天' },
  { value: 'weekly:1', label: '每周一' },
  { value: 'weekly:2', label: '每周二' },
  { value: 'weekly:3', label: '每周三' },
  { value: 'weekly:4', label: '每周四' },
  { value: 'weekly:5', label: '每周五' },
  { value: 'weekly:6', label: '每周六' },
  { value: 'weekly:7', label: '每周日' },
]

async function save() {
  if (!props.task) return
  if (!form.title.trim()) {
    error.value = '标题不能为空'
    return
  }
  try {
    await tasks.update(props.task.id, {
      title: form.title.trim(),
      note: form.note.trim() || null,
      projectId: form.projectId,
      priority: form.priority,
      dueDate: form.dueDate || null,
      repeatRule: form.repeatRule || null,
      status: form.status,
    })
    emit('saved')
  } catch (e) {
    error.value = String(e)
  }
}

async function remove() {
  if (!props.task) return
  await tasks.remove(props.task.id)
  emit('saved')
}
</script>

<template>
  <FormDrawer v-if="task" title="编辑任务" @close="emit('close')" @save="save" @delete="remove">
    <div class="form-grid">
      <label class="field span2">
        <span>标题</span>
        <input v-model="form.title" type="text" />
      </label>
      <label class="field span2">
        <span>备注</span>
        <textarea v-model="form.note" rows="3" />
      </label>
      <label class="field span2">
        <span>课题</span>
        <select v-model="form.projectId">
          <option :value="null">收件箱（不关联）</option>
          <option v-for="p in projects.projects" :key="p.id" :value="p.id">{{ p.name }}</option>
        </select>
      </label>
      <label class="field">
        <span>优先级</span>
        <select v-model.number="form.priority">
          <option :value="0">!高</option>
          <option :value="1">!中</option>
          <option :value="2">低（默认）</option>
        </select>
      </label>
      <label class="field">
        <span>截止</span>
        <input v-model="form.dueDate" type="date" />
      </label>
      <label class="field">
        <span>重复</span>
        <select v-model="form.repeatRule">
          <option v-for="o in REPEAT_OPTIONS" :key="o.value" :value="o.value">
            {{ o.label }}
          </option>
        </select>
      </label>
      <label class="field">
        <span>状态</span>
        <select v-model="form.status">
          <option>待办</option>
          <option>进行中</option>
          <option>完成</option>
          <option>放弃</option>
        </select>
      </label>
      <p v-if="error" class="error">{{ error }}</p>
    </div>
  </FormDrawer>
</template>
