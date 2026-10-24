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
  fields?: Record<string, unknown> | null
  refs?: RefItem[] | null
}

export interface RefItem {
  label: string
  path: string
}

export interface Entry {
  id: number
  date: string
  time: string | null
  kind: string
  title: string
  content: string
  projectId: number | null
  tags: string[]
  fields: Record<string, unknown> | null
  refs: RefItem[]
  createdAt: string | null
  updatedAt: string | null
}

export interface HistoryItem {
  snapshot: Record<string, unknown>
  savedAt: string
}

export interface TaskLite {
  id: number
  title: string
  projectId: number | null
  priority: number
}

export interface DdlLite {
  id: number
  name: string
  category: string | null
  dueDate: string
}

export interface TrajectoryItem {
  src: 'entry' | 'task_done' | 'ddl'
  at: string
  entry?: Entry
  task?: TaskLite
  ddl?: DdlLite
}

export interface TrajectoryDay {
  date: string
  items: TrajectoryItem[]
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

export interface Ddl {
  id: number
  name: string
  category: string | null
  dueDate: string
  leadDays: string
  projectId: number | null
  status: string
  repeatYearly: boolean
  note: string | null
}

export interface DdlInput {
  name: string
  category?: string | null
  dueDate: string
  leadDays?: string | null
  projectId?: number | null
  repeatYearly?: boolean
  note?: string | null
  status?: string
}

export interface Milestone {
  id: number
  name: string
  targetDate: string | null
  status: string
  doneAt: string | null
  note: string | null
}

export interface MilestoneInput {
  name: string
  targetDate?: string | null
  status?: string
  note?: string | null
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

export interface ListEntriesParams {
  from?: string
  to?: string
  kind?: string
  projectId?: number | null
  q?: string
}

export const listEntries = (p: ListEntriesParams = {}) =>
  invoke<Entry[]>('list_entries', { ...p })
export const updateEntry = (id: number, entry: EntryInput) =>
  invoke<void>('update_entry', { id, entry })
export const deleteEntry = (id: number) => invoke<void>('delete_entry', { id })
export const entryHistory = (id: number) => invoke<HistoryItem[]>('entry_history', { id })
export const todayEntryCount = () => invoke<number>('today_entry_count')

// ---------- 轨迹投影 ----------

export const getTrajectory = (from: string, to: string) =>
  invoke<TrajectoryDay[]>('get_trajectory', { from, to })

// ---------- 周报 ----------

export interface WeeklyReport {
  weekStart: string
  content: string
  generatedAt: string | null
  editedAt: string | null
}

export const generateWeekly = (weekStart?: string) =>
  invoke<string>('generate_weekly', { weekStart })
export const listWeeklies = () => invoke<WeeklyReport[]>('list_weeklies')
export const saveWeekly = (weekStart: string, content: string) =>
  invoke<void>('save_weekly', { weekStart, content })
export const exportWeekly = (weekStart: string) =>
  invoke<string>('export_weekly', { weekStart })

// ---------- 备份与导出 ----------

export interface BackupLog {
  id: number
  file: string
  sizeBytes: number | null
  status: string
  createdAt: string
}

export const runBackup = () => invoke<string>('run_backup')
export const listBackups = () => invoke<BackupLog[]>('list_backups')
export const restoreBackup = (file: string) => invoke<void>('restore_backup', { file })
export const exportData = (format: 'json' | 'md') => invoke<string>('export_data', { format })
export const changePassword = (oldPassword: string, newPassword: string) =>
  invoke<void>('change_password', { oldPassword, newPassword })

// ---------- 回顾统计（M2-1） ----------

export interface DayCount {
  date: string
  experiment: number
  literature: number
  expense: number
  meeting: number
  note: number
  tasksDone: number
}

export interface Kpis {
  totalEntries: number
  totalTasksDone: number
  totalExperiments: number
  currentStreak: number
  longestStreak: number
}

export interface ProjectDist {
  projectId: number | null
  name: string
  color: string | null
  count: number
}

export interface ReviewStats {
  from: string
  to: string
  kpis: Kpis
  daily: DayCount[]
  distribution: ProjectDist[]
}

export interface SemesterSummary {
  filename: string
  content: string
}

export const getReviewStats = (from: string, to: string) =>
  invoke<ReviewStats>('get_review_stats', { from, to })
export const semesterSummary = () => invoke<SemesterSummary>('semester_summary')
export const exportFile = (name: string, content: string) =>
  invoke<string>('export_file', { name, content })

// ---------- 论文投稿流水线（M2-2） ----------

export interface Paper {
  id: number
  title: string
  venue: string | null
  stage: string
  submitDate: string | null
  expectReply: string | null
  reviseDeadline: string | null
  projectId: number | null
  ddlId: number | null
  note: string | null
  createdAt: string | null
  updatedAt: string | null
}

export interface PaperInput {
  title: string
  venue?: string | null
  stage?: string
  submitDate?: string | null
  expectReply?: string | null
  reviseDeadline?: string | null
  projectId?: number | null
  note?: string | null
}

export const listPapers = () => invoke<Paper[]>('list_papers')
export const createPaper = (input: PaperInput) => invoke<number>('create_paper', { input })
export const updatePaper = (id: number, input: PaperInput) =>
  invoke<void>('update_paper', { id, input })
export const deletePaper = (id: number) => invoke<void>('delete_paper', { id })

// ---------- 统计 ----------

export const statsWeek = () => invoke<WeekStats>('stats_week')

// ---------- DDL 与里程碑 ----------

export const listDdls = (activeOnly = false) => invoke<Ddl[]>('list_ddls', { activeOnly })
export const createDdl = (input: DdlInput) => invoke<number>('create_ddl', { input })
export const updateDdl = (id: number, input: DdlInput) => invoke<void>('update_ddl', { id, input })
export const deleteDdl = (id: number) => invoke<void>('delete_ddl', { id })
export const rolloverDdl = (id: number) => invoke<number>('rollover_ddl', { id })

export const listMilestones = () => invoke<Milestone[]>('list_milestones')
export const createMilestone = (input: MilestoneInput) =>
  invoke<number>('create_milestone', { input })
export const updateMilestone = (id: number, input: MilestoneInput) =>
  invoke<void>('update_milestone', { id, input })
export const deleteMilestone = (id: number) => invoke<void>('delete_milestone', { id })

// ---------- 跨窗口刷新事件 ----------

export const emitChanged = () => emit('scitrail://changed')
export const onChanged = (cb: () => void) => listen('scitrail://changed', () => cb())
