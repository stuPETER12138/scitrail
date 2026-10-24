//! 备份与导出（说明书 §4.6）：
//! 备份一律 VACUUM INTO 在线一致性快照（WAL 下裸拷贝可能不一致，禁止）；
//! 恢复 = 快照当前 → checkpoint → 替换主库 → 重开连接。

use crate::models::{Ddl, Entry, Milestone, Project, Task};
use chrono::Local;
use rusqlite::{Connection, params};
use serde::Serialize;
use std::path::Path;

const KEEP: usize = 30;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupLog {
    pub id: i64,
    pub file: String,
    pub size_bytes: Option<i64>,
    pub status: String,
    pub created_at: String,
}

/// 执行一次备份（VACUUM INTO + 记日志 + 清理超额）
pub fn run_backup(conn: &Connection, backups_dir: &Path) -> Result<String, String> {
    std::fs::create_dir_all(backups_dir).map_err(|e| e.to_string())?;
    let stamp = Local::now().format("%Y%m%d-%H%M%S");
    let mut name = format!("scitrail-{stamp}.db");
    let mut seq = 1;
    while backups_dir.join(&name).exists() {
        seq += 1;
        name = format!("scitrail-{stamp}-{seq}.db");
    }
    let path = backups_dir.join(&name);
    conn.execute(
        "VACUUM INTO ?1",
        params![path.to_string_lossy().to_string()],
    )
    .map_err(|e| e.to_string())?;
    let size = std::fs::metadata(&path).map(|m| m.len() as i64).ok();
    conn.execute(
        "INSERT INTO backup_log(file, size_bytes, status) VALUES(?1, ?2, 'ok')",
        params![name, size],
    )
    .map_err(|e| e.to_string())?;
    prune(conn, backups_dir)?;
    Ok(name)
}

