//! 报销垫付台账（说明书 §4.10）：expense 条目的聚合视图，零新表。
//! 垫付中 → 已提交 → 已到账 状态流转；汇总"当前垫付未回款合计"。

use crate::services::entries::{self, EntryInput};
use rusqlite::Connection;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseItem {
    pub id: i64,
    pub date: String,
    pub title: String,
    pub amount: f64,
    pub reason: Option<String>,
    pub status: String,
    pub project_id: Option<i64>,
    pub project_name: Option<String>,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseSummary {
    pub advanced: f64,  // 垫付中合计（未回款）
    pub submitted: f64, // 已提交
    pub received: f64,  // 已到账
    pub total: f64,
    pub advanced_count: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectExpense {
    pub project_id: Option<i64>,
    pub name: String,
    pub color: Option<String>,
    pub total: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseLedger {
    pub items: Vec<ExpenseItem>,
    pub summary: ExpenseSummary,
    pub by_project: Vec<ProjectExpense>,
}

/// "¥380" / "380元" / " 1,200.5 " → f64；解析失败返回 0
fn parse_amount(raw: &str) -> f64 {
    let mut num = String::new();
    let mut dot_seen = false;
    for ch in raw.chars() {
        if ch.is_ascii_digit() {
            num.push(ch);
        } else if ch == '.' && !dot_seen && !num.is_empty() {
            dot_seen = true;
            num.push(ch);
        }
    }
    num.parse().unwrap_or(0.0)
}

/// 从标题里兜底提取金额（速录 `钱 移相器 380` 未填 fields.amount 时）
fn amount_from_title(title: &str) -> f64 {
    for token in title.split_whitespace() {
        let t = token.trim_start_matches(['¥', '￥']);
        if t.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            let v = parse_amount(t);
            if v > 0.0 {
                return v;
            }
        }
    }
    0.0
}

pub fn ledger(conn: &Connection) -> Result<ExpenseLedger, String> {
    let mut stmt = conn
        .prepare(
            "SELECT e.id, e.date, e.title, e.fields, e.project_id, p.name, p.color
             FROM entry e
             LEFT JOIN project p ON p.id = e.project_id AND p.deleted_at IS NULL
             WHERE e.deleted_at IS NULL AND e.kind = 'expense'
             ORDER BY e.date DESC, e.id DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<i64>>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, Option<String>>(6)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    let mut summary = ExpenseSummary::default();
    let mut projects: std::collections::BTreeMap<Option<i64>, (String, Option<String>, f64)> =
        std::collections::BTreeMap::new();

    for row in rows {
        let (id, date, title, fields_json, project_id, project_name, project_color) =
            row.map_err(|e| e.to_string())?;
        let fields: Value = serde_json::from_str(&fields_json).unwrap_or(Value::Null);
        let amount = match fields.get("amount").and_then(|v| v.as_str()) {
            Some(s) => parse_amount(s),
            None => fields
                .get("amount")
                .and_then(|v| v.as_f64())
                .unwrap_or_else(|| amount_from_title(&title)),
        };
        let status = fields
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("垫付中")
            .to_string();
        let reason = fields
            .get("reason")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        match status.as_str() {
            "已提交" => summary.submitted += amount,
            "已到账" => summary.received += amount,
            _ => {
                summary.advanced += amount;
                summary.advanced_count += 1;
            }
        }
        summary.total += amount;
        let entry = projects.entry(project_id).or_insert((
            project_name.clone().unwrap_or_else(|| "未关联".into()),
            project_color.clone(),
            0.0,
        ));
        entry.2 += amount;

        items.push(ExpenseItem {
            id,
            date,
            title,
            amount,
            reason,
            status,
            project_id,
            project_name,
        });
    }

    let mut by_project: Vec<ProjectExpense> = projects
        .into_iter()
        .map(|(project_id, (name, color, total))| ProjectExpense {
            project_id,
            name,
            color,
            total,
        })
        .collect();
    by_project.sort_by(|a, b| {
        b.total
            .partial_cmp(&a.total)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(ExpenseLedger {
        items,
        summary,
        by_project,
    })
}

/// 状态流转（垫付中/已提交/已到账）；走 entries::update 保留历史快照语义
pub fn set_status(conn: &Connection, id: i64, status: &str) -> Result<(), String> {
    if !matches!(status, "垫付中" | "已提交" | "已到账") {
        return Err(format!("未知报销状态：{status}"));
    }
    let e = entries::get(conn, id)?;
    if e.kind != "expense" {
        return Err("该条目不是报销记录".into());
    }
    let mut fields = match e.fields {
        Value::Object(map) => serde_json::Map::from_iter(map),
        _ => serde_json::Map::new(),
    };
    fields.insert("status".into(), Value::String(status.to_string()));
    let input = EntryInput {
        date: Some(e.date),
        time: e.time,
        kind: e.kind,
        title: e.title,
        content: Some(e.content),
        project_id: e.project_id,
        tags: Some(e.tags),
        fields: Some(Value::Object(fields)),
        refs: Some(e.refs),
    };
    entries::update(conn, id, input)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_conn;
    use rusqlite::params;

    #[test]
    fn amount_parsing_variants() {
        assert_eq!(parse_amount("¥380"), 380.0);
        assert_eq!(parse_amount("1,200.5 元"), 1200.5);
        assert_eq!(parse_amount("abc"), 0.0);
        assert_eq!(amount_from_title("移相器 380 #催化"), 380.0);
        assert_eq!(amount_from_title("没有数字"), 0.0);
    }

    #[test]
    fn ledger_aggregates_and_status_flow() {
        let conn = test_conn();
        let mk = |date: &str, title: &str, fields: Value| {
            entries::create(
                &conn,
                EntryInput {
                    date: Some(date.into()),
                    time: None,
                    kind: "expense".into(),
                    title: title.into(),
                    content: None,
                    project_id: None,
                    tags: None,
                    fields: Some(fields),
                    refs: None,
                },
            )
            .unwrap()
        };
        let a = mk(
            "2026-10-20",
            "移相器",
            serde_json::json!({"amount": "380", "reason": "京东采购", "status": "垫付中"}),
        );
        let b = mk(
            "2026-10-21",
            "耗材一批",
            serde_json::json!({"amount": "¥1,200.5", "status": "已提交"}),
        );
        // 速录兜底：无 fields.amount，从标题取
        mk("2026-10-22", "打车 85", serde_json::json!({}));

        let led = ledger(&conn).unwrap();
        assert_eq!(led.items.len(), 3);
        // 垫付中 = 380 + 85（无状态默认垫付）；已提交 = 1200.5
        assert!(
            (led.summary.advanced - 465.0).abs() < 1e-6,
            "{}",
            led.summary.advanced
        );
        assert!((led.summary.submitted - 1200.5).abs() < 1e-6);
        assert_eq!(led.summary.advanced_count, 2);
        assert!((led.summary.total - 1665.5).abs() < 1e-6);
        assert_eq!(led.by_project.len(), 1); // 未关联
        assert!((led.by_project[0].total - 1665.5).abs() < 1e-6);

        // 状态流转：a → 已到账，垫付归零，且写入历史快照
        set_status(&conn, a, "已到账").unwrap();
        let led2 = ledger(&conn).unwrap();
        assert!((led2.summary.advanced - 85.0).abs() < 1e-6);
        assert!((led2.summary.received - 380.0).abs() < 1e-6);
        let hist: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM entry_history WHERE entry_id = ?1",
                params![a],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(hist, 1);
        // 非报销条目拒绝
        let nid = entries::create(
            &conn,
            EntryInput {
                date: None,
                time: None,
                kind: "note".into(),
                title: "普通笔记".into(),
                content: None,
                project_id: None,
                tags: None,
                fields: None,
                refs: None,
            },
        )
        .unwrap();
        let _ = b;
        assert!(set_status(&conn, nid, "已到账").is_err());
    }
}
