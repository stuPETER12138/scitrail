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
import FormDrawer from '../components/FormDrawer.vue'
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
  await deleteAchievement(editingAch.value.id)
  achDrawer.value = false
  await reload()
}

// ---- 毕业要求编辑抽屉 ----
const reqDrawer = ref(false)
const editingReq = ref<GradRequirement | null>(null)
const reqError = ref('')

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
  <FormDrawer
    v-if="achDrawer"
    :title="editingAch ? '编辑成果' : '登记成果'"
    @close="achDrawer = false"
    @save="saveAch"
    @delete="removeAch"
  >
    <div class="form-grid">
      <label class="field span2">
        <span>类型</span>
        <select v-model="achForm.kind">
          <option v-for="k in ACHIEVEMENT_KINDS" :key="k">{{ k }}</option>
        </select>
      </label>
      <label class="field span2">
        <span>标题</span>
        <input v-model="achForm.title" />
      </label>
      <label class="field">
        <span>作者排序</span>
        <input v-model="achForm.authorRank" placeholder="第一作者 / 通讯作者…" />
      </label>
      <label class="field">
        <span>状态</span>
        <select v-model="achForm.status">
          <option v-for="s in ACHIEVEMENT_STATUSES" :key="s">{{ s }}</option>
        </select>
      </label>
      <label class="field">
        <span>日期</span>
        <input v-model="achForm.date" type="date" />
      </label>
      <label class="field">
        <span>期刊 / 颁发单位</span>
        <input v-model="achForm.venue" />
      </label>
      <label class="field">
        <span>DOI / 链接</span>
        <input v-model="achForm.doi" />
      </label>
      <label class="field">
        <span>支撑课题</span>
        <select v-model="achForm.projectId">
          <option :value="null">不关联</option>
          <option v-for="p in projects.projects" :key="p.id" :value="p.id">{{ p.name }}</option>
        </select>
      </label>
      <label class="field span2">
        <span>备注</span>
        <textarea v-model="achForm.note" rows="2" />
      </label>
      <p v-if="achError" class="error">{{ achError }}</p>
    </div>
  </FormDrawer>

  <!-- 要求抽屉 -->
  <FormDrawer
    v-if="reqDrawer"
    :title="editingReq ? '编辑毕业要求' : '录入毕业要求'"
    @close="reqDrawer = false"
    @save="saveReq"
    @delete="removeReq"
  >
    <div class="form-grid">
      <label class="field span2">
        <span>要求条目</span>
        <input v-model="reqForm.item" placeholder="如：SCI 一区/二区论文" />
      </label>
      <label class="field">
        <span>需要数量</span>
        <input v-model.number="reqForm.needCount" type="number" min="1" />
      </label>
      <div class="field span2">
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
      <label class="field check span2">
        <input v-model="reqForm.includePending" type="checkbox" />
        「在投」成果计入达标（默认只计已录用/已发表/已授权/已获奖/已登记）
      </label>
      <label class="field span2">
        <span>依据（培养方案文件与版本，便于核对）</span>
        <input v-model="reqForm.note" placeholder="如：《研究生培养方案（2027 版）》学位细则第 5 条" />
      </label>
      <p v-if="reqError" class="error">{{ reqError }}</p>
    </div>
  </FormDrawer>
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
  font-size: var(--text-lg);
  font-weight: 500;
  margin: 0;
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
  font-size: var(--text-md);
}

.m-count {
  font-size: var(--text-md);
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
  font-size: var(--text-xs);
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
  font-size: var(--text-sm);
  color: var(--muted-fg);
  cursor: pointer;
}

.edit:hover {
  color: var(--fg);
}

.m-note {
  margin: 6px 0 0;
  font-size: var(--text-xs);
  color: var(--muted-fg);
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
  font-size: var(--text-md);
}

.a-row:last-child {
  border-bottom: none;
}

.a-row:hover {
  background: var(--hover);
}

.a-kind {
  font-size: var(--text-2xs);
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
  font-size: var(--text-sm);
  margin-left: auto;
  flex: none;
  max-width: 46%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.a-status {
  font-size: var(--text-xs);
  color: var(--ok);
  flex: none;
}

.a-status.pend {
  color: var(--warn);
}

.a-auto {
  font-size: var(--text-2xs);
  color: var(--muted-fg);
  border: 1px dashed var(--line);
  border-radius: 8px;
  padding: 0 6px;
  flex: none;
}

/* 要求抽屉里的类型多选 chip */
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
  font-size: var(--text-sm);
  color: var(--muted-fg);
  cursor: pointer;
}

.k-chip.on {
  background: color-mix(in srgb, var(--accent) 14%, transparent);
  border-color: var(--accent);
  color: var(--accent);
}
</style>
