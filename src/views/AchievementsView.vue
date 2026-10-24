<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import {
  ACHIEVEMENT_KINDS,
  ACHIEVEMENT_STATUSES,
  achievementsMarkdown,
  createAchievement,
  createRequirement,
  deleteAchievement,
  deleteRequirement,
  exportFile,
  listAchievements,
  listPapers,
  listRequirements,
  updateAchievement,
  updateRequirement,
  type Achievement,
  type GradRequirement,
} from '../api/commands'
import { useProjectsStore } from '../stores/projects'

const projects = useProjectsStore()
const achievements = ref<Achievement[]>([])
const requirements = ref<GradRequirement[]>([])
const pendingPapers = ref(0)
const message = ref('')

async function reload() {
  ;[achievements.value, requirements.value] = await Promise.all([
    listAchievements(),
    listRequirements(),
  ])
  const papers = await listPapers()
  pendingPapers.value = papers.filter((p) =>
    ['投稿中', '审稿中', '返修'].includes(p.stage),
  ).length
}

onMounted(() => {
  void reload()
  void projects.load()
})

// ---- 对照（按类型自动匹配，说明书 §4.9） ----
function csv(s: string): string[] {
  return s
    .split(',')
    .map((x) => x.trim())
    .filter(Boolean)
}

interface Match {
  req: GradRequirement
  achieved: number
  gap: number
  includePending: boolean
}

const matches = computed<Match[]>(() =>
  requirements.value.map((req) => {
    const kinds = csv(req.matchKinds)
    const statuses = csv(req.matchStatus)
    const achieved = achievements.value.filter(
      (a) =>
        (kinds.length === 0 || kinds.includes(a.kind)) && statuses.includes(a.status),
    ).length
    return {
      req,
      achieved: Math.min(achieved, req.needCount),
      gap: Math.max(0, req.needCount - achieved),
      includePending: statuses.includes('在投'),
    }
  }),
)

const pendingTotal = computed(
  () =>
    pendingPapers.value + achievements.value.filter((a) => a.status === '在投').length,
)

// ---- 成果编辑抽屉 ----
const achDrawer = ref(false)
const editingAch = ref<Achievement | null>(null)
const achError = ref('')
const confirmAchDelete = ref(false)

const achForm = reactive({
  kind: '期刊论文',
  title: '',
  authorRank: '第一作者',
  status: '已发表',
  date: '',
  venue: '',
  doi: '',
  projectId: null as number | null,
  note: '',
})

watch(
  [achDrawer, editingAch],
  () => {
    const a = editingAch.value
    achForm.kind = a?.kind ?? '期刊论文'
    achForm.title = a?.title ?? ''
    achForm.authorRank = a?.authorRank ?? '第一作者'
    achForm.status = a?.status ?? '已发表'
    achForm.date = a?.date?.slice(0, 10) ?? ''
    achForm.venue = a?.venue ?? ''
    achForm.doi = a?.doi ?? ''
    achForm.projectId = a?.projectId ?? null
    achForm.note = a?.note ?? ''
    achError.value = ''
    confirmAchDelete.value = false
  },
  { immediate: true },
)

function openAchCreate() {
  editingAch.value = null
  achDrawer.value = true
}

function openAchEdit(a: Achievement) {
  editingAch.value = a
  achDrawer.value = true
}

async function saveAch() {
  if (!achForm.title.trim()) {
    achError.value = '标题不能为空'
    return
  }
  const input = {
    kind: achForm.kind,
    title: achForm.title.trim(),
    authorRank: achForm.authorRank.trim() || null,
    status: achForm.status,
    date: achForm.date || null,
    venue: achForm.venue.trim() || null,
    doi: achForm.doi.trim() || null,
    projectId: achForm.projectId,
    note: achForm.note.trim() || null,
  }
  try {
    if (editingAch.value) await updateAchievement(editingAch.value.id, input)
    else await createAchievement(input)
    achDrawer.value = false
    await reload()
  } catch (e) {
    achError.value = String(e)
  }
}

