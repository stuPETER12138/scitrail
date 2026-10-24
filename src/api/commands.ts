import { invoke } from '@tauri-apps/api/core'

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

export interface AppInfo {
  version: string
  dbPath: string
  schemaVersion: number
}

export const hasPassword = () => invoke<boolean>('has_password')
export const unlock = (password: string) => invoke<boolean>('unlock', { password })
export const lock = () => invoke<void>('lock')
export const listProjects = () => invoke<Project[]>('list_projects')
export const appInfo = () => invoke<AppInfo>('app_info')
