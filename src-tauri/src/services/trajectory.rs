//! 轨迹投影（说明书 §4.4）：entry ∪ 任务完成 ∪ DDL 提醒 的 UNION 查询投影。
//! 轨迹是投影不是副本——数据各归各表，这里只负责按时序合并渲染。

use crate::models::Entry;
use chrono::{Duration, Local};
use rusqlite::{Connection, params};
use serde::Serialize;

use super::tasks::today_str;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskLite {
    pub id: i64,
    pub title: String,
    pub project_id: Option<i64>,
    pub priority: i64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DdlLite {
    pub id: i64,
    pub name: String,
    pub category: Option<String>,
    pub due_date: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TrajectoryItem {
    pub src: String, // entry | task_done | ddl
    pub at: String,  // 'HH:MM'；空 = 未记时刻（不虚构时间）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry: Option<Entry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<TaskLite>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ddl: Option<DdlLite>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrajectoryDay {
    pub date: String,
    pub items: Vec<TrajectoryItem>,
}

const ENTRY_COLUMNS: &str =
    "id, date, time, kind, title, content, project_id, tags, fields, refs, created_at, updated_at";

/// DDL 提醒提前量：生效 DDL 距今天 ≤ 该天数即在「今天」分组投出提醒行
const DDL_LOOKAHEAD_DAYS: i64 = 14;

pub fn get(conn: &Connection, from: &str, to: &str) -> Result<Vec<TrajectoryDay>, String> {
    use std::collections::BTreeMap;
    let mut days: BTreeMap<String, Vec<TrajectoryItem>> = BTreeMap::new();

    // 条目
    {
        let mut stmt = conn
            .prepare(&format!(
                "SELECT {ENTRY_COLUMNS} FROM entry
                 WHERE deleted_at IS NULL AND date BETWEEN ?1 AND ?2
                 ORDER BY date, (time IS NULL), time"
            ))
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![from, to], Entry::from_row)
            .map_err(|e| e.to_string())?;
        for row in rows {
            let e = row.map_err(|e| e.to_string())?;
            days.entry(e.date.clone())
                .or_default()
                .push(TrajectoryItem {
                    src: "entry".into(),
                    at: e.time.clone().unwrap_or_default(),
                    entry: Some(e),
                    task: None,
                    ddl: None,
                });
        }
    }

    // 任务完成事件
    {
        let mut stmt = conn
            .prepare(
                "SELECT id, title, project_id, priority, substr(completed_at,1,10) AS d,
                        substr(completed_at,12,5) AS t
                 FROM task
                 WHERE deleted_at IS NULL AND status='完成' AND completed_at IS NOT NULL
                   AND substr(completed_at,1,10) BETWEEN ?1 AND ?2",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![from, to], |r| {
                Ok((
                    TaskLite {
                        id: r.get(0)?,
                        title: r.get(1)?,
                        project_id: r.get(2)?,
                        priority: r.get(3)?,
                    },
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5).unwrap_or_default(),
                ))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (t, date, at) = row.map_err(|e| e.to_string())?;
            days.entry(date).or_default().push(TrajectoryItem {
                src: "task_done".into(),
                at,
                entry: None,
                task: Some(t),
                ddl: None,
            });
        }
    }

    // DDL 提醒：投到「今天」分组（仅当今天在查询区间内）
    let today = today_str();
    if from <= today.as_str() && today.as_str() <= to {
        let horizon = (Local::now().date_naive() + Duration::days(DDL_LOOKAHEAD_DAYS))
            .format("%Y-%m-%d")
            .to_string();
        let mut stmt = conn
            .prepare(
                "SELECT id, name, category, due_date FROM ddl
                 WHERE deleted_at IS NULL AND status='生效' AND substr(due_date,1,10) <= ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![horizon], |r| {
                Ok(DdlLite {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    category: r.get(2)?,
                    due_date: r.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let d = row.map_err(|e| e.to_string())?;
            days.entry(today.clone()).or_default().push(TrajectoryItem {
                src: "ddl".into(),
                at: String::new(),
                entry: None,
                task: None,
                ddl: Some(d),
            });
        }
    }

    // 组内排序：DDL 提醒置顶，其余按时刻升序、无时刻垫底
    Ok(days
        .into_iter()
        .map(|(date, mut items)| {
            items.sort_by(|a, b| {
                // DDL 提醒置顶；其余有时刻在前、时刻升序；无时刻垫底
                let ka = (a.at.is_empty(), a.at.clone());
                let kb = (b.at.is_empty(), b.at.clone());
                let sa = if a.src == "ddl" { 0 } else { 1 };
                let sb = if b.src == "ddl" { 0 } else { 1 };
                sa.cmp(&sb).then(ka.cmp(&kb))
            });
            TrajectoryDay { date, items }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_conn;
    use crate::services::entries::{self, EntryInput};

    fn input(title: &str, time: Option<&str>) -> EntryInput {
        EntryInput {
            date: None,
            time: time.map(|s| s.to_string()),
            kind: "experiment".into(),
            title: title.into(),
            content: None,
            project_id: None,
            tags: None,
            fields: None,
            refs: None,
        }
    }

    #[test]
    fn projection_merges_three_sources() {
        let conn = test_conn();
        let today = today_str();

        // 条目（带时刻 + 无时刻）
        entries::create(&conn, input("带时刻的实验", Some("09:12"))).unwrap();
        entries::create(&conn, input("无时刻的补录", None)).unwrap();

        // 任务完成事件
        conn.execute(
            "INSERT INTO task(title, status, completed_at) VALUES('完成任务A','完成', ?1)",
            params![format!("{today} 11:40")],
        )
        .unwrap();

        // DDL 提醒（5 天后 → 投到今天组顶部）
        conn.execute(
            "INSERT INTO ddl(name, due_date) VALUES('ACSAMI 截稿', ?1)",
            params![format!(
                "{}",
                (Local::now().date_naive() + Duration::days(5)).format("%Y-%m-%d")
            )],
        )
        .unwrap();

        let days = get(&conn, &today, &today).unwrap();
        assert_eq!(days.len(), 1);
        let items = &days[0].items;
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].src, "ddl"); // DDL 置顶
        assert_eq!(items[0].ddl.as_ref().unwrap().name, "ACSAMI 截稿");
        assert_eq!(items[1].at, "09:12"); // 有时刻在前
        assert_eq!(items[1].src, "entry");
        assert_eq!(items[2].src, "task_done");
        assert_eq!(items[2].at, "11:40");
        assert_eq!(items[3].at, ""); // 无时刻垫底
    }

    #[test]
    fn range_and_soft_delete_respected() {
        let conn = test_conn();
        let today = today_str();
        let yesterday = (Local::now().date_naive() - Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();
        let old = (Local::now().date_naive() - Duration::days(10))
            .format("%Y-%m-%d")
            .to_string();

        let mut e = input("昨天的", None);
        e.date = Some(yesterday.clone());
        let id_y = entries::create(&conn, e).unwrap();
        let mut e2 = input("十天前的", None);
        e2.date = Some(old);
        entries::create(&conn, e2).unwrap();

        // 区间外不入账本
        let days = get(&conn, &yesterday, &today).unwrap();
        assert_eq!(days.len(), 1);
        assert_eq!(days[0].date, yesterday);

        // 软删除后消失
        entries::delete(&conn, id_y).unwrap();
        let days2 = get(&conn, &yesterday, &today).unwrap();
        assert!(days2.is_empty());
    }
}
