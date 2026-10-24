# 方案 E · 全功能科研工作台（OpenProject 式）

| 一句话定位 | 按需求报告 v1.0 全模块建齐：仪表盘 + 13 个功能页 + 范式化数据库，一步到位的"科研管理系统" |
|---|---|
| 交互重心 | 仪表盘 + 模块化页面（表格/看板/图表并用） |
| 数据范式 | 完整范式化关系型（约 16 张表） |
| MVP 工作量 | 约 15~20 个集中开发日（M1），完整版 8~10 周 |
| 成长路径 | ★★★★★ 终点站形态；且是方案 A 的超集 |

---

## 1. 核心理念

- 不做减法：需求报告 4.1~4.14 的每个模块都建成**独立页面 + 独立表**，模块间用外键严格关联；
- 仪表盘是**真仪表盘**：毕业倒计时（2032-06）、成果缺口、30 天 DDL、课题红绿灯，一屏四区常驻；
- 界面走"工作台"路线：高密度表格 + 筛选器 + 详情抽屉 + 图表（ECharts），像 OpenProject/飞书项目一样"系统味"十足；
- 为将来推广课题组预留的结构（owner 字段、软删除、审计日志）**从第一天就建**，不是预留口子而是直接实现。

## 2. 借鉴的开源项目

