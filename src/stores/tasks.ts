import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  completeTask,
  createTask,
  deleteTask,
  listTasks,
  statsWeek,
  updateTask,
  type Task,
  type TaskInput,
  type TaskPatch,
  type WeekStats,
} from '../api/commands'

export const useTasksStore = defineStore('tasks', () => {
  const today = ref<Task[]>([])
  const inbox = ref<Task[]>([])
  const active = ref<Task[]>([])
  const done = ref<Task[]>([])
  const stats = ref<WeekStats | null>(null)

  async function loadToday() {
    today.value = await listTasks('today')
  }
  async function loadInbox() {
    inbox.value = await listTasks('inbox')
  }
  async function loadActive() {
    active.value = await listTasks('active')
  }
  async function loadDone() {
    done.value = await listTasks('done')
  }
  async function loadStats() {
    stats.value = await statsWeek()
  }

  async function refresh() {
    await Promise.all([loadToday(), loadInbox(), loadActive(), loadDone(), loadStats()])
  }

  async function create(input: TaskInput) {
    const id = await createTask(input)
    await refresh()
    return id
  }

  async function update(id: number, patch: TaskPatch) {
    await updateTask(id, patch)
    await refresh()
  }

  async function complete(id: number) {
    const newId = await completeTask(id)
    await refresh()
    return newId
  }

  async function remove(id: number) {
    await deleteTask(id)
    await refresh()
  }

  return {
    today,
    inbox,
    active,
    done,
    stats,
    loadToday,
    loadInbox,
    loadActive,
    loadDone,
    loadStats,
    refresh,
    create,
    update,
    complete,
    remove,
  }
})