async function removeAch() {
  if (!editingAch.value) return
  if (!confirmAchDelete.value) {
    confirmAchDelete.value = true
    return
  }
  await deleteAchievement(editingAch.value.id)
  achDrawer.value = false
  await reload()
}

// ---- 毕业要求编辑抽屉 ----
const reqDrawer = ref(false)
const editingReq = ref<GradRequirement | null>(null)
const reqError = ref('')
const confirmReqDelete = ref(false)

const reqForm = reactive({
  item: '',
  needCount: 1,
  kinds: [] as string[],
  includePending: false,
  note: '',
})

watch(
  [reqDrawer, editingReq],
  () => {
    const r = editingReq.value
    reqForm.item = r?.item ?? ''
    reqForm.needCount = r?.needCount ?? 1
    reqForm.kinds = r ? csv(r.matchKinds) : []
    reqForm.includePending = r ? csv(r.matchStatus).includes('在投') : false
    reqForm.note = r?.note ?? ''
    reqError.value = ''
    confirmReqDelete.value = false
  },
  { immediate: true },
)

function toggleKind(k: string) {
  const i = reqForm.kinds.indexOf(k)
  if (i >= 0) reqForm.kinds.splice(i, 1)
  else reqForm.kinds.push(k)
}

async function saveReq() {
  if (!reqForm.item.trim()) {
    reqError.value = '要求条目不能为空'
    return
  }
  const statuses = reqForm.includePending
    ? ['在投', '已录用', '已发表', '已授权', '已获奖', '已登记'].join(',')
    : '' // 空 = 后端默认（在投不计）
  const input = {
    item: reqForm.item.trim(),
    needCount: reqForm.needCount,
    matchKinds: reqForm.kinds.join(','),
    matchStatus: statuses,
    note: reqForm.note.trim() || null,
    sort: editingReq.value?.sort,
  }
  try {
    if (editingReq.value) await updateRequirement(editingReq.value.id, input)
    else await createRequirement(input)
    reqDrawer.value = false
    await reload()
  } catch (e) {
    reqError.value = String(e)
  }
}

async function removeReq() {
  if (!editingReq.value) return
  if (!confirmReqDelete.value) {
    confirmReqDelete.value = true
    return
  }
  await deleteRequirement(editingReq.value.id)
  reqDrawer.value = false
  await reload()
}

// ---- 清单导出 ----
async function exportList() {
  try {
    const { filename, content } = await achievementsMarkdown()
    const path = await exportFile(filename, content)
    message.value = `已导出并在资源管理器定位：${path}`
  } catch (e) {
    message.value = String(e)
  }
}

const KIND_COLORS: Record<string, string> = {
  期刊论文: '#8b5cf6',
  会议论文: '#3b7dd8',
  发明专利: '#d97706',
  实用新型: '#c2410c',
  软件著作权: '#0ea5a4',
  竞赛获奖: '#e0533d',
  其他: '#64748b',
}
</script>