fn prune(conn: &Connection, backups_dir: &Path) -> Result<(), String> {
    let rows: Vec<(i64, String)> = conn
        .prepare("SELECT id, file FROM backup_log ORDER BY id DESC")
        .map_err(|e| e.to_string())?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    if rows.len() > KEEP {
        for (id, file) in rows.iter().skip(KEEP) {
            let _ = std::fs::remove_file(backups_dir.join(file));
            conn.execute("DELETE FROM backup_log WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn list_backups(conn: &Connection) -> Result<Vec<BackupLog>, String> {
    let mut stmt = conn
        .prepare("SELECT id, file, size_bytes, status, created_at FROM backup_log ORDER BY id DESC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(BackupLog {
                id: r.get(0)?,
                file: r.get(1)?,
                size_bytes: r.get(2)?,
                status: r.get(3)?,
                created_at: r.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

/// 是否到了该备份的时刻：今天还没有成功备份（定时线程 15 分钟轮询 + 启动补查共用）
pub fn should_run(conn: &Connection) -> bool {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let last: Option<String> = conn
        .query_row(
            "SELECT substr(created_at,1,10) FROM backup_log WHERE status='ok' ORDER BY id DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .ok();
    last.as_deref() != Some(today.as_str())
}

/// 恢复前置：当前状态先快照 + WAL checkpoint，让主库文件可安全替换
pub fn prepare_restore(conn: &Connection, backups_dir: &Path) -> Result<(), String> {
    run_backup(conn, backups_dir)?;
    // wal_checkpoint 返回结果行，需读回而非 execute
    let _: (i64, i64, i64) = conn
        .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------- 全量导出 ----------

fn dump_core(conn: &Connection) -> Result<serde_json::Value, String> {
    let projects: Vec<Project> = conn
        .prepare(
            "SELECT id, name, type, role, status, color, start_date, end_date, note
             FROM project WHERE deleted_at IS NULL ORDER BY id",
        )
        .map_err(|e| e.to_string())?
        .query_map([], Project::from_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;

    let tasks: Vec<Task> = conn
        .prepare(
            "SELECT t.id, t.title, t.note, t.project_id, p.name, t.milestone_id, t.priority,
                    t.due_date, t.status, t.repeat_rule, t.completed_at, t.created_at
             FROM task t LEFT JOIN project p ON p.id = t.project_id
             WHERE t.deleted_at IS NULL ORDER BY t.id",
        )
        .map_err(|e| e.to_string())?
        .query_map([], Task::from_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;

    let entries: Vec<Entry> = conn
        .prepare(
            "SELECT id, date, time, kind, title, content, project_id, tags, fields, refs, created_at, updated_at
             FROM entry WHERE deleted_at IS NULL ORDER BY date, id",
        )
        .map_err(|e| e.to_string())?
        .query_map([], Entry::from_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;

    let ddls: Vec<Ddl> = conn
        .prepare(
            "SELECT id, name, category, due_date, lead_days, project_id, status, repeat_yearly, note
             FROM ddl WHERE deleted_at IS NULL ORDER BY due_date",
        )
        .map_err(|e| e.to_string())?
        .query_map([], Ddl::from_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;

    let milestones: Vec<Milestone> = conn
        .prepare(
            "SELECT id, name, target_date, status, done_at, note
             FROM milestone WHERE deleted_at IS NULL ORDER BY target_date",
        )
        .map_err(|e| e.to_string())?
        .query_map([], Milestone::from_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;

    let weeklies: Vec<(String, String)> = conn
        .prepare("SELECT week_start, content FROM weekly_report WHERE deleted_at IS NULL ORDER BY week_start")
        .map_err(|e| e.to_string())?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;

    let schema_version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap_or(0);

    Ok(serde_json::json!({
        "schemaVersion": schema_version,
        "exportedAt": Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        "projects": projects,
        "tasks": tasks,
        "entries": entries,
        "ddls": ddls,
        "milestones": milestones,
        "weeklyReports": weeklies.into_iter()
            .map(|(week_start, content)| serde_json::json!({"weekStart": week_start, "content": content}))
            .collect::<Vec<_>>(),
    }))
}

fn dump_markdown(conn: &Connection) -> Result<String, String> {
    let core = dump_core(conn)?;
    let mut md = String::from("# 科迹 SciTrail 全量导出（Markdown）\n\n");
    md.push_str(&format!(
        "导出时间：{}\n\n",
        core["exportedAt"].as_str().unwrap_or_default()
    ));

    md.push_str("## 课题\n");
    for p in core["projects"].as_array().unwrap_or(&vec![]) {
        md.push_str(&format!(
            "- {}（{} · {}）\n",
            p["name"], p["type"], p["status"]
        ));
    }

    md.push_str("\n## 任务\n");
    for t in core["tasks"].as_array().unwrap_or(&vec![]) {
        md.push_str(&format!(
            "- [{}] {}（{}）{}\n",
            if t["status"] == "完成" { "x" } else { " " },
            t["title"],
            t["status"].as_str().unwrap_or_default(),
            t["dueDate"].as_str().unwrap_or("")
        ));
    }

    md.push_str("\n## 轨迹条目\n");
    for e in core["entries"].as_array().unwrap_or(&vec![]) {
        md.push_str(&format!(
            "- {} {} [{}] {}\n",
            e["date"].as_str().unwrap_or_default(),
            e["time"].as_str().unwrap_or(""),
            e["kind"].as_str().unwrap_or_default(),
            e["title"]
        ));
    }

    md.push_str("\n## DDL\n");
    for d in core["ddls"].as_array().unwrap_or(&vec![]) {
        md.push_str(&format!(
            "- {} · {}（{}）\n",
            d["dueDate"]
                .as_str()
                .unwrap_or_default()
                .get(0..10)
                .unwrap_or_default(),
            d["name"],
            d["status"]
        ));
    }

    md.push_str("\n## 里程碑\n");
    for m in core["milestones"].as_array().unwrap_or(&vec![]) {
        md.push_str(&format!(
            "- {}（{} · {}）\n",
            m["name"], m["targetDate"], m["status"]
        ));
    }
    Ok(md)
}

/// 写入 exports 目录并返回路径；format: "json" | "md"
pub fn export(conn: &Connection, exports_dir: &Path, format: &str) -> Result<String, String> {
    std::fs::create_dir_all(exports_dir).map_err(|e| e.to_string())?;
    let stamp = Local::now().format("%Y%m%d-%H%M%S");
    let (name, content) = match format {
        "json" => (
            format!("scitrail-export-{stamp}.json"),
            serde_json::to_string_pretty(&dump_core(conn)?).map_err(|e| e.to_string())?,
        ),
        "md" => (format!("scitrail-export-{stamp}.md"), dump_markdown(conn)?),
        _ => return Err(format!("未知导出格式：{format}")),
    };
    let path = exports_dir.join(&name);
    std::fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().into_owned())
}

/// 周报导出为 .md 文件
pub fn export_weekly(
    conn: &Connection,
    exports_dir: &Path,
    week_start: &str,
) -> Result<String, String> {
    let content: Option<String> = conn
        .query_row(
            "SELECT content FROM weekly_report WHERE week_start = ?1 AND deleted_at IS NULL",
            params![week_start],
            |r| r.get(0),
        )
        .ok();
    let content = content.ok_or("该周周报不存在，请先生成")?;
    std::fs::create_dir_all(exports_dir).map_err(|e| e.to_string())?;
    let path = exports_dir.join(format!("周报-{week_start}.md"));
    std::fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().into_owned())
}

/// 通用文本导出（学期总结等）：文件名做净化，写入 exports 并返回路径
pub fn export_named(exports_dir: &Path, filename: &str, content: &str) -> Result<String, String> {
    const INVALID: [char; 9] = ['\\', '/', ':', '*', '?', '"', '<', '>', '|'];
    let safe: String = filename
        .chars()
        .map(|c| if INVALID.contains(&c) { '_' } else { c })
        .collect();
    if safe.trim().is_empty() {
        return Err("文件名无效".into());
    }
    std::fs::create_dir_all(exports_dir).map_err(|e| e.to_string())?;
    let path = exports_dir.join(safe);
    std::fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init as db_init;
    use crate::services::entries::{self, EntryInput};
    use std::env;

    fn temp_db() -> (std::path::PathBuf, Connection) {
        let dir = env::temp_dir().join(format!("scitrail-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.db");
        let conn = db_init(&path).unwrap();
        (dir, conn)
    }

    #[test]
    fn backup_restore_and_prune() {
        let (dir, conn) = temp_db();
        let backups = dir.join("backups");
        entries::create(
            &conn,
            EntryInput {
                date: None,
                time: None,
                kind: "note".into(),
                title: "会被备份的数据".into(),
                content: None,
                project_id: None,
                tags: None,
                fields: None,
                refs: None,
            },
        )
        .unwrap();

        let f1 = run_backup(&conn, &backups).unwrap();
        assert!(backups.join(&f1).exists());
        assert!(!should_run(&conn)); // 今天已备份

        // 导出 JSON 包含数据
        let json_path = export(&conn, &dir.join("exports"), "json").unwrap();
        let text = std::fs::read_to_string(&json_path).unwrap();
        assert!(text.contains("会被备份的数据"));
        assert!(text.contains("schemaVersion"));
        let md_path = export(&conn, &dir.join("exports"), "md").unwrap();
        assert!(
            std::fs::read_to_string(&md_path)
                .unwrap()
                .contains("## 轨迹条目")
        );

        // 恢复前置流程可执行
        prepare_restore(&conn, &backups).unwrap();

        // 清理：造 32 条日志 → 保留 30
        for _ in 0..31 {
            conn.execute(
                "INSERT INTO backup_log(file, size_bytes, status) VALUES('fake.db', 1, 'ok')",
                [],
            )
            .unwrap();
        }
        prune(&conn, &backups).unwrap();
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM backup_log", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 30);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
