-- 科迹 SciTrail · schema v2：论文投稿流水线（M2-2，说明书 §4.8）
-- 放榜提醒不另建表：保存论文时同步生成/办结一条 ddl（数据一处存储）

CREATE TABLE paper (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  title           TEXT NOT NULL,
  venue           TEXT,                          -- 目标期刊/会议
  stage           TEXT NOT NULL DEFAULT '构思',  -- 构思/撰写中/组内审/投稿中/审稿中/返修/录用/见刊
  submit_date     TEXT,                          -- 投稿日期
  expect_reply    TEXT,                          -- 预计放榜（自动生成 DDL）
  revise_deadline TEXT,                          -- 返修截止
  project_id      INTEGER REFERENCES project(id) ON DELETE SET NULL,
  ddl_id          INTEGER REFERENCES ddl(id) ON DELETE SET NULL,  -- 关联的放榜 DDL
  note            TEXT,
  owner           INTEGER NOT NULL DEFAULT 1,
  deleted_at      TEXT,
  created_at      TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
  updated_at      TEXT
);
CREATE INDEX idx_paper_stage ON paper(deleted_at, stage);
