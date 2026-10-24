import { emit, listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'

// ---------- 类型 ----------

export interface Project {
  id: number
  name: string
  type: string
  role: string | null
  status: string
  color: string | null
  startDate: string | null
  endDate: string | null
  note: string | null
}

export interface ProjectInput {
  name: string
  type?: string
  role?: string | null
  status?: string
  color?: string | null
  startDate?: string | null
  endDate?: string | null
  note?: string | null
}

export interface Task {
  id: number
  title: string
  note: string | null
  projectId: number | null
  projectName: string | null
  milestoneId: number | null
  priority: number
  dueDate: string | null
  status: string
  repeatRule: string | null
  completedAt: string | null
  createdAt: string | null
}

export interface TaskInput {
  title: string
  note?: string | null
  projectId?: number | null
  priority?: number
  dueDate?: string | null
  repeatRule?: string | null
}

export interface TaskPatch extends TaskInput {
  status?: string
}

export interface EntryInput {
  kind: string
  title: string
  date?: string | null
  time?: string | null
  content?: string | null
  projectId?: number | null
  tags?: string[]
}

export interface ParsedDraft {
  kind: string
  title: string
  projectName: string | null
  projectMatched: number | null
  tags: string[]
  priority: number | null
  date: string | null
  time: string | null
}

export interface WeekStats {
  tasksCompleted: number
  tasksCreated: number
  entriesCreated: number
  weekStart: string
}

export interface AppInfo {
  version: string
  dbPath: string
  schemaVersion: number
}

// ---------- 认证 ----------

export const hasPassword = () => invoke<boolean>('has_password')
export const unlock = (password: string) => invoke<boolean>('unlock', { password })
export const lock = () => invoke<void>('lock')

// ---------- 元信息 ----------

export const appInfo = () => invoke<AppInfo>('app_info')

// ---------- 课题 ----------

export const listProjects = () => invoke<Project[]>('list_projects')
export const createProject = (input: ProjectInput) => invoke<number>('create_project', { input })
export const updateProject = (id: number, input: ProjectInput) =>
  invoke<void>('update_project', { id, input })
export const deleteProject = (id: number) => invoke<void>('delete_project', { id })

// ---------- 任务 ----------

export const listTasks = (view: string, projectId?: number | null) =>
  invoke<Task[]>('list_tasks', { view, projectId })
export const createTask = (task: TaskInput) => invoke<number>('create_task', { task })
export const updateTask = (id: number, task: TaskPatch) => invoke<void>('update_task', { id, task })
export const completeTask = (id: number) => invoke<number | null>('complete_task', { id })
export const deleteTask = (id: number) => invoke<void>('delete_task', { id })

// ---------- 速录与条目 ----------

export const parseQuick = (text: string, defaultKind?: string) =>
  invoke<ParsedDraft>('parse_quick', { text, defaultKind })
export const createEntry = (entry: EntryInput) => invoke<number>('create_entry', { entry })

// ---------- 统计 ----------

export const statsWeek = () => invoke<WeekStats>('stats_week')

// ---------- 跨窗口刷新事件 ----------

export const emitChanged = () => emit('scitrail://changed')
export const onChanged = (cb: () => void) => listen('scitrail://changed', () => cb())
