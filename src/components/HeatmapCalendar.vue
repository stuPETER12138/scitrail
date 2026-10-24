<script setup lang="ts">
import { computed } from 'vue'
import type { DayCount } from '../api/commands'

const props = defineProps<{ days: DayCount[] }>()
defineEmits<{ pick: [date: string] }>()

interface Cell {
  date: string
  total: number
  level: number
  label: string
}

function totalOf(d: DayCount): number {
  return d.experiment + d.literature + d.expense + d.meeting + d.note + d.tasksDone
}

function levelOf(n: number): number {
  if (n <= 0) return 0
  if (n === 1) return 1
  if (n <= 3) return 2
  if (n <= 6) return 3
  return 4
}

const weeks = computed(() => {
  if (!props.days.length) return [] as { label: string; cells: (Cell | null)[] }[]
  const first = props.days[0].date
  const [y, m, d] = first.split('-').map(Number)
  const offset = (new Date(y, m - 1, d).getDay() + 6) % 7 // 周一=0
  const cells: (Cell | null)[] = [...Array(offset).fill(null)]
  for (const day of props.days) {
    const total = totalOf(day)
    cells.push({
      date: day.date,
      total,
      level: levelOf(total),
      label: `${day.date} · ${total} 条（🧪${day.experiment} 📖${day.literature} ✅${day.tasksDone}）`,
    })
  }
  // 补齐最后一列
  while (cells.length % 7 !== 0) cells.push(null)

  const out: { label: string; cells: (Cell | null)[] }[] = []
  let lastMonth = -1
  for (let i = 0; i < cells.length; i += 7) {
    const col = cells.slice(i, i + 7)
    const head = col.find((c): c is Cell => c !== null)
    const mon = head ? Number(head.date.slice(5, 7)) : lastMonth
    out.push({ label: mon !== lastMonth ? `${mon}月` : '', cells: col })
    lastMonth = mon
  }
  return out
})

const WEEKDAYS = ['一', '', '三', '', '五', '', '']
</script>

<template>
  <div class="hm">
    <div class="hm-scroll">
      <div class="hm-grid">
        <div class="hm-months">
          <span v-for="(w, i) in weeks" :key="i" class="hm-month">{{ w.label }}</span>
        </div>
        <div class="hm-body">
          <div class="hm-weekdays">
            <span v-for="(w, i) in WEEKDAYS" :key="i">{{ w }}</span>
          </div>
          <div class="hm-cols">
            <div v-for="(w, i) in weeks" :key="i" class="hm-col">
              <button
                v-for="(c, j) in w.cells"
                :key="j"
                type="button"
                class="hm-cell"
                :class="`l${c?.level ?? 0}`"
                :title="c?.label ?? ''"
                :disabled="!c"
                @click="c && $emit('pick', c.date)"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
    <div class="hm-legend">
      少
      <span class="hm-cell l0" /><span class="hm-cell l1" /><span class="hm-cell l2" /><span
        class="hm-cell l3"
      /><span class="hm-cell l4" />
      多 · 点击某天 → 轨迹页定位该日
    </div>
  </div>
</template>

<style scoped>
.hm {
  margin-top: 8px;
}

.hm-scroll {
  overflow-x: auto;
  padding-bottom: 4px;
}

.hm-grid {
  display: inline-flex;
  flex-direction: column;
  gap: 4px;
  min-width: 100%;
}

.hm-months {
  display: flex;
  gap: 3px;
  padding-left: 20px;
  font-size: var(--text-2xs);
  color: var(--muted-fg);
  height: 12px;
}

.hm-month {
  width: 11px;
  flex: none;
  white-space: nowrap;
}

.hm-body {
  display: flex;
  gap: 4px;
}

.hm-weekdays {
  display: flex;
  flex-direction: column;
  gap: 3px;
  font-size: var(--text-2xs);
  color: var(--muted-fg);
  width: 16px;
}

.hm-weekdays span {
  height: 11px;
  line-height: 11px;
}

.hm-cols {
  display: flex;
  gap: 3px;
}

.hm-col {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.hm-cell {
  width: 11px;
  height: 11px;
  border-radius: 2.5px;
  padding: 0;
  border: none;
  cursor: pointer;
  background: var(--hover);
}

.hm-cell.l1 {
  background: color-mix(in srgb, var(--accent) 28%, var(--bg));
}

.hm-cell.l2 {
  background: color-mix(in srgb, var(--accent) 52%, var(--bg));
}

.hm-cell.l3 {
  background: color-mix(in srgb, var(--accent) 76%, var(--bg));
}

.hm-cell.l4 {
  background: var(--accent);
}

.hm-cell:disabled {
  cursor: default;
  background: transparent;
}

.hm-cell:not(:disabled):hover {
  outline: 1.5px solid var(--fg);
}

.hm-legend {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-top: 8px;
  font-size: var(--text-xs);
  color: var(--muted-fg);
}

.hm-legend .hm-cell {
  cursor: default;
}
</style>
