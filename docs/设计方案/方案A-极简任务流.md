# 方案 A · 极简任务流（Todoist 式）

| 一句话定位 | 万物皆任务，课题只是任务的智能分组——用最少的表和最快的录入把系统先用起来 |
|---|---|
| 交互重心 | 任务列表 / 今日视图 |
| 数据范式 | 经典关系型（范式化表结构） |
| MVP 工作量 | 约 6~8 个集中开发日 |
| 成长路径 | ★★★★★ 与方案 E 同范式，可平滑长成全功能版 |

---

## 1. 核心理念

- 系统只有一个一等公民：**任务**。课题、里程碑、DDL 都是为"决定下一个任务做什么"服务的；
- 课题 = 一个带过滤条件的任务列表（类似 Todoist 的 Project / Kanboard 的 Swimlane），不做复杂的项目对象；
- 实验记录、文献等做成**轻量表**挂在任务体系旁边，不追求结构完备，先保证"记得下来"；
- 所有界面为 30 秒原则让路：全局快捷键 + 速记语法 + 默认值，点开即录。

## 2. 借鉴的开源项目

| 项目 | 借鉴点 |
|---|---|
| [Kanboard](https://kanboard.org) | 极简主义哲学：PHP + 可选 SQLite 单文件，表结构极小（约 20 张表撑起全部功能）；证明"小表 + 多视图"够用 |
| [Vikunja](https://vikunja.io) | 任务模型：projects → tasks 两层 + labels + reminders + attachments；同一份数据渲染列表/看板/甘特/表格四种视图——视图只是查询，不是存储 |
| [Super Productivity](https://super-productivity.com) | "今日聚焦"交互：每天从全部任务里挑几条进 Today；本地优先、离线可用、无账号 |

## 3. 界面设计

### 3.1 布局线框（主窗口）

```
┌──────────────────────────────────────────────────────────────┐
│ ⌘K  [周五 完成XPS数据分析 #催化 !高 @ACSAMI]          回车即存 │
├────────────┬─────────────────────────────────────────────────┤
│ 视图        │ 今日（3）                                        │
│  ▣ 今日     │ ──────────────────────────────────────────────  │
│  ▣ 收件箱   │ ○ 修正常压炉温控参数        #催化   今天  !高     │
│  ▼ 课题     │ ○ 回复审稿意见-二轮         #论文A  今天         │
│    · 催化   │ ○ 组会PPT                   每周一               │
│    · 论文A  │                                                  │
│    · 国自然 │ 临近 DDL ────────────────────────────────────   │
│    · 申报   │ ⏳ ACSAMI 全文截稿      12 天  ▸关联任务(2)      │
│  ▣ DDL      │ ⏳ 奖学金申请            25 天                   │
│  ▣ 实验     │                                                  │
│  ▣ 周报     │ 本周小结 ─────────────────────────────────────  │
│  ⚙ 设置     │ 已完成 14 · 实验 3 次 · 周报未写 [生成草稿]       │
└────────────┴─────────────────────────────────────────────────┘
```

### 3.2 页面清单（7 页）

今日 / 收件箱 / 课题（列表=过滤视图，详情含任务+实验时间线）/ DDL 时间轴 / 实验（轻量时间线+搜索）/ 周报助手 / 设置

### 3.3 关键交互

- **速记语法**：`日期 + 事项 + #课题 + !优先级 + @DDL`，解析失败整条进收件箱，绝不弹错误框；
- **数字键切视图**：1=今日 2=收件箱 3=课题…全程不离键盘；
- **任务详情侧滑抽屉**：不跳页；详情内可一键"转为实验记录"或"挂到 DDL"；
- **周五按钮**：仪表盘常驻"生成周报草稿"，聚合本周完成任务 + 实验条目。

### 3.4 视觉风格

类 Things/Todoist 极简清单风：白底、单色强调、大量留白；任务行=复选框+标题+右侧元信息；无卡片阴影，信息密度可调（紧凑/舒适）。

## 4. 数据库设计

### 4.1 表清单（8 张）

| 表 | 职责 |
|---|---|
| project | 课题（含类型/角色/四态状态） |
| task | 任务（可空外键=收件箱任务） |
| milestone | 学位里程碑 |
| ddl | 独立 DDL 条目 |
| experiment | 实验流水（轻量结构化） |
| meeting_note | 组会记录 |
| label + task_label | 标签（备用，初期可只用 #课题） |
| setting | 配置（速记语法、备份路径、提醒阈值） |

> 周报**不落表**：永远是"任务+实验"的按周聚合视图，生成草稿后导出即走。

### 4.2 核心表 DDL（SQLite）

```sql
CREATE TABLE project (
  id         INTEGER PRIMARY KEY,
  name       TEXT NOT NULL,
  type       TEXT NOT NULL DEFAULT '自选',   -- 学位课题/导师纵向/横向/自选/申报
  role       TEXT,                            -- 负责人/核心参与/参与
  status     TEXT NOT NULL DEFAULT '进行中',  -- 进行中/暂停/已结题/归档
  start_date TEXT, end_date TEXT, note TEXT
);

CREATE TABLE task (
  id           INTEGER PRIMARY KEY,
  title        TEXT NOT NULL,
  note         TEXT,
  project_id   INTEGER REFERENCES project(id) ON DELETE SET NULL, -- NULL=收件箱
  milestone_id INTEGER REFERENCES milestone(id) ON DELETE SET NULL,
  priority     INTEGER NOT NULL DEFAULT 2,   -- 0高 1中 2低
  due_date     TEXT,
  status       TEXT NOT NULL DEFAULT '待办', -- 待办/进行中/完成/放弃
  repeat_rule  TEXT,                         -- 如 'weekly:1'（每周一）
  completed_at TEXT
);

CREATE TABLE ddl (
  id       INTEGER PRIMARY KEY,
  name     TEXT NOT NULL,
  category TEXT,                    -- 会议截稿/期刊返修/奖学金/竞赛/答辩窗口/其他
  due_date TEXT NOT NULL,
  lead_days TEXT NOT NULL DEFAULT '7,3,1',
  note     TEXT
);

CREATE TABLE experiment (
  id         INTEGER PRIMARY KEY,
  date       TEXT NOT NULL,         -- 可补录
  project_id INTEGER REFERENCES project(id),
  purpose    TEXT, conditions TEXT, params TEXT,
  sample_no  TEXT, data_path TEXT, conclusion TEXT,
  created_at TEXT DEFAULT (datetime('now'))
);

CREATE VIRTUAL TABLE task_fts USING fts5(title, note, content='task');
```

### 4.3 设计取舍

- **里程碑单独成表**而不是任务的一种：因为倒计时语义不同（计划 vs 死线）；
- **实验独立成表**而不是 task 的 subtype：工科实验有"参数/数据路径"等任务没有的字段，硬塞进任务表会催生一排 NULL；
- 索引只需三个：`task(status, due_date)`、`task(project_id)`、`experiment(date)`——单机数据量（万级）下足够。

## 5. 对需求报告的覆盖度

| 需求模块 | 覆盖情况 |
|---|---|
| 课题 / 任务收件箱 / DDL里程碑 / 仪表盘 / 周报 / 备份 | ✅ 完整（P0 全覆盖） |
| 实验记录 | ✅ 完整（但无"一键转任务"以外的联动） |
| 论文投稿流水线 | ⚠️ 需后补（P1 时加 paper 表 + 看板视图） |
| 成果对照 / 报销 / 文献 / 资料索引 | ⚠️ 需后补（各加 1 张轻量表即可） |

## 6. 优缺点与风险

**优点**：最快可用；录入摩擦最小；范式与方案 E 完全兼容，后期加表即可长成全功能版，**零迁移成本**。

**缺点**：初期"课题详情"较薄（没有聚合时间线的富度）；论文流水线等 P1 模块要等二期。

**风险**：模块都是后补的，需要守住"先加表不改范式"的纪律，避免中途推翻。

## 7. 工作量估计

M1（今日/收件箱/课题/DDL/速录/仪表盘）：6~8 个集中开发日；实验+周报：+3 天；备份导出：+1 天。

## 8. 何时选它

想**两周内用上**、日常以任务驱动、对"自由记录"需求不高（笔记另有去处）——选 A。它也是最稳的起点：哪怕最后想升级成方案 E，前面的开发全部保值。
