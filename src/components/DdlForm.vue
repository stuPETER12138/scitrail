<script setup lang="ts">
import { reactive, ref, watch } from 'vue'
import type { Ddl } from '../api/commands'
import { DDL_CATEGORIES, useTimeStore } from '../stores/time'
import { useProjectsStore } from '../stores/projects'

const props = defineProps<{ ddl: Ddl | null }>()
const emit = defineEmits<{ saved: []; cancel: [] }>()

const time = useTimeStore()
const projects = useProjectsStore()
const error = ref('')

const form = reactive({
  name: '',
  category: '会议截稿',
  date: '',
  time: '',
  projectId: null as number | null,
  repeatYearly: false,
  note: '',
})

watch(
  () => props.ddl,
  (d) => {
    if (d) {
      form.name = d.name
      form.category = d.category ?? '其他'
      form.date = d.dueDate.slice(0, 10)
      form.time = d.dueDate.includes(' ') ? d.dueDate.split(' ')[1] : ''
      form.projectId = d.projectId
      form.repeatYearly = d.repeatYearly
      form.note = d.note ?? ''
    } else {
      form.name = ''
      form.category = '会议截稿'
      form.date = ''
      form.time = ''
      form.projectId = null
      form.repeatYearly = false
      form.note = ''
    }
    error.value = ''
  },
  { immediate: true },
)

async function save() {
  if (!form.name.trim() || !form.date) {
    error.value = '名称与截止日期必填'
    return
  }
  const dueDate = form.time.trim() ? `${form.date} ${form.time.trim()}` : form.date
  const input = {
    name: form.name.trim(),
    category: form.category,
    dueDate,
    projectId: form.projectId,
    repeatYearly: form.repeatYearly,
    note: form.note.trim() || null,
  }
  try {
    if (props.ddl) await time.update(props.ddl.id, input)
    else await time.createD(input)
    emit('saved')
  } catch (e) {
    error.value = String(e)
  }
}
</script>

<template>
  <form class="form" @submit.prevent="save">
    <div class="grid">
      <label class="field span2">
        <span>名称</span>
        <input v-model="form.name" placeholder="如：ACSAMI 全文截稿" autofocus />
      </label>
      <label class="field">
        <span>类别</span>
        <select v-model="form.category">
          <option v-for="c in DDL_CATEGORIES" :key="c">{{ c }}</option>
        </select>
      </label>
      <label class="field">
        <span>截止日期</span>
        <input v-model="form.date" type="date" required />
      </label>
      <label class="field">
        <span>时刻（可选）</span>
        <input v-model="form.time" type="time" />
      </label>
      <label class="field">
        <span>关联课题（可选）</span>
        <select v-model="form.projectId">
          <option :value="null">不关联</option>
          <option v-for="p in projects.projects" :key="p.id" :value="p.id">{{ p.name }}</option>
        </select>
      </label>
      <label class="field">
        <span>备注（可选）</span>
        <input v-model="form.note" placeholder="投稿系统链接等" />
      </label>
    </div>
    <label class="field check">
      <input v-model="form.repeatYearly" type="checkbox" />
      年度事项（办结时可一键复制到明年）
    </label>
    <p v-if="error" class="error">{{ error }}</p>
    <div class="actions">
      <button class="btn primary" type="submit">{{ ddl ? '保存' : '创建' }}</button>
      <button class="btn" type="button" @click="emit('cancel')">取消</button>
    </div>
  </form>
</template>

<style scoped>
.form {
  border: 1px solid var(--line);
  border-radius: var(--radius);
  background: var(--card);
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

@media (max-width: 700px) {
  .grid {
    grid-template-columns: 1fr;
  }

  .grid .span2 {
    grid-column: span 1;
  }
}

.actions {
  display: flex;
  gap: 8px;
}
</style>