<template>
  <div class="page wide">
    <div class="head">
      <h2>成果与毕业对照</h2>
      <button class="btn" type="button" @click="exportList">导出成果清单</button>
    </div>
    <p v-if="message" class="msg">{{ message }}</p>

    <!-- 对照 -->
    <section>
      <h3>毕业要求对照</h3>
      <div v-if="matches.length" class="m-grid">
        <div v-for="m in matches" :key="m.req.id" class="m-card" :class="{ ok: m.gap === 0 }">
          <div class="m-head">
            <span class="m-item">{{ m.req.item }}</span>
            <span class="m-count">{{ m.achieved }} / {{ m.req.needCount }}</span>
          </div>
          <div class="m-bar">
            <i :style="{ width: `${(m.achieved / m.req.needCount) * 100}%` }" />
          </div>
          <div class="m-meta">
            <span v-if="m.gap === 0" class="tag ok">✓ 达标</span>
            <span v-else class="tag gap">缺 {{ m.gap }} 项</span>
            <span v-if="m.gap > 0 && pendingTotal > 0" class="tag pend">
              在途 {{ pendingTotal }}（论文流水线 + 在投）
            </span>
            <span class="edit" role="button" @click="editingReq = m.req; reqDrawer = true">
              编辑
            </span>
          </div>
          <p v-if="m.req.note" class="m-note">{{ m.req.note }}</p>
        </div>
      </div>
      <p v-else class="empty">
        学院毕业要求尚未发布 —— 发布后点上方「＋ 录入要求」逐条填入即可自动对照；
        未录入不影响其他功能。
        <button class="btn sm" type="button" style="margin-left: 8px" @click="editingReq = null; reqDrawer = true">
          ＋ 录入要求
        </button>
      </p>
      <button
        v-if="matches.length"
        class="btn sm"
        type="button"
        @click="editingReq = null; reqDrawer = true"
      >
        ＋ 录入要求
      </button>
    </section>

    <!-- 成果库 -->
    <section>
      <h3>成果库 · {{ achievements.length }} 项</h3>
      <button class="btn sm" type="button" @click="openAchCreate">＋ 登记成果</button>
      <div v-if="achievements.length" class="a-list">
        <div v-for="a in achievements" :key="a.id" class="a-row" @click="openAchEdit(a)">
          <span class="a-kind" :style="{ background: KIND_COLORS[a.kind] ?? '#64748b' }">
            {{ a.kind }}
          </span>
          <span class="a-title">{{ a.title }}</span>
          <span class="a-meta">
            {{ a.authorRank ?? '' }}{{ a.venue ? ` · ${a.venue}` : '' }}{{ a.date ? ` · ${a.date.slice(0, 7)}` : '' }}
          </span>
          <span class="a-status" :class="{ pend: a.status === '在投' }">{{ a.status }}</span>
          <span v-if="a.paperId" class="a-auto" title="论文录用后自动生成">📄 联动</span>
        </div>
      </div>
      <p v-else class="empty">
        还没有登记的成果 —— 论文流水线流转到「录用」会自动生成条目，也可手动登记专利/获奖等
      </p>
    </section>
  </div>

  <!-- 成果抽屉 -->
  <div v-if="achDrawer" class="mask" @click.self="achDrawer = false">
    <div class="drawer">
      <div class="d-head">
        <span>{{ editingAch ? '编辑成果' : '登记成果' }}</span>
        <button class="close" type="button" @click="achDrawer = false">✕</button>
      </div>
      <div class="d-body">
        <label class="f span2">
          <span>类型</span>
          <select v-model="achForm.kind">
            <option v-for="k in ACHIEVEMENT_KINDS" :key="k">{{ k }}</option>
          </select>
        </label>
        <label class="f span2">
          <span>标题</span>
          <input v-model="achForm.title" />
        </label>
        <label class="f">
          <span>作者排序</span>
          <input v-model="achForm.authorRank" placeholder="第一作者 / 通讯作者…" />
        </label>
        <label class="f">
          <span>状态</span>
          <select v-model="achForm.status">
            <option v-for="s in ACHIEVEMENT_STATUSES" :key="s">{{ s }}</option>
          </select>
        </label>
        <label class="f">
          <span>日期</span>
          <input v-model="achForm.date" type="date" />
        </label>
        <label class="f">
          <span>期刊 / 颁发单位</span>
          <input v-model="achForm.venue" />
        </label>
        <label class="f">
          <span>DOI / 链接</span>
          <input v-model="achForm.doi" />
        </label>
        <label class="f">
          <span>支撑课题</span>
          <select v-model="achForm.projectId">
            <option :value="null">不关联</option>
            <option v-for="p in projects.projects" :key="p.id" :value="p.id">{{ p.name }}</option>
          </select>
        </label>
        <label class="f span2">
          <span>备注</span>
          <textarea v-model="achForm.note" rows="2" />
        </label>
        <p v-if="achError" class="error">{{ achError }}</p>
      </div>
      <div class="d-foot">
        <button class="btn danger" type="button" @click="removeAch">
          {{ confirmAchDelete ? '确认删除？' : '删除' }}
        </button>
        <button class="btn primary" type="button" @click="saveAch">保存</button>
      </div>
    </div>
  </div>

  <!-- 要求抽屉 -->
  <div v-if="reqDrawer" class="mask" @click.self="reqDrawer = false">
    <div class="drawer">
      <div class="d-head">
        <span>{{ editingReq ? '编辑毕业要求' : '录入毕业要求' }}</span>
        <button class="close" type="button" @click="reqDrawer = false">✕</button>
      </div>
      <div class="d-body">
        <label class="f span2">
          <span>要求条目</span>
          <input v-model="reqForm.item" placeholder="如：SCI 一区/二区论文" />
        </label>
        <label class="f">
          <span>需要数量</span>
          <input v-model.number="reqForm.needCount" type="number" min="1" />
        </label>
        <div class="f span2">
          <span>匹配成果类型（不选 = 不限）</span>
          <div class="kinds">
            <button
              v-for="k in ACHIEVEMENT_KINDS"
              :key="k"
              type="button"
              class="k-chip"
              :class="{ on: reqForm.kinds.includes(k) }"
              @click="toggleKind(k)"
            >
              {{ k }}
            </button>
          </div>
        </div>
        <label class="f span2 check">
          <input v-model="reqForm.includePending" type="checkbox" />
          「在投」成果计入达标（默认只计已录用/已发表/已授权/已获奖/已登记）
        </label>
        <label class="f span2">
          <span>依据（培养方案文件与版本，便于核对）</span>
          <input v-model="reqForm.note" placeholder="如：《研究生培养方案（2027 版）》学位细则第 5 条" />
        </label>
        <p v-if="reqError" class="error">{{ reqError }}</p>
      </div>
      <div class="d-foot">
        <button class="btn danger" type="button" @click="removeReq">
          {{ confirmReqDelete ? '确认删除？' : '删除' }}
        </button>
        <button class="btn primary" type="button" @click="saveReq">保存</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.page.wide {
  max-width: 1080px;
  padding: 18px 24px;
}

