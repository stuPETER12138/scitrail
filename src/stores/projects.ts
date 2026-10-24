import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  createProject,
  deleteProject,
  listProjects,
  updateProject,
  type Project,
  type ProjectInput,
} from '../api/commands'

export const PROJECT_COLORS = [
  '#e0533d',
  '#3b7dd8',
  '#2f9e68',
  '#8b5cf6',
  '#d97706',
  '#0ea5a4',
  '#c2410c',
  '#64748b',
]
export const PROJECT_TYPES = ['学位课题', '导师纵向', '横向', '自选', '申报']
export const PROJECT_STATUSES = ['进行中', '暂停', '已结题', '归档']

export const useProjectsStore = defineStore('projects', () => {
  const projects = ref<Project[]>([])
  const loaded = ref(false)

  async function load(force = false) {
    if (loaded.value && !force) return
    projects.value = await listProjects()
    loaded.value = true
  }

  async function create(input: ProjectInput) {
    await createProject(input)
    await load(true)
  }

  async function update(id: number, input: ProjectInput) {
    await updateProject(id, input)
    await load(true)
  }

  async function remove(id: number) {
    await deleteProject(id)
    await load(true)
  }

  function byId(id: number | null | undefined) {
    if (id == null) return undefined
    return projects.value.find((p) => p.id === id)
  }

  return { projects, loaded, load, create, update, remove, byId }
})
