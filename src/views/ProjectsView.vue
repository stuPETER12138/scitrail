<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { PROJECT_COLORS, PROJECT_STATUSES, PROJECT_TYPES, useProjectsStore } from '../stores/projects'

const projects = useProjectsStore()
const showCreate = ref(false)
const error = ref('')

const form = ref({
  name: '',
  type: '学位课题',
  color: PROJECT_COLORS[0],
  role: '',
})

onMounted(() => void projects.load(true))

async function submitCreate() {
  if (!form.value.name.trim()) {
    error.value = '课题名称不能为空'
    return
  }
  try {
    await projects.create({
      name: form.value.name.trim(),
      type: form.value.type,
      color: form.value.color,
      role: form.value.role.trim() || null,
    })
    showCreate.value = false
    form.value = { name: '', type: '学位课题', color: PROJECT_COLORS[0], role: '' }
    error.value = ''
  } catch (e) {
    error.value = String(e)
  }
}

async function changeStatus(id: number, name: string, status: string, type: string, color: string | null, role: string | null) {
  await projects.update(id, { name, type, status, color, role })
}

async function removeProject(id: number, name: string) {
  if (!window.confirm(`删除课题「${name}」？其下任务将移回收件箱。`)) return
  await projects.remove(id)
}
</script>

<template>
  <div class="page">
    <h2>课题</h2>

    <button v-if="!showCreate" class="btn" type="button" @click="showCreate = true">＋ 新建课题</button>

    <form v-else class="create" @submit.prevent="submitCreate">
      <input v-model="form.name" placeholder="课题名称（必填）" autofocus class="name" />
      <div class="row">
        <label>
          类型
          <select v-model="form.type">
            <option v-for="t in PROJECT_TYPES" :key="t">{{ t }}</option>
          </select>
        </label>
        <label>
          我的角色
          <input v-model="form.role" placeholder="负责人 / 参与（可空）" />
        </label>
      </div>
      <div class="row colors">
        <button
          v-for="c in PROJECT_COLORS"
          :key="c"
          type="button"
          class="swatch"
          :class="{ on: form.color === c }"
          :style="{ background: c }"
          :aria-label="`颜色 ${c}`"
          @click="form.color = c"
        />
      </div>
      <p v-if="error" class="error">{{ error }}</p>
      <div class="row">
        <button class="btn primary" type="submit">创建</button>
        <button class="btn" type="button" @click="showCreate = false">取消</button>
      </div>
    </form>

    <ul v-if="projects.projects.length" class="list">
      <li v-for="p in projects.projects" :key="p.id" :class="{ archived: p.status === '归档' }">
        <span class="dot" :style="{ background: p.color ?? '#8a94a0' }" />
        <div class="body">
          <div class="name">{{ p.name }}</div>
          <div class="meta">{{ p.type }}{{ p.role ? ` · ${p.role}` : '' }}</div>
        </div>
        <select
          class="status"
          :value="p.status"
          @change="changeStatus(p.id, p.name, ($event.target as HTMLSelectElement).value, p.type, p.color, p.role)"
        >
          <option v-for="s in PROJECT_STATUSES" :key="s">{{ s }}</option>
        </select>
        <button class="del" type="button" @click="removeProject(p.id, p.name)">删除</button>
      </li>
    </ul>
    <p v-else class="placeholder">暂无课题 —— 建议先建一个「学位课题」</p>
  </div>
</template>

<style scoped>
.btn {
  border: 1px solid var(--line);
  background: var(--card);
  border-radius: 8px;
  padding: 7px 14px;
  cursor: pointer;
}

.btn.primary {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--accent-fg);
}

.create {
  margin-top: 14px;
  border: 1px solid var(--line);
  border-radius: var(--radius);
  padding: 14px;
  background: var(--card);
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-width: 460px;
}

.name {
  padding: 8px 10px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: var(--bg);
  outline: none;
}

.name:focus {
  border-color: var(--accent);
}

.row {
  display: flex;
  gap: 12px;
  align-items: center;
}

.row label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
  color: var(--muted-fg);
  flex: 1;
}

.row select,
.row input {
  padding: 7px 9px;
  border: 1px solid var(--line);
  border-radius: 7px;
  background: var(--bg);
  color: var(--fg);
  outline: none;
}

.swatch {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  padding: 0;
}

.swatch.on {
  border-color: var(--fg);
}

.error {
  color: var(--danger);
  font-size: 12px;
  margin: 0;
}

.list {
  list-style: none;
  margin: 16px 0 0;
  padding: 0;
}

.list li {
  display: flex;
  gap: 10px;
  align-items: center;
  padding: 10px 4px;
  border-bottom: 1px solid var(--line);
}

.list li.archived {
  opacity: 0.55;
}

.dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex: none;
}

.body {
  flex: 1;
  min-width: 0;
}

.name {
  font-weight: 500;
}

.meta {
  color: var(--muted-fg);
  font-size: 12px;
}

.status {
  border: 1px solid var(--line);
  border-radius: 7px;
  background: var(--bg);
  color: var(--fg);
  padding: 4px 6px;
  font-size: 12px;
}

.del {
  border: none;
  background: none;
  color: var(--muted-fg);
  cursor: pointer;
  font-size: 12px;
}

.del:hover {
  color: var(--danger);
}
</style>
