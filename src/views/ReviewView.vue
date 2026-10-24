<script setup lang="ts">
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import MarkdownIt from 'markdown-it'
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import HeatmapCalendar from '../components/HeatmapCalendar.vue'
import {
  exportFile,
  getReviewStats,
  semesterSummary,
  type ReviewStats,
  type SemesterSummary,
} from '../api/commands'
import { addDays, todayStr } from '../utils/date'

const router = useRouter()
const md = new MarkdownIt()

const stats = ref<ReviewStats | null>(null)
const loadError = ref('')

onMounted(async () => {
  try {
    stats.value = await getReviewStats(addDays(todayStr(), -364), todayStr())
  } catch (e) {
    loadError.value = String(e)
  }
})

// ---- 趋势（近 N 天，热力图同一份数据切片） ----
const RANGE = [7, 30, 90] as const
const range = ref<7 | 30 | 90>(30)

const trend = computed(() => {
  const daily = stats.value?.daily ?? []
  return daily.slice(-range.value)
})

const TREND_W = 640
const TREND_H = 150

const trendMax = computed(() => {
  let max = 3
  for (const d of trend.value) {
    max = Math.max(max, d.tasksDone, d.experiment, d.literature)
  }
  return max
})

function points(key: 'tasksDone' | 'experiment' | 'literature'): string {
  const n = trend.value.length
  if (n < 2) return ''
  return trend.value
    .map((d, i) => {
      const x = 40 + (i * (TREND_W - 60)) / (n - 1)
      const y = 130 - (d[key] / trendMax.value) * 115
      return `${x.toFixed(1)},${y.toFixed(1)}`
    })
    .join(' ')
}

const SERIES: { key: 'tasksDone' | 'experiment' | 'literature'; label: string; color: string }[] = [
  { key: 'tasksDone', label: '✅ 任务完成', color: '#2f9e68' },
  { key: 'experiment', label: '🧪 实验', color: '#8b5cf6' },
  { key: 'literature', label: '📖 文献', color: '#3b7dd8' },
]

const trendEnds = computed(() => {
  const t = trend.value
  return { first: t[0]?.date.slice(5) ?? '', last: t[t.length - 1]?.date.slice(5) ?? '' }
})

// ---- 学期总结 ----
const semester = ref<SemesterSummary | null>(null)
const semesterBusy = ref(false)
const message = ref('')

async function makeSemester() {
  if (semesterBusy.value) return
  semesterBusy.value = true
  try {
    semester.value = await semesterSummary()
  } catch (e) {
    message.value = String(e)
  } finally {
    semesterBusy.value = false
  }
}

async function copySemester() {
  if (!semester.value) return
  await writeText(semester.value.content)
  message.value = '已复制到剪贴板'
}

async function exportSemester() {
  if (!semester.value) return
  try {
    const path = await exportFile(semester.value.filename, semester.value.content)
    message.value = `已导出：${path}`
  } catch (e) {
    message.value = String(e)
  }
}

const semesterHtml = computed(() =>
  semester.value ? md.render(semester.value.content) : '',
)

function pickDay(date: string) {
  void router.push({ path: '/trajectory', query: { date } })
}
</script>

<template>
  <div class="page">
    <h2>回顾</h2>
    <p v-if="loadError" class="error">{{ loadError }}</p>

    <template v-if="stats">
      <!-- KPI -->
      <div class="kpi">
        <div class="cell"><div class="v">{{ stats.kpis.totalEntries }}</div><div class="k">累计轨迹条目</div></div>
        <div class="cell"><div class="v">{{ stats.kpis.totalTasksDone }}</div><div class="k">完成任务</div></div>
        <div class="cell"><div class="v">{{ stats.kpis.totalExperiments }}</div><div class="k">实验次数</div></div>
        <div class="cell">
          <div class="v">{{ stats.kpis.currentStreak }}<small> 天</small></div>
          <div class="k">当前连续记录</div>
        </div>
        <div class="cell">
          <div class="v">{{ stats.kpis.longestStreak }}<small> 天</small></div>
          <div class="k">最长连续记录</div>
        </div>
      </div>

      <!-- 热力图 -->
      <section>
        <h3>活动热力图 · 近一年</h3>
        <HeatmapCalendar :days="stats.daily" @pick="pickDay" />
      </section>

      <!-- 趋势 -->
      <section>
        <h3 class="row-h">
          每日趋势 · 按类型
          <span class="range">
            <button
              v-for="r in RANGE"
              :key="r"
              type="button"
              class="r-btn"
              :class="{ on: range === r }"
              @click="range = r"
            >
              近{{ r }}日
            </button>
          </span>
        </h3>
        <div class="legend">
          <span v-for="s in SERIES" :key="s.key">
            <span class="sw" :style="{ background: s.color }" />{{ s.label }}
          </span>
        </div>
        <svg :viewBox="`0 0 ${TREND_W} ${TREND_H}`" class="trend" role="img" aria-label="每日记录趋势">
          <g stroke="var(--line)" stroke-width="1">
            <line x1="36" y1="130" x2="620" y2="130" />
            <line x1="36" y1="92" x2="620" y2="92" />
            <line x1="36" y1="54" x2="620" y2="54" />
            <line x1="36" y1="16" x2="620" y2="16" />
          </g>
          <g fill="var(--muted-fg)" font-size="10">
            <text x="30" y="133" text-anchor="end">0</text>
            <text x="30" y="58" text-anchor="end">{{ trendMax }}</text>
            <text x="40" y="146">{{ trendEnds.first }}</text>
            <text x="620" y="146" text-anchor="end">{{ trendEnds.last }}</text>
          </g>
          <polyline
            v-for="s in SERIES"
            :key="s.key"
            fill="none"
            :stroke="s.color"
            stroke-width="1.8"
            :points="points(s.key)"
          />
        </svg>
      </section>

      <div class="two-col">
        <!-- 课题分布 -->
        <section>
          <h3>课题投入分布 · 近 90 天</h3>
          <div v-if="stats.distribution.length" class="dist">
            <div v-for="d in stats.distribution" :key="d.projectId ?? 'none'" class="d-row">
              <span class="d-dot" :style="{ background: d.color ?? '#8a94a0' }" />
              <span class="d-name">{{ d.name }}</span>
              <span class="d-bar"><i :style="{ width: `${(d.count / stats.distribution[0].count) * 100}%`, background: d.color ?? '#8a94a0' }" /></span>
              <span class="d-count">{{ d.count }}</span>
            </div>
          </div>
          <p v-else class="dim">近 90 天暂无记录</p>
        </section>

        <!-- 学期总结 -->
        <section>
          <h3>学期总结</h3>
          <p class="dim small">按学期（3 月 / 9 月起点）汇总任务、条目与连续性，生成 Markdown 回顾。</p>
          <button class="btn primary" type="button" :disabled="semesterBusy" @click="makeSemester">
            {{ semesterBusy ? '生成中…' : '生成本学期总结' }}
          </button>
          <p v-if="message" class="msg">{{ message }}</p>
        </section>
      </div>
    </template>
    <p v-else class="dim">统计加载中…</p>

    <!-- 学期总结弹层 -->
    <div v-if="semester" class="modal-mask" @click.self="semester = null">
      <div class="modal">
        <div class="m-head">
          <b>{{ semester.filename }}</b>
          <div class="m-acts">
            <button class="btn" type="button" @click="copySemester">复制</button>
            <button class="btn" type="button" @click="exportSemester">导出 .md</button>
            <button class="btn" type="button" @click="semester = null">关闭</button>
          </div>
        </div>
        <div class="m-body md" v-html="semesterHtml" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.kpi {
  display: flex;
  border: 1px solid var(--line);
  border-radius: var(--radius);
  margin: 14px 0 0;
  overflow: hidden;
  flex-wrap: wrap;
  background: var(--card);
}

