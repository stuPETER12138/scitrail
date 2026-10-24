<script setup lang="ts">
import { computed } from 'vue'
import type { Task } from '../api/commands'
import { useProjectsStore } from '../stores/projects'
import { dueInfo } from '../utils/date'

const props = defineProps<{ task: Task }>()
defineEmits<{ complete: [id: number]; edit: [task: Task] }>()

const projects = useProjectsStore()
const done = computed(() => props.task.status === '完成')
const project = computed(() => projects.byId(props.task.projectId))
const due = computed(() => (props.task.dueDate ? dueInfo(props.task.dueDate) : null))
</script>

<template>
  <div class="row" :class="{ done }" @click="$emit('edit', task)">
    <button
      class="check"
      :class="{ checked: done }"
      :disabled="done"
      :title="done ? '已完成' : '点击完成'"
      @click.stop="$emit('complete', task.id)"
    >
      <span v-if="done">✓</span>
    </button>
    <span class="title">{{ task.title }}</span>
    <span v-if="task.repeatRule" class="repeat" title="重复任务">🔁</span>
    <span v-if="task.priority === 0" class="pri high">!高</span>
    <span v-else-if="task.priority === 1" class="pri mid">!中</span>
    <span v-if="project" class="proj">
      <span class="dot" :style="{ background: project.color ?? '#8a94a0' }" />
      {{ project.name }}
    </span>
    <span v-if="due" class="due" :class="due.cls">{{ due.label }}</span>
  </div>
</template>

<style scoped>
.row {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 8px 6px;
  border-bottom: 1px solid var(--line);
  cursor: default;
}

.row:hover {
  background: var(--hover);
}

.check {
  width: 16px;
  height: 16px;
  flex: none;
  border: 1.5px solid var(--muted-fg);
  border-radius: 50%;
  background: transparent;
  color: #fff;
  font-size: 10px;
  line-height: 1;
  padding: 0;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.check:hover {
  border-color: var(--ok);
}

.check.checked {
  border-color: var(--ok);
  background: var(--ok);
}

.check:disabled {
  cursor: default;
}

.title {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.done .title {
  text-decoration: line-through;
  color: var(--muted-fg);
}

.repeat {
  font-size: 11px;
}

.pri {
  font-size: 10px;
  border-radius: 4px;
  padding: 0 5px;
  flex: none;
}

.pri.high {
  background: var(--danger);
  color: #fff;
}

.pri.mid {
  background: color-mix(in srgb, var(--warn) 18%, transparent);
  color: var(--warn);
}

.proj {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--muted-fg);
  flex: none;
}

.dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
}

.due {
  font-size: 11px;
  flex: none;
  color: var(--muted-fg);
}

.due.overdue {
  color: var(--danger);
  font-weight: 500;
}

.due.today {
  color: var(--warn);
  font-weight: 500;
}
</style>
