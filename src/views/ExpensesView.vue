<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import {
  expenseLedger,
  setExpenseStatus,
  type ExpenseLedger,
} from '../api/commands'

const ledger = ref<ExpenseLedger | null>(null)
const loadError = ref('')

async function reload() {
  try {
    ledger.value = await expenseLedger()
  } catch (e) {
    loadError.value = String(e)
  }
}

onMounted(() => void reload())

async function changeStatus(id: number, status: string) {
  await setExpenseStatus(id, status)
  await reload()
}

function yuan(n: number): string {
  return n.toLocaleString('zh-CN', { minimumFractionDigits: 0, maximumFractionDigits: 1 })
}

const maxProject = computed(() => ledger.value?.byProject[0]?.total ?? 1)
</script>

<template>
  <div class="page wide">
    <h2>报销台账</h2>
    <p class="hint">
      记账入口：Ctrl+Alt+K 速录 <code>钱 移相器 380 #催化</code> 或轨迹页「💰 报销」类型；
      金额缺省时自动从标题数字提取
    </p>
    <p v-if="loadError" class="error">{{ loadError }}</p>

    <template v-if="ledger">
      <!-- 汇总 -->
      <div class="cards">
        <div class="card adv" :class="{ zero: ledger.summary.advanced === 0 }">
          <div class="v">¥{{ yuan(ledger.summary.advanced) }}</div>
          <div class="k">垫付未回款 · {{ ledger.summary.advancedCount }} 笔</div>
        </div>
        <div class="card">
          <div class="v">¥{{ yuan(ledger.summary.submitted) }}</div>
          <div class="k">已提交待到账</div>
        </div>
        <div class="card">
          <div class="v">¥{{ yuan(ledger.summary.received) }}</div>
          <div class="k">已到账</div>
        </div>
        <div class="card">
          <div class="v">¥{{ yuan(ledger.summary.total) }}</div>
          <div class="k">累计经手</div>
        </div>
      </div>

      <!-- 按课题 -->
      <section v-if="ledger.byProject.length > 1 || (ledger.byProject[0] && ledger.byProject[0].projectId !== null)">
        <h3>按课题开销</h3>
        <div class="dist">
          <div v-for="p in ledger.byProject" :key="p.projectId ?? 'none'" class="d-row">
            <span class="d-dot" :style="{ background: p.color ?? '#8a94a0' }" />
            <span class="d-name">{{ p.name }}</span>
            <span class="d-bar">
              <i :style="{ width: `${(p.total / maxProject) * 100}%`, background: p.color ?? '#8a94a0' }" />
            </span>
            <span class="d-count">¥{{ yuan(p.total) }}</span>
          </div>
        </div>
      </section>

      <!-- 明细 -->
      <section>
        <h3>明细 · {{ ledger.items.length }} 笔</h3>
        <div v-if="ledger.items.length" class="tbl">
          <div class="row head-row">
            <span>日期</span><span class="t-title">事由</span><span class="amt">金额</span>
            <span>课题</span><span>状态</span>
          </div>
          <div v-for="it in ledger.items" :key="it.id" class="row">
            <span class="t-date">{{ it.date.slice(5) }}</span>
            <span class="t-title">
              {{ it.title }}
              <span v-if="it.reason" class="t-reason">{{ it.reason }}</span>
            </span>
            <span class="amt">¥{{ yuan(it.amount) }}</span>
            <span class="t-proj">{{ it.projectName ?? '—' }}</span>
            <select
              class="t-status"
              :class="it.status"
              :value="it.status"
              @change="changeStatus(it.id, ($event.target as HTMLSelectElement).value)"
            >
              <option>垫付中</option>
              <option>已提交</option>
              <option>已到账</option>
            </select>
          </div>
        </div>
        <p v-else class="empty">还没有报销记录 —— 速录一笔试试</p>
      </section>
    </template>
    <p v-else class="empty">加载中…</p>
  </div>
</template>

<style scoped>
.page.wide {
  max-width: 1080px;
  padding: 18px 24px;
}

.hint {
  color: var(--muted-fg);
  font-size: var(--text-sm);
  margin: 4px 0 0;
}

code {
  background: var(--hover);
  border-radius: 4px;
  padding: 0 5px;
}

.error {
  color: var(--danger);
}

.cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  gap: 10px;
  margin-top: 16px;
}

.card {
  border: 1px solid var(--line);
  border-radius: var(--radius);
  background: var(--card);
  padding: 14px 16px;
}

.card.adv {
  border-color: color-mix(in srgb, var(--warn) 55%, var(--line));
  background: color-mix(in srgb, var(--warn) 7%, var(--card));
}

.card.adv.zero {
  border-color: var(--line);
  background: var(--card);
}

.v {
  font-size: var(--text-xl);
  font-weight: 500;
  font-variant-numeric: tabular-nums;
}

.card.adv .v {
  color: var(--warn);
}

.card.adv.zero .v {
  color: var(--ok);
}

.k {
  font-size: var(--text-xs);
  color: var(--muted-fg);
  margin-top: 2px;
}

section {
  margin-top: 22px;
}

h3 {
  font-size: var(--text-md);
  font-weight: 500;
  color: var(--muted-fg);
  margin: 0 0 8px;
}

.dist {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-width: 560px;
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
  width: 76px;
  text-align: right;
  color: var(--muted-fg);
  font-variant-numeric: tabular-nums;
}

.tbl {
  border: 1px solid var(--line);
  border-radius: var(--radius);
  background: var(--card);
  overflow: hidden;
}

.row {
  display: grid;
  grid-template-columns: 54px 1fr 90px 110px 92px;
  gap: 8px;
  align-items: center;
  padding: 8px 14px;
  border-bottom: 1px solid var(--line);
  font-size: var(--text-md);
}

.row:last-child {
  border-bottom: none;
}

.head-row {
  color: var(--muted-fg);
  font-size: var(--text-xs);
  background: var(--hover);
}

.t-date {
  color: var(--muted-fg);
  font-variant-numeric: tabular-nums;
}

.t-title {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.t-reason {
  color: var(--muted-fg);
  font-size: var(--text-xs);
  margin-left: 6px;
}

.amt {
  text-align: right;
  font-variant-numeric: tabular-nums;
  font-weight: 500;
}

.t-proj {
  color: var(--muted-fg);
  font-size: var(--text-sm);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.t-status {
  border: 1px solid var(--line);
  border-radius: 7px;
  padding: 3px 6px;
  background: var(--bg);
  color: var(--fg);
  font-size: var(--text-sm);
  cursor: pointer;
}

.t-status.垫付中 {
  color: var(--warn);
}

.t-status.已到账 {
  color: var(--ok);
}

.empty {
  color: var(--muted-fg);
  font-size: var(--text-md);
  padding: 16px 0;
}
</style>
