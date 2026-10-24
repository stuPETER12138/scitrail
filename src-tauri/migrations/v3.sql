-- 科迹 SciTrail · schema v3：成果库与毕业要求对照（M2-3，说明书 §4.9）
-- 对照采用"按类型自动匹配"（grad_requirement.match_kinds / match_status），免手动映射表

CREATE TABLE achievement (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  kind        TEXT NOT NULL,                    -- 期刊论文/会议论文/发明专利/实用新型/软件著作权/竞赛获奖/其他
  title       TEXT NOT NULL,
  author_rank TEXT,                             -- 第一作者/第二作者/通讯作者/…
  status      TEXT NOT NULL DEFAULT '已发表',   -- 在投/已录用/已发表/已授权/已获奖/已登记
  date        TEXT,                             -- 发表/授权/获奖日期
  venue       TEXT,                             -- 期刊/会议/颁发单位
  doi         TEXT,
  project_id  INTEGER REFERENCES project(id) ON DELETE SET NULL,
  paper_id    INTEGER REFERENCES paper(id) ON DELETE SET NULL,  -- 论文录用自动生成时关联（防重复）
  note        TEXT,
  owner       INTEGER NOT NULL DEFAULT 1,
  deleted_at  TEXT,
  created_at  TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
  updated_at  TEXT
);
CREATE INDEX idx_achievement_kind ON achievement(deleted_at, kind);

CREATE TABLE grad_requirement (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  item         TEXT NOT NULL,                   -- 如 'SCI 一区/二区论文'
  need_count   INTEGER NOT NULL DEFAULT 1,
  match_kinds  TEXT NOT NULL DEFAULT '',        -- 匹配成果类型，逗号分隔；空 = 不限
  match_status TEXT NOT NULL DEFAULT '',        -- 计入状态，逗号分隔；空 = 非"在投"全部
  note         TEXT,                            -- 依据（培养方案/学位细则文件与版本）
  sort         INTEGER NOT NULL DEFAULT 0,
  deleted_at   TEXT,
  created_at   TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
  updated_at   TEXT
);
