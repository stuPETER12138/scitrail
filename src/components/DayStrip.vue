<script setup lang="ts">
import { computed, ref } from 'vue'
import { KIND_META } from '../stores/trajectory'

export interface StripTick {
  at: string
  kind: string
  title: string
}

const props = defineProps<{ ticks: StripTick[]; active: boolean }>()
const emit = defineEmits<{ seek: [at: string]; filter: [range: { from: string; to: string } | null] }>()

const HOURS = [0, 3, 6, 9, 12, 15, 18, 21, 24]

function minutesOf(at: string): number {
  const [h, m] = at.split(':').map(Number)
  return h * 60 + m
}

function toHHMM(min: number): string {
  const m = Math.max(0, Math.min(1440, Math.round(min)))
  return `${String(Math.floor(m / 60)).padStart(2, '0')}:${String(m % 60).padStart(2, '0')}`
}

const positioned = computed(() =>
  props.ticks
    .filter((t) => t.at)
    .map((t) => ({
      ...t,
      left: (minutesOf(t.at) / 1440) * 100,
      color: KIND_META[t.kind]?.color ?? '#8a94a0',
    })),
)

// 拖拽框选 → 过滤账本
const trackEl = ref<HTMLElement>()
const drag = ref<{ x1: number; x2: number } | null>(null)

function pctOf(clientX: number): number {
  const el = trackEl.value
  if (!el) return 0
  const rect = el.getBoundingClientRect()
  return Math.max(0, Math.min(1, (clientX - rect.left) / rect.width))
}

function onDown(e: PointerEvent) {
  drag.value = { x1: e.clientX, x2: e.clientX }
}

function onMove(e: PointerEvent) {
  if (drag.value) drag.value.x2 = e.clientX
}

function onUp() {
  const d = drag.value
  drag.value = null
  if (!d) return
  const moved = Math.abs(d.x2 - d.x1)
  if (moved < 6) {
    // 视为点击空白：清除过滤
    emit('filter', null)
    return
  }
  const p1 = pctOf(Math.min(d.x1, d.x2))
  const p2 = pctOf(Math.max(d.x1, d.x2))
  if ((p2 - p1) * 1440 < 10) return // 不足 10 分钟忽略
  emit('filter', { from: toHHMM(p1 * 1440), to: toHHMM(p2 * 1440) })
}

const dragRange = computed(() => {
  const d = drag.value
  if (!d || Math.abs(d.x2 - d.x1) < 6) return null
  return { left: pctOf(Math.min(d.x1, d.x2)) * 100, width: pctOf(Math.max(d.x1, d.x2)) * 100 }
})

const dragRangeStyle = computed(() =>
  dragRange.value
    ? { left: `${dragRange.value.left}%`, width: `${dragRange.value.width - dragRange.value.left}%` }
    : undefined,
)
</script>

<template>
  <div class="strip" :class="{ active }">
    <div class="label">当日时序</div>
    <div
      ref="trackEl"
      class="track"
      @pointerdown="onDown"
      @pointermove="onMove"
      @pointerup="onUp"
      @pointercancel="onUp"
    >
      <div class="line" />
      <span v-for="h in HOURS" :key="h" class="hour" :style="{ left: `${(h / 24) * 100}%` }">
        {{ h }}
      </span>
      <div v-if="dragRangeStyle" class="drag-sel" :style="dragRangeStyle" />
      <button
        v-for="(t, i) in positioned"
        :key="i"
        class="tick"
        :style="{ left: `${t.left}%`, background: t.color }"
        :title="`${t.at} ${t.title}`"
        type="button"
        @click.stop="emit('seek', t.at)"
      />
    </div>
    <span class="hint">拖拽框选过滤 · 点刻度定位</span>
  </div>
</template>

<style scoped>
.strip {
  display: grid;
  grid-template-columns: 64px 1fr auto;
  gap: 12px;
  align-items: center;
  border: 1px solid var(--line);
  border-radius: var(--radius);
  padding: 8px 14px 16px;
  margin-top: 12px;
  user-select: none;
}

.label {
  font-size: var(--text-xs);
  color: var(--muted-fg);
}

.hint {
  font-size: var(--text-2xs);
  color: var(--muted-fg);
  opacity: 0.8;
}

.track {
  position: relative;
  height: 26px;
  cursor: ew-resize;
  touch-action: none;
}

.line {
  position: absolute;
  left: 0;
  right: 0;
  top: 12px;
  height: 1px;
  background: var(--line);
}

.hour {
  position: absolute;
  top: 16px;
  transform: translateX(-50%);
  font-size: var(--text-2xs);
  color: var(--muted-fg);
}

.hour:first-child {
  transform: none;
}

.hour:last-child {
  transform: translateX(-100%);
}

.drag-sel {
  position: absolute;
  top: 2px;
  bottom: 2px;
  background: color-mix(in srgb, var(--accent) 16%, transparent);
  border-left: 1px dashed #4a7fd6;
  border-right: 1px dashed #4a7fd6;
  border-radius: 4px;
}

.tick {
  position: absolute;
  top: 12px;
  transform: translate(-50%, -50%);
  width: 14px;
  height: 14px;
  border-radius: 50%;
  border: 2px solid var(--card);
  padding: 0;
  cursor: pointer;
}
</style>
