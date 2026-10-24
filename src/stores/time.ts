import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  createDdl,
  createMilestone,
  deleteDdl,
  deleteMilestone,
  listDdls,
  listMilestones,
  rolloverDdl,
  updateDdl,
  updateMilestone,
  type Ddl,
  type DdlInput,
  type Milestone,
  type MilestoneInput,
} from '../api/commands'

export const DDL_CATEGORIES = ['会议截稿', '期刊返修', '奖学金', '竞赛', '答辩窗口', '其他']
export const MILESTONE_STATUSES = ['未开始', '进行中', '已完成']

export const useTimeStore = defineStore('time', () => {
  const ddls = ref<Ddl[]>([])
  const milestones = ref<Milestone[]>([])
  const loaded = ref(false)

  async function loadDdls() {
    ddls.value = await listDdls(false)
  }
  async function loadMilestones() {
    milestones.value = await listMilestones()
  }

  async function load(force = false) {
    if (loaded.value && !force) return
    await Promise.all([loadDdls(), loadMilestones()])
    loaded.value = true
  }

  async function createD(input: DdlInput) {
    await createDdl(input)
    await loadDdls()
  }
  async function update(id: number, input: DdlInput) {
    await updateDdl(id, input)
    await loadDdls()
  }
  async function remove(id: number) {
    await deleteDdl(id)
    await loadDdls()
  }
  async function rollover(id: number) {
    await rolloverDdl(id)
    await loadDdls()
  }

  async function createM(input: MilestoneInput) {
    await createMilestone(input)
    await loadMilestones()
  }
  async function updateM(id: number, input: MilestoneInput) {
    await updateMilestone(id, input)
    await loadMilestones()
  }
  async function removeM(id: number) {
    await deleteMilestone(id)
    await loadMilestones()
  }

  return {
    ddls,
    milestones,
    load,
    loadDdls,
    loadMilestones,
    createD,
    update,
    remove,
    rollover,
    createM,
    updateM,
    removeM,
  }
})
