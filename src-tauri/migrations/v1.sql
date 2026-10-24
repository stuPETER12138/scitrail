-- 科迹 SciTrail · schema v1（《开发设计说明书》§4）
-- 时间约定：TEXT，本地时区，'YYYY-MM-DD[ HH:MM[:SS]]'
-- 公共约定：业务表带 owner（课题组化预留）、deleted_at（软删除）、created_at/updated_at

CREATE TABLE project (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  name        TEXT NOT NULL,
  type        TEXT NOT NULL DEFAULT '自选',   -- 学位课题/导师纵向/横向/自选/申报
  role        TEXT,                           -- 负责人/核心参与/参与
  status      TEXT NOT NULL DEFAULT '进行中', -- 进行中/暂停/已结题/归档
  color       TEXT,
  start_date  TEXT,
  end_date    TEXT,
  note        TEXT,
  owner       INTEGER NOT NULL DEFAULT 1,
  deleted_at  TEXT,
  created_at  TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
  updated_at  TEXT
);

CREATE TABLE milestone (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  name        TEXT NOT NULL,
  target_date TEXT,
  status      TEXT NOT NULL DEFAULT '未开始', -- 未开始/进行中/已完成
  done_at     TEXT,
  note        TEXT,
  owner       INTEGER NOT NULL DEFAULT 1,
  deleted_at  TEXT,
  created_at  TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
  updated_at  TEXT
);

CREATE TABLE task (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  title        TEXT NOT NULL,
  note         TEXT,
  project_id   INTEGER REFERENCES project(id) ON DELETE SET NULL,   -- NULL = 收件箱
  milestone_id INTEGER REFERENCES milestone(id) ON DELETE SET NULL,
  priority     INTEGER NOT NULL DEFAULT 2,    -- 0高 1中 2低
  due_date     TEXT,
  status       TEXT NOT NULL DEFAULT '待办',  -- 待办/进行中/完成/放弃
  repeat_rule  TEXT,                          -- 如 'weekly:1'
  completed_at TEXT,                          -- 完成时刻，轨迹投影用
  owner        INTEGER NOT NULL DEFAULT 1,
  deleted_at   TEXT,
  created_at   TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
  updated_at   TEXT
);
CREATE INDEX idx_task_view    ON task(deleted_at, status, due_date);
CREATE INDEX idx_task_project ON task(project_id);

CREATE TABLE ddl (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  name          TEXT NOT NULL,
  category      TEXT,                    -- 会议截稿/期刊返修/奖学金/竞赛/答辩窗口/其他
  due_date      TEXT NOT NULL,
  lead_days     TEXT NOT NULL DEFAULT '7,3,1',
  project_id    INTEGER REFERENCES project(id) ON DELETE SET NULL,
  status        TEXT NOT NULL DEFAULT '生效',  -- 生效/已办结/已取消
  repeat_yearly INTEGER NOT NULL DEFAULT 0,
  note          TEXT,
  owner         INTEGER NOT NULL DEFAULT 1,
  deleted_at    TEXT,
  created_at    TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
  updated_at    TEXT
);
CREATE INDEX idx_ddl_due ON ddl(status, due_date);

CREATE TABLE entry (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  date       TEXT NOT NULL,              -- 业务日期，可补录
  time       TEXT,                       -- 'HH:MM'，可空 = 未知时刻（不虚构时间）
  kind       TEXT NOT NULL CHECK (kind IN ('experiment','literature','expense','meeting','note')),
  title      TEXT NOT NULL,
  content    TEXT NOT NULL DEFAULT '',
  project_id INTEGER REFERENCES project(id) ON DELETE SET NULL,
  tags       TEXT NOT NULL DEFAULT '[]',
  fields     TEXT NOT NULL DEFAULT '{}',
  refs       TEXT NOT NULL DEFAULT '[]',
  owner      INTEGER NOT NULL DEFAULT 1,
  deleted_at TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
  updated_at TEXT
);
CREATE INDEX idx_entry_date    ON entry(date);
CREATE INDEX idx_entry_kind    ON entry(kind, date);
CREATE INDEX idx_entry_project ON entry(project_id);

CREATE VIRTUAL TABLE entry_fts USING fts5(
  title, content, content='entry', content_rowid='id', tokenize='unicode61'
);
CREATE TRIGGER entry_ai AFTER INSERT ON entry BEGIN
  INSERT INTO entry_fts(rowid, title, content) VALUES (new.id, new.title, new.content);
END;
CREATE TRIGGER entry_ad AFTER DELETE ON entry BEGIN
  INSERT INTO entry_fts(entry_fts, rowid, title, content) VALUES ('delete', old.id, old.title, old.content);
END;
CREATE TRIGGER entry_au AFTER UPDATE ON entry BEGIN
  INSERT INTO entry_fts(entry_fts, rowid, title, content) VALUES ('delete', old.id, old.title, old.content);
  INSERT INTO entry_fts(rowid, title, content) VALUES (new.id, new.title, new.content);
END;

CREATE TABLE entry_history (
  id        INTEGER PRIMARY KEY AUTOINCREMENT,
  entry_id  INTEGER NOT NULL REFERENCES entry(id) ON DELETE CASCADE,
  snapshot  TEXT NOT NULL,
  saved_at  TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

CREATE TABLE weekly_report (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  week_start   TEXT NOT NULL,
  content      TEXT NOT NULL,
  generated_at TEXT,
  edited_at    TEXT,
  owner        INTEGER NOT NULL DEFAULT 1,
  deleted_at   TEXT
);
CREATE UNIQUE INDEX idx_weekly ON weekly_report(week_start);

CREATE TABLE setting (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE backup_log (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  file        TEXT NOT NULL,
  size_bytes  INTEGER,
  status      TEXT NOT NULL DEFAULT 'ok',
  created_at  TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

CREATE VIRTUAL TABLE task_fts USING fts5(
  title, note, content='task', content_rowid='id', tokenize='unicode61'
);
CREATE TRIGGER task_ai AFTER INSERT ON task BEGIN
  INSERT INTO task_fts(rowid, title, note) VALUES (new.id, new.title, COALESCE(new.note, ''));
END;
CREATE TRIGGER task_ad AFTER DELETE ON task BEGIN
  INSERT INTO task_fts(task_fts, rowid, title, note) VALUES ('delete', old.id, old.title, COALESCE(old.note, ''));
END;
CREATE TRIGGER task_au AFTER UPDATE ON task BEGIN
  INSERT INTO task_fts(task_fts, rowid, title, note) VALUES ('delete', old.id, old.title, COALESCE(old.note, ''));
  INSERT INTO task_fts(rowid, title, note) VALUES (new.id, new.title, COALESCE(new.note, ''));
END;
