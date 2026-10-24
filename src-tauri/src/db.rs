use crate::migration;
use rusqlite::Connection;
use std::path::Path;

/// 打开数据库并跑到最新 schema（WAL + 外键 + 迁移）
pub fn init(path: &Path) -> Result<Connection, rusqlite::Error> {
    let mut conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    // journal_mode 是少数返回结果行的 PRAGMA，需读回而非 update
    let _mode: String = conn.query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))?;
    migration::migrations()
        .to_latest(&mut conn)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    pub(crate) fn test_conn() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        migration::migrations().to_latest(&mut conn).unwrap();
        conn
    }

    #[test]
    fn migrations_create_all_tables() {
        let conn = test_conn();
        for table in [
            "project",
            "milestone",
            "task",
            "ddl",
            "entry",
            "entry_history",
            "weekly_report",
            "setting",
            "backup_log",
            "entry_fts",
            "task_fts",
        ] {
            let n: i64 = conn
                .query_row(
                    &format!("SELECT COUNT(*) FROM sqlite_master WHERE name = '{table}'"),
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(n, 1, "表 {table} 未创建");
        }
    }

    #[test]
    fn fts5_available_and_insert_synced() {
        // P0 验收项：bundled SQLite 的 FTS5 可用（说明书 §3 的待验证项）
        let conn = test_conn();
        conn.execute(
            "INSERT INTO entry(date, kind, title, content) VALUES('2026-10-24','note','sinter gradient test','XPS analysis')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO entry(date, kind, title, content) VALUES('2026-10-24','note','另一条中文记录','内容')",
            [],
        )
        .unwrap();
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM entry_fts WHERE entry_fts MATCH 'sinter'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 1, "FTS5 不可用或插入触发器未同步");
        // 中文按 unicode61 不分词，检索走 LIKE 兜底（说明书 §3 约定）
        let m: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM entry WHERE title LIKE '%中文%'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(m, 1);
    }

    #[test]
    fn fts_update_and_delete_synced() {
        let conn = test_conn();
        conn.execute(
            "INSERT INTO entry(date, kind, title) VALUES('2026-10-24','note','alpha beta')",
            [],
        )
        .unwrap();
        conn.execute("UPDATE entry SET title = 'gamma delta' WHERE id = 1", [])
            .unwrap();
        let (a, g): (i64, i64) = conn
            .query_row(
                "SELECT
                   (SELECT COUNT(*) FROM entry_fts WHERE entry_fts MATCH 'alpha'),
                   (SELECT COUNT(*) FROM entry_fts WHERE entry_fts MATCH 'gamma')",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!((a, g), (0, 1), "更新触发器未同步");
        conn.execute("DELETE FROM entry WHERE id = 1", []).unwrap();
        let d: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM entry_fts WHERE entry_fts MATCH 'gamma'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(d, 0, "删除触发器未同步");
    }

    #[test]
    fn setting_roundtrip() {
        let conn = test_conn();
        crate::services::settings::set(&conn, "k", "v").unwrap();
        assert_eq!(
            crate::services::settings::get(&conn, "k")
                .unwrap()
                .as_deref(),
            Some("v")
        );
        crate::services::settings::set(&conn, "k", "v2").unwrap();
        assert_eq!(
            crate::services::settings::get(&conn, "k")
                .unwrap()
                .as_deref(),
            Some("v2")
        );
        conn.execute("DELETE FROM setting WHERE key='k'", params![])
            .unwrap();
    }
}