.head {
  display: flex;
  align-items: center;
  gap: 12px;
}

.head h2 {
  font-size: 16px;
  font-weight: 500;
  margin: 0;
}

section {
  margin-top: 20px;
}

h3 {
  font-size: 13px;
  font-weight: 500;
  color: var(--muted-fg);
  margin: 0 0 8px;
}

.btn {
  border: 1px solid var(--line);
  background: var(--card);
  border-radius: 8px;
  padding: 7px 14px;
  cursor: pointer;
}

.btn.sm {
  padding: 4px 10px;
  font-size: 12px;
}

.btn.primary {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--accent-fg);
}

.btn.danger {
  color: var(--danger);
}

.msg {
  font-size: 12px;
  color: var(--ok);
  margin: 8px 0 0;
  word-break: break-all;
}

/* 对照卡片 */
.m-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 10px;
  margin-bottom: 10px;
}

.m-card {
  border: 1px solid var(--line);
  border-radius: var(--radius);
  padding: 12px 14px;
  background: var(--card);
}

.m-card.ok {
  border-color: color-mix(in srgb, var(--ok) 45%, var(--line));
}

.m-head {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: 8px;
}

.m-item {
  font-weight: 500;
  font-size: 13px;
}

.m-count {
  font-size: 13px;
  color: var(--muted-fg);
  font-variant-numeric: tabular-nums;
  flex: none;
}

.m-bar {
  height: 7px;
  border-radius: 4px;
  background: var(--hover);
  overflow: hidden;
  margin: 8px 0;
}

.m-bar i {
  display: block;
  height: 100%;
  background: var(--accent);
  border-radius: 4px;
}