.kpi .cell {
  flex: 1;
  min-width: 110px;
  padding: 12px 16px;
  border-right: 1px solid var(--line);
}

.kpi .cell:last-child {
  border-right: none;
}

.kpi .v {
  font-size: var(--text-xl);
  font-weight: 500;
  font-variant-numeric: tabular-nums;
}

.kpi small {
  font-size: var(--text-sm);
  color: var(--muted-fg);
  font-weight: 400;
}

.kpi .k {
  font-size: var(--text-xs);
  color: var(--muted-fg);
  margin-top: 2px;
}

section {
  margin-top: 20px;
}

h3 {
  font-size: var(--text-md);
  font-weight: 500;
  color: var(--muted-fg);
  margin: 0 0 8px;
}

.row-h {
  display: flex;
  align-items: center;
  gap: 12px;
}

.range {
  display: flex;
  gap: 4px;
}

.r-btn {
  border: 1px solid var(--line);
  background: var(--card);
  border-radius: 6px;
  padding: 1px 9px;
  font-size: var(--text-xs);
  color: var(--muted-fg);
  cursor: pointer;
}

.r-btn.on {
  background: var(--hover);
  color: var(--fg);
  font-weight: 500;
}

.legend {
  display: flex;
  gap: 14px;
  font-size: var(--text-xs);
  color: var(--muted-fg);
  margin-bottom: 4px;
}

.legend span {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.sw {
  width: 9px;
  height: 9px;
  border-radius: 2px;
  display: inline-block;
}

.trend {
  width: 100%;
  margin-top: 4px;
}

.two-col {
  display: grid;
  grid-template-columns: 1.4fr 1fr;
  gap: 20px;
}

@media (max-width: 800px) {
  .two-col {
    grid-template-columns: 1fr;
  }
}

.dist {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.d-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: var(--text-sm);
}

.d-dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  flex: none;
}

.d-name {
  width: 90px;
  flex: none;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.d-bar {
  flex: 1;
  height: 7px;
  border-radius: 4px;
  background: var(--hover);
  overflow: hidden;
}

.d-bar i {
  display: block;
  height: 100%;
  border-radius: 4px;
}

.d-count {
  width: 30px;
  text-align: right;
  color: var(--muted-fg);
  font-variant-numeric: tabular-nums;
}

.dim {
  color: var(--muted-fg);
  font-size: var(--text-md);
}

.small {
  font-size: var(--text-sm);
}

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

.msg {
  font-size: var(--text-sm);
  color: var(--ok);
  margin: 8px 0 0;
  word-break: break-all;
}

.error {
  color: var(--danger);
}

/* 弹层 */
.modal-mask {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 60;
  padding: 24px;
}

.modal {
  background: var(--card);
  border-radius: 12px;
  width: min(720px, 100%);
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border: 1px solid var(--line);
}

.m-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--line);
  font-size: var(--text-md);
}

.m-acts {
  display: flex;
  gap: 6px;
}

.m-body {
  padding: 16px 20px;
  overflow-y: auto;
  line-height: 1.8;
}

:deep(.md h1) {
  font-size: var(--text-lg);
  margin: 0 0 10px;
}

:deep(.md h2) {
  font-size: var(--text-base);
  margin: 16px 0 6px;
}

:deep(.md li) {
  font-size: var(--text-md);
  margin: 3px 0;
}
</style>