| 项目 | 借鉴点 |
|---|---|
| [OpenProject](https://github.com/opf/openproject) | Work Package 统一抽象 + 模块化页面（甘特/看板/表格/日历同源）；成熟的开源 PM 数据模型 |
| [Taiga](https://github.com/taigaio/taiga) | Django 系分层：milestone / userstory / task / issue 四类工作对象 + 项目级聚合页 |
| [Vikunja](https://vikunja.io) | 一份数据多种视图的查询层设计；提醒/订阅模型 |
| [eLabFTW](https://github.com/elabftw/elabftw) | 实验条目的时间戳/留痕字段设计 |

## 3. 界面设计

### 3.1 布局线框（主窗口：仪表盘）

```
┌────────────────────────────────────────────────────────────────────┐
│ 📊仪表盘 📁课题 ✅任务 ⏰DDL 🧪实验 📄论文 🏆成果 💰报销 📖文献      │
│ 🗄资料 📆周报 📈统计 ⚙设置                    [⌘K速录] [🔔3]        │
├────────────────────────────────────────────────────────────────────┤
│ 🎓 距预计毕业 2032-06 还有 2062 天 │ 本学期第 4 周 │ ⏰30天DDL: 3   │
├──────────────────┬──────────────────────┬──────────────────────────┤
│ 课题红绿灯        │ 今日任务 (4)          │ 毕业成果对照              │
│ 🟢 催化课题 72%   │ ○ 修正常压炉参数 !高  │ SCI(1区2区): ▓▓░ 需1缺1  │
│ 🟡 论文A 45%     │ ○ 回复审稿意见        │   在途: ACSAMI审稿中     │
│ 🟢 国自然预研 30% │ ○ 组会PPT            │ EI: ▓▓▓ 已达标 ✅        │
│ 🔴 申报 逾期任务1 │ ○ 生成周报草稿        │ 专利: ▓░░ 需1缺1         │
├──────────────────┴──────────────────────┴──────────────────────────┤
│ 🧪 最近实验: 10-23 预烧结1150℃ → 10-21 XPS对比 → 10-19 …          │
└────────────────────────────────────────────────────────────────────┘
```

### 3.2 页面清单（13 页，对应需求报告第 7 节）

仪表盘 / 课题列表+详情（聚合时间线）/ 任务（今日+看板+按课题）/ DDL 时间轴 / 实验记录 / 论文看板 / 成果与毕业对照 / 报销流水 / 文献清单 / 资料索引 / 周报助手 / 统计图表 / 设置（含字典管理）

### 3.3 关键交互

- 每个模块页标配：**筛选器 + 排序 + 分页表格 + 详情抽屉**（Element Plus 组件化，风格统一）；
- 论文页到成果页的**自动流转**：论文标记"录用"时弹窗引导生成成果条目；
- **毕业对照实时计算**：成果变动 → 对照表立即刷新，缺口用红色角标常驻仪表盘；
- 全局搜索跨 13 个模块（FTS 联合查询）。

### 3.4 视觉风格

Ant Design / OpenProject 工作台风：深色侧边栏、卡片化仪表盘、ECharts 图表；信息密度最高，"系统感"最强。

## 4. 数据库设计

### 4.1 表清单（16 张，范式化）

| 表 | 职责 | 表 | 职责 |
|---|---|---|---|
| user | 单用户占位（含口令哈希） | paper | 论文（阶段/投稿日期/预计放榜） |
| project | 课题 | achievement | 成果（类型/作者排序/DOI） |
| project_member | 成员（预留课题组化） | grad_requirement | 毕业要求条款 |
| milestone | 学位里程碑 | grad_req_map | 要求↔成果对照 |
| task | 任务 | expense | 报销垫付 |
| ddl | DDL 条目 | literature | 文献 |
| experiment | 实验记录 | doc_ref | 资料索引 |
| meeting_note | 组会记录 | weekly_report | 周报（生成后存档） |
| attachment | 附件索引 | notification | 提醒（本地） |
| audit_log | 审计日志 | setting | 配置/字典 |

> 所有业务表带 `owner`、`deleted_at`（软删除）、`created_at/updated_at`。

### 4.2 核心表 DDL 示例（SQLite）

```sql
CREATE TABLE task (
  id INTEGER PRIMARY KEY, owner INTEGER NOT NULL DEFAULT 1,
  title TEXT NOT NULL, note TEXT,
  project_id INTEGER REFERENCES project(id) ON DELETE SET NULL,
  milestone_id INTEGER REFERENCES milestone(id) ON DELETE SET NULL,
  priority INTEGER DEFAULT 2, due_date TEXT,
  status TEXT DEFAULT '待办', repeat_rule TEXT,
  completed_at TEXT, deleted_at TEXT,
  created_at TEXT DEFAULT (datetime('now')),
  updated_at TEXT
);
CREATE INDEX idx_task_status ON task(deleted_at, status, due_date);

CREATE TABLE paper (
  id INTEGER PRIMARY KEY, owner INTEGER NOT NULL DEFAULT 1,
  title TEXT, venue TEXT, stage TEXT NOT NULL DEFAULT '构思',
  submit_date TEXT, expect_reply TEXT,   -- 到期自动生成 DDL
  project_id INTEGER REFERENCES project(id),
  note TEXT, deleted_at TEXT,
  created_at TEXT DEFAULT (datetime('now')), updated_at TEXT
);

CREATE TABLE grad_requirement (
  id INTEGER PRIMARY KEY,
  item TEXT NOT NULL,        -- 如 'SCI(一区/二区)'
  need_count INTEGER DEFAULT 1,
  note TEXT, source TEXT,    -- 依据文件与版本；要求未发布时整表为空
  deleted_at TEXT
);
```

### 4.3 设计取舍

- **严格范式化** vs 方案 B/C 的 JSON：查询、统计、约束全部交给数据库；代价是每加一个模块要写迁移脚本——但模块清单在需求报告里已冻结，可接受；
- **audit_log 从第一天就有**：单机系统翻车时它是唯一的事故现场还原手段；
- **weekly_report 落表**（方案 A 不落）：因为 E 的周报要支持回溯编辑和跨学期统计。

## 5. 对需求报告的覆盖度

| 需求模块 | 覆盖情况 |
|---|---|
| P0 全部 + P1 全部（论文流水/成果对照/报销/文献/资料/统计） | ✅ **五案唯一全量覆盖** |
| P2（甘特/日历/统计图表） | ⚠️ 甘特可后补，其余预留接口 |

## 6. 优缺点与风险

**优点**：一步到位，与需求报告一一对应；数据模型最严谨；将来推广课题组时**代码即蓝图**（v0.1 存档版的实现起点）。

**缺点**：开发周期最长（完整版 8~10 周）；界面工具味重，"记一条实验"要进出模块页，30 秒原则靠速记浮窗硬撑；单人使用前期大部分功能是空的，**空系统最容易劝退使用者**。

**风险**：范围大导致烂尾（仅次于 C）；建议即使选 E 也必须按 M1→M3 分期上线，绝不憋大招。

## 7. 工作量估计

M1（登录/课题/任务/DDL/仪表盘/速录）：8~10 天；M2（实验/周报/备份）：+4 天；M3（论文/成果/报销/文献/资料/统计）：+8~10 天。

## 8. 何时选它

不赶时间、想要"毕其功于一役"、享受搭系统本身——选 E。它也是方案 A 明确的成长终点：**先 A 后 E 是同一条路的两个阶段**。