.m-card.ok .m-bar i {
  background: var(--ok);
}

.m-meta {
  display: flex;
  gap: 6px;
  align-items: center;
  flex-wrap: wrap;
}

.tag {
  font-size: 11px;
  border-radius: 9px;
  padding: 1px 8px;
}

.tag.ok {
  background: color-mix(in srgb, var(--ok) 14%, transparent);
  color: var(--ok);
}

.tag.gap {
  background: color-mix(in srgb, var(--danger) 12%, transparent);
  color: var(--danger);
}

.tag.pend {
  background: color-mix(in srgb, var(--warn) 14%, transparent);
  color: var(--warn);
}

.edit {
  margin-left: auto;
  font-size: 12px;
  color: var(--muted-fg);
  cursor: pointer;
}

.edit:hover {
  color: var(--fg);
}

.m-note {
  margin: 6px 0 0;
  font-size: 11px;
  color: var(--muted-fg);
}

.empty {
  color: var(--muted-fg);
  font-size: 13px;
}

/* 成果列表 */
.a-list {
  margin-top: 10px;
  border: 1px solid var(--line);
  border-radius: var(--radius);
  background: var(--card);
}

.a-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 14px;
  border-bottom: 1px solid var(--line);
  cursor: pointer;
  font-size: 13px;
}

.a-row:last-child {
  border-bottom: none;
}

.a-row:hover {
  background: var(--hover);
}

.a-kind {
  font-size: 10px;
  color: #fff;
  border-radius: 6px;
  padding: 2px 7px;
  flex: none;
}

.a-title {
  font-weight: 500;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.a-meta {
  color: var(--muted-fg);
  font-size: 12px;
  margin-left: auto;
  flex: none;
  max-width: 46%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.a-status {
  font-size: 11px;
  color: var(--ok);
  flex: none;
}

.a-status.pend {
  color: var(--warn);
}

.a-auto {
  font-size: 10px;
  color: var(--muted-fg);
  border: 1px dashed var(--line);
  border-radius: 8px;
  padding: 0 6px;
  flex: none;
}

/* 抽屉（与论文页同款） */
.mask {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  justify-content: flex-end;
  z-index: 50;
}

.drawer {
  width: 380px;
  max-width: 100%;
  height: 100%;
  background: var(--card);
  border-left: 1px solid var(--line);
  display: flex;
  flex-direction: column;
}

.d-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 13px 16px;
  border-bottom: 1px solid var(--line);
  font-weight: 500;
}

.close {
  border: none;
  background: none;
  color: var(--muted-fg);
  cursor: pointer;
}

.d-body {
  flex: 1;
  overflow-y: auto;
  padding: 14px 16px;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  align-content: start;
}

.span2 {
  grid-column: span 2;
}

.f {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
  color: var(--muted-fg);
}

.f input,
.f select,
.f textarea {
  padding: 7px 9px;
  border: 1px solid var(--line);
  border-radius: 7px;
  background: var(--bg);
  color: var(--fg);
  outline: none;
  font-size: 13px;
}

.f input:focus,
.f select:focus,
.f textarea:focus {
  border-color: var(--accent);
}

.check {
  flex-direction: row;
  align-items: center;
  gap: 7px;
  font-size: 12px;
  cursor: pointer;
}

.kinds {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

.k-chip {
  border: 1px solid var(--line);
  background: var(--bg);
  border-radius: 9px;
  padding: 2px 9px;
  font-size: 12px;
  color: var(--muted-fg);
  cursor: pointer;
}

.k-chip.on {
  background: color-mix(in srgb, var(--accent) 14%, transparent);
  border-color: var(--accent);
  color: var(--accent);
}

.error {
  grid-column: span 2;
  color: var(--danger);
  font-size: 12px;
  margin: 0;
}

.d-foot {
  display: flex;
  justify-content: space-between;
  padding: 12px 16px;
  border-top: 1px solid var(--line);
}
</style>
