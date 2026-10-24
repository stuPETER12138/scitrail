//! 回顾统计（M2-1，说明书 §5 /docs 回顾页）：
//! KPI、连续记录天数、逐日计数（热力图+趋势共用）、课题投入分布、学期总结导出。
//! 全部为纯日期聚合，无新表。

use chrono::{Datelike, Duration, Local, NaiveDate};
use rusqlite::{Connection, params};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct DayCount {
    pub date: String,
    pub experiment: i64,
    pub literature: i64,
    pub expense: i64,
    pub meeting: i64,
    pub note: i64,
    pub tasks_done: i64,
}

impl DayCount {
    pub fn total(&self) -> i64 {
        self.experiment + self.literature + self.expense + self.meeting + self.note + self.tasks_done
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Kpis {
    pub total_entries: i64,
    pub total_tasks_done: i64,
    pub total_experiments: i64,
    pub current_streak: i64,
    pub longest_streak: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDist {
    pub project_id: Option<i64>,
    pub name: String,
    pub color: Option<String>,
    pub count: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewStats {
    pub from: String,
    pub to: String,
    pub kpis: Kpis,
    /// 连续填充的逐日计数（含 0 天），供热力图与趋势共用
    pub daily: Vec<DayCount>,
    /// 近 90 天课题投入分布（entry + 完成任务）
    pub distribution: Vec<ProjectDist>,
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

fn fmt(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

/// 任意"活跃日"集合：当天有 ≥1 条 entry 或完成 ≥1 个任务
fn activity_days(conn: &Connection) -> Result<Vec<String>, String> {
    let mut set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut stmt = conn
        .prepare("SELECT DISTINCT date FROM entry WHERE deleted_at IS NULL")
        .map_err(|e| e.to_string())?;
    let rows: Vec<String> = stmt
        .query_map([], |r| r.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    set.extend(rows);
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT substr(completed_at,1,10) FROM task
             WHERE deleted_at IS NULL AND status='完成' AND completed_at IS NOT NULL",
        )
        .map_err(|e| e.to_string())?;
    let rows: Vec<String> = stmt
        .query_map([], |r| r.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    set.extend(rows);
    Ok(set.into_iter().collect())
}

fn streaks(days: &[String], today: NaiveDate) -> (i64, i64) {
    let set: std::collections::BTreeSet<&str> = days.iter().map(|s| s.as_str()).collect();
    // 当前连续：今天没记不打断，从昨天起算
    let mut cursor = if set.contains(fmt(today).as_str()) {
        today
    } else {
        today - Duration::days(1)
    };
    let mut current = 0i64;
    while set.contains(fmt(cursor).as_str()) {
        current += 1;
        cursor -= Duration::days(1);
    }
    // 最长连续：扫描全部活跃日
    let mut longest = 0i64;
    let mut run = 0i64;
    let mut prev: Option<NaiveDate> = None;
    for d in &set {
        let Some(cur) = parse_date(d) else { continue };
        run = match prev {
            Some(p) if (cur - p).num_days() == 1 => run + 1,
            _ => 1,
        };
        longest = longest.max(run);
        prev = Some(cur);
    }
    (current, longest)
}

pub fn review(conn: &Connection, from: &str, to: &str) -> Result<ReviewStats, String> {
    let from_d = parse_date(from).ok_or("from 日期格式错误")?;
    let to_d = parse_date(to).ok_or("to 日期格式错误")?;

    // 逐日计数（entry 按类型）
    let mut map: BTreeMap<String, DayCount> = BTreeMap::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT date, kind, COUNT(*) FROM entry
                 WHERE deleted_at IS NULL AND date BETWEEN ?1 AND ?2
                 GROUP BY date, kind",
            )
            .map_err(|e| e.to_string())?;
        let rows: Vec<(String, String, i64)> = stmt
            .query_map(params![from, to], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        for (date, kind, n) in rows {
            let dc = map.entry(date).or_default();
            match kind.as_str() {
                "experiment" => dc.experiment = n,
                "literature" => dc.literature = n,
                "expense" => dc.expense = n,
                "meeting" => dc.meeting = n,
                _ => dc.note = n,
            }
        }
    }
    // 完成任务计数
    {
        let mut stmt = conn
            .prepare(
                "SELECT substr(completed_at,1,10), COUNT(*) FROM task
                 WHERE deleted_at IS NULL AND status='完成' AND completed_at IS NOT NULL
                   AND substr(completed_at,1,10) BETWEEN ?1 AND ?2
                 GROUP BY substr(completed_at,1,10)",
            )
            .map_err(|e| e.to_string())?;
        let rows: Vec<(String, i64)> = stmt
            .query_map(params![from, to], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        for (date, n) in rows {
            map.entry(date).or_default().tasks_done = n;
        }
    }

    // 连续填充区间每一天
    let mut daily = Vec::new();
    let mut d = from_d;
    while d <= to_d {
        let date = fmt(d);
        let dc = map.remove(&date).unwrap_or_default();
        daily.push(DayCount { date, ..dc });
        d += Duration::days(1);
    }

    // KPI（全生命周期口径）
    let count = |sql: &str| -> Result<i64, String> {
        conn.query_row(sql, [], |r| r.get(0)).map_err(|e| e.to_string())
    };
    let total_entries = count("SELECT COUNT(*) FROM entry WHERE deleted_at IS NULL")?;
    let total_tasks_done =
        count("SELECT COUNT(*) FROM task WHERE deleted_at IS NULL AND status='完成'")?;
    let total_experiments =
        count("SELECT COUNT(*) FROM entry WHERE deleted_at IS NULL AND kind='experiment'")?;
    let (current_streak, longest_streak) = streaks(&activity_days(conn)?, Local::now().date_naive());

    // 课题投入分布（近 90 天：entry + 完成任务）
    let d90 = fmt(Local::now().date_naive() - Duration::days(89));
    let mut dist: BTreeMap<Option<i64>, i64> = BTreeMap::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT project_id, COUNT(*) FROM entry
                 WHERE deleted_at IS NULL AND date >= ?1 GROUP BY project_id",
            )
            .map_err(|e| e.to_string())?;
        let rows: Vec<(Option<i64>, i64)> = stmt
            .query_map(params![d90], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        for (pid, n) in rows {
            *dist.entry(pid).or_insert(0) += n;
        }
        let mut stmt = conn
            .prepare(
                "SELECT project_id, COUNT(*) FROM task
                 WHERE deleted_at IS NULL AND status='完成' AND completed_at IS NOT NULL
                   AND substr(completed_at,1,10) >= ?1
                 GROUP BY project_id",
            )
            .map_err(|e| e.to_string())?;
        let rows: Vec<(Option<i64>, i64)> = stmt
            .query_map(params![d90], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        for (pid, n) in rows {
            *dist.entry(pid).or_insert(0) += n;
        }
    }
    let mut distribution: Vec<ProjectDist> = Vec::new();
    for (pid, n) in dist {
        if n == 0 {
            continue;
        }
        let (name, color) = match pid {
            Some(id) => conn
                .query_row(
                    "SELECT name, color FROM project WHERE id = ?1 AND deleted_at IS NULL",
                    params![id],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?)),
                )
                .unwrap_or(("（已删除课题）".into(), None)),
            None => ("未关联".to_string(), None),
        };
        distribution.push(ProjectDist { project_id: pid, name, color, count: n });
    }
    distribution.sort_by(|a, b| b.count.cmp(&a.count));

    Ok(ReviewStats {
        from: from.to_string(),
        to: to.to_string(),
        kpis: Kpis {
            total_entries,
            total_tasks_done,
            total_experiments,
            current_streak,
            longest_streak,
        },
        daily,
        distribution,
    })
}

// ---------- 学期总结 ----------

/// 学期起点：2~8 月 → 当年 3 月 1 日；9~1 月 → 上一年 9 月 1 日
fn semester_start(today: NaiveDate) -> NaiveDate {
    let y = today.year();
    let m = today.month();
    if (9..=12).contains(&m) {
        NaiveDate::from_ymd_opt(y, 9, 1).unwrap()
    } else if m == 1 {
        NaiveDate::from_ymd_opt(y - 1, 9, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(y, 3, 1).unwrap()
    }
}

pub fn semester_markdown(conn: &Connection) -> Result<(String, String), String> {
    let today = Local::now().date_naive();
    let start = semester_start(today);
    let (start_s, today_s) = (fmt(start), fmt(today));

    let stats = review(conn, &start_s, &today_s)?;
    let total: i64 = stats.daily.iter().map(|d| d.total()).sum();
    let active_days = stats.daily.iter().filter(|d| d.total() > 0).count();

    let mut md = format!("# 科迹 · 学期总结（{start_s} ~ {today_s}）\n\n");
    md.push_str(&format!(
        "- 记录条目：{} 条 · 完成任务：{} 个（其中实验 {} 次）\n- 活跃天数：{} 天 · 当前连续：{} 天 · 最长连续：{} 天\n\n",
        stats.kpis.total_entries, stats.kpis.total_tasks_done, stats.kpis.total_experiments,
        active_days, stats.kpis.current_streak, stats.kpis.longest_streak,
    ));
    let _ = total;

    // 按月分组列任务与条目
    let mut month = String::new();
    let mut stmt = conn
        .prepare(
            "SELECT substr(t.completed_at,1,10), t.title, p.name FROM task t
             LEFT JOIN project p ON p.id = t.project_id
             WHERE t.deleted_at IS NULL AND t.status='完成' AND t.completed_at IS NOT NULL
               AND substr(t.completed_at,1,10) BETWEEN ?1 AND ?2
             ORDER BY t.completed_at",
        )
        .map_err(|e| e.to_string())?;
    let done: Vec<(String, String, Option<String>)> = stmt
        .query_map(params![start_s, today_s], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    for (date, title, project) in &done {
        let m = &date[..7];
        if m != month {
            month = m.to_string();
            md.push_str(&format!("\n## {m}\n"));
        }
        match project {
            Some(p) => md.push_str(&format!("- [x] {title}（{p}）\n")),
            None => md.push_str(&format!("- [x] {title}\n")),
        }
    }

    md.push_str("\n## 轨迹条目（按类型汇总）\n");
    let (mut exp, mut lit, mut expn, mut mtg, mut note) = (0, 0, 0, 0, 0);
    for d in &stats.daily {
        exp += d.experiment;
        lit += d.literature;
        expn += d.expense;
        mtg += d.meeting;
        note += d.note;
    }
    md.push_str(&format!(
        "- 🧪 实验 {exp} · 📖 文献 {lit} · 💰 报销 {expn} · 📅 组会 {mtg} · 📝 笔记 {note}\n"
    ));

    if let Some(top) = stats.distribution.first() {
        md.push_str(&format!("\n投入最多：{}（{} 条记录）\n", top.name, top.count));
    }
    Ok((format!("学期总结-{start_s}~{today_s}.md"), md))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_conn;
    use crate::services::entries::{self, EntryInput};
    use chrono::Duration as Dur;

    fn entry_at(date: &str, kind: &str) {
        entries::create(
            &test_conn_placeholder(),
            EntryInput {
                date: Some(date.into()),
                time: None,
                kind: kind.into(),
                title: "t".into(),
                content: None,
                project_id: None,
                tags: None,
                fields: None,
                refs: None,
            },
        )
        .unwrap();
    }
    fn test_conn_placeholder() -> Connection {
        unreachable!("占位，实际用下面的共享连接方式")
    }

    #[test]
    fn streaks_logic() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 24).unwrap();
        // 今天没记 → 从昨天起算；昨天与前天有 → current=2；更早 3 天连续 → longest=3
        let days = vec![
            "2026-10-18".to_string(),
            "2026-10-19".to_string(),
            "2026-10-20".to_string(),
            "2026-10-22".to_string(),
            "2026-10-23".to_string(),
        ];
        let (cur, long) = streaks(&days, today);
        assert_eq!((cur, long), (2, 3));
    }

    #[test]
    fn review_aggregates_and_fills() {
        let conn = test_conn();
        let today = Local::now().date_naive();
        let mk = |d: NaiveDate, kind: &'static str| {
            entries::create(
                &conn,
                EntryInput {
                    date: Some(fmt(d)),
                    time: None,
                    kind: kind.into(),
                    title: "标题".into(),
                    content: None,
                    project_id: None,
                    tags: None,
                    fields: None,
                    refs: None,
                },
            )
            .unwrap();
        };
        mk(today, "experiment");
        mk(today, "note");
        mk(today - Dur::days(1), "experiment");
        conn.execute(
            "INSERT INTO task(title, status, completed_at) VALUES('完成任务','完成', ?1)",
            params![format!("{} 10:00", fmt(today - Dur::days(2)))],
        )
        .unwrap();

        let r = review(&conn, &fmt(today - Dur::days(6)), &fmt(today)).unwrap();
        assert_eq!(r.daily.len(), 7);
        let tod = r.daily.last().unwrap();
        assert_eq!(tod.experiment, 1);
        assert_eq!(tod.note, 1);
        assert_eq!(r.daily[4].tasks_done, 1); // 3 天前
        assert_eq!(r.kpis.total_entries, 3);
        assert_eq!(r.kpis.total_experiments, 2);
        assert_eq!(r.kpis.total_tasks_done, 1);
        // 当前连续 3 天（今天/昨天/前天都有活动）
        assert_eq!(r.kpis.current_streak, 3);
        assert_eq!(r.kpis.longest_streak, 3);
        // 分布：未关联 4 条（3 entry + 1 task）
        assert_eq!(r.distribution.len(), 1);
        assert_eq!(r.distribution[0].name, "未关联");
        assert_eq!(r.distribution[0].count, 3); // 分布窗口 90 天含全部，entry=3

        // 学期总结生成
        let (name, md) = semester_markdown(&conn).unwrap();
        assert!(name.starts_with("学期总结-"));
        assert!(md.contains("完成任务"));
        assert!(md.contains("轨迹条目（按类型汇总）"));
    }

    #[test]
    fn semester_start_boundaries() {
        let s = semester_start(NaiveDate::from_ymd_opt(2026, 10, 24).unwrap());
        assert_eq!(fmt(s), "2026-09-01");
        let s2 = semester_start(NaiveDate::from_ymd_opt(2027, 3, 15).unwrap());
        assert_eq!(fmt(s2), "2027-03-01");
        let s3 = semester_start(NaiveDate::from_ymd_opt(2027, 1, 10).unwrap());
        assert_eq!(fmt(s3), "2026-09-01");
    }
}
