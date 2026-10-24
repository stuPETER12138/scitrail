import { defineStore } from 'pinia'
import { ref } from 'vue'
import { getTrajectory, type TrajectoryDay } from '../api/commands'
import { addDays, todayStr } from '../utils/date'

/** 类型元信息：轨迹账本全局共用 */
export const KIND_META: Record<string, { label: string; icon: string; color: string }> = {
  experiment: { label: '实验', icon: '🧪', color: '#8b5cf6' },
  literature: { label: '文献', icon: '📖', color: '#3b7dd8' },
  expense: { label: '报销', icon: '💰', color: '#d97706' },
  meeting: { label: '组会', icon: '📅', color: '#0ea5a4' },
  note: { label: '笔记', icon: '📝', color: '#8a94a0' },
  task: { label: '任务', icon: '✓', color: '#2f9e68' },
  ddl: { label: 'DDL', icon: '⏰', color: '#e5484d' },
}

const WINDOW = 14
const STEP = 7

export const useTrajectoryStore = defineStore('trajectory', () => {
  const days = ref<TrajectoryDay[]>([])
  const oldest = ref(todayStr())
  const noMore = ref(false)
  const loading = ref(false)

  async function load() {
    loading.value = true
    try {
      const to = todayStr()
      const from = addDays(to, -(WINDOW - 1))
      days.value = await getTrajectory(from, to)
      oldest.value = from
      noMore.value = false
    } finally {
      loading.value = false
    }
  }

  /// 向上翻更早的一页；返回是否加载到内容
  async function loadOlder(): Promise<boolean> {
    if (loading.value) return false
    loading.value = true
    try {
      const to = addDays(oldest.value, -1)
      const from = addDays(to, -(STEP - 1))
      const older = await getTrajectory(from, to)
      oldest.value = from
      if (!older.length) {
        noMore.value = true
        return false
      }
      days.value = [...older, ...days.value]
      return true
    } finally {
      loading.value = false
    }
  }

  return { days, oldest, noMore, loading, load, loadOlder }
})
