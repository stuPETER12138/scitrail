//! 速录语法解析（说明书 §4.5）：
//! `[类型前缀] 内容 #课题 *标签 !优先级 [日期] [时间]`
//! 解析失败不报错：整段作为标题，其余留空可后补。

use chrono::{Datelike, Duration, Local, NaiveDate};
use serde::Serialize;

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ParsedDraft {
    pub kind: String, // task / experiment / literature / expense / meeting / note
    pub title: String,
    pub project_name: Option<String>,
    pub project_matched: Option<i64>,
    pub tags: Vec<String>,
    pub priority: Option<i32>, // 0高 1中 2低（仅任务）
    pub date: Option<String>,  // YYYY-MM-DD
    pub time: Option<String>,  // HH:MM
}

const KIND_PREFIX: [(&str, &str); 6] = [
    ("实", "experiment"),
    ("文", "literature"),
    ("钱", "expense"),
    ("会", "meeting"),
    ("记", "note"),
    ("任", "task"),
];

pub fn parse(text: &str, default_kind: &str) -> ParsedDraft {
    let mut draft = ParsedDraft {
        kind: default_kind.to_string(),
        title: String::new(),
        project_name: None,
        project_matched: None,
        tags: Vec::new(),
        priority: None,
        date: None,
        time: None,
    };

    let today = Local::now().date_naive();
    let mut kind_set = false;

    for token in text.split_whitespace() {
        // 按特殊符号切分：完成#催化 → [完成, #, 催化]
        for part in split_special(token) {
            match part {
                Special::Hash(name) => {
                    if !name.is_empty() && draft.project_name.is_none() {
                        draft.project_name = Some(name.to_string());
                    } else if !name.is_empty() {
                        draft.tags.push(name.to_string());
                    }
                }
                Special::Star(name) => {
                    if !name.is_empty() {
                        draft.tags.push(name.to_string());
                    }
                }
                Special::Bang(name) => match name {
                    "高" => draft.priority = Some(0),
                    "中" => draft.priority = Some(1),
                    "低" => draft.priority = Some(2),
                    other => push_title(&mut draft.title, &format!("!{other}")),
                },
                Special::Text(t) => {
                    if !kind_set
                        && let Some((prefix, kind)) =
                            KIND_PREFIX.iter().find(|(p, _)| t.starts_with(p))
                    {
                        draft.kind = kind.to_string();
                        kind_set = true;
                        let rest = &t[prefix.len()..];
                        if !rest.is_empty() {
                            push_title(&mut draft.title, rest);
                        }
                        continue;
                    }
                    // 日期（可带粘连时间：明天9:00）
                    if let Some((d, rest)) = parse_date_prefix(t, today) {
                        draft.date = Some(d);
                        if !rest.is_empty() {
                            if let Some(tm) = parse_time(&rest) {
                                draft.time = Some(tm);
                            } else {
                                push_title(&mut draft.title, &rest);
                            }
                        }
                        continue;
                    }
                    if let Some(tm) = parse_time(t) {
                        draft.time = Some(tm);
                        continue;
                    }
                    push_title(&mut draft.title, t);
                }
            }
        }
    }

    if draft.title.is_empty() {
        // 全是特殊 token：把原文作为标题兜底（不虚构、不丢弃）
        draft.title = text.trim().to_string();
    }
    draft
}

fn push_title(title: &mut String, part: &str) {
    if !title.is_empty() {
        title.push(' ');
    }
    title.push_str(part);
}

enum Special<'a> {
    Hash(&'a str),
    Star(&'a str),
    Bang(&'a str),
    Text(&'a str),
}

/// 按特殊符号切段，全部借用自 token：
/// "完成#催化*加急" → [Text("完成"), Hash("催化"), Star("加急")]
fn split_special(token: &str) -> Vec<Special<'_>> {
    let bytes = token.as_bytes();
    let mut out: Vec<Special> = Vec::new();
    let mut text_start = 0usize;
    let mut have_text = false;
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'#' || b == b'*' || b == b'!' {
            if have_text {
                out.push(Special::Text(&token[text_start..i]));
                have_text = false;
            }
            let seg_start = i + 1;
            let mut j = seg_start;
            while j < bytes.len() && bytes[j] != b'#' && bytes[j] != b'*' && bytes[j] != b'!' {
                j += 1;
            }
            let seg = &token[seg_start..j];
            out.push(match b {
                b'#' => Special::Hash(seg),
                b'*' => Special::Star(seg),
                _ => Special::Bang(seg),
            });
            i = j;
        } else {
            if !have_text {
                text_start = i;
                have_text = true;
            }
            i += 1;
        }
    }
    if have_text {
        out.push(Special::Text(&token[text_start..]));
    }
    out.retain(|s| !matches!(s, Special::Text("")));
    out
}

/// 解析日期词；支持词尾粘连时间（"明天9:00" → 明天 + 9:00）
fn parse_date_prefix(s: &str, today: NaiveDate) -> Option<(String, String)> {
    let (days, rest) = if let Some(r) = s.strip_prefix("大后天") {
        (3, r)
    } else if let Some(r) = s.strip_prefix("后天") {
        (2, r)
    } else if let Some(r) = s.strip_prefix("明天") {
        (1, r)
    } else if let Some(r) = s.strip_prefix("今天") {
        (0, r)
    } else {
        return parse_weekday_or_numeric(s, today);
    };
    Some((format_date(today + Duration::days(days)), rest.to_string()))
}

fn parse_weekday_or_numeric(s: &str, today: NaiveDate) -> Option<(String, String)> {
    // 下周X / 周X / 星期X / 礼拜X
    for (prefix, next_week) in [
        ("下周", true),
        ("周", false),
        ("星期", false),
        ("礼拜", false),
    ] {
        if let Some(rest) = s.strip_prefix(prefix) {
            let (idx, remain) = chinese_weekday(rest)?;
            let mut delta = (idx - today.weekday().num_days_from_monday() as i64).rem_euclid(7);
            if delta == 0 || next_week {
                if next_week {
                    delta += 7;
                } else {
                    delta = 7;
                }
            }
            return Some((
                format_date(today + Duration::days(delta)),
                remain.to_string(),
            ));
        }
    }
    // 2027-03-01 / 3月1日 / 3月1 / 12-01
    let bytes = s.as_bytes();
    let mut i = 0;
    let num_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == num_start {
        return None;
    }
    let first: u32 = s[num_start..i].parse().ok()?;
    let rest = &s[i..];
    if let Some(r) = rest.strip_prefix('-') {
        // 12-01 或 2027-03-01
        let mut j = 0;
        while j < r.len() && r.as_bytes()[j].is_ascii_digit() {
            j += 1;
        }
        let second: u32 = r[..j].parse().ok()?;
        let tail = &r[j..];
        let date = if let Some(day_part) = tail.strip_prefix('-') {
            // 年-月-日
            let day: u32 = day_part.split('-').next()?.parse().ok()?;
            NaiveDate::from_ymd_opt(first as i32, second, day)?
        } else if first > 31 {
            // 年-月（不含日）：不解析，避免歧义
            return None;
        } else {
            let year = if (1..=12).contains(&first) && (1..=31).contains(&second) {
                // first=月 second=日：若今年该日已过则顺延明年
                let y = today.year();
                let candidate = NaiveDate::from_ymd_opt(y, first, second)?;
                if candidate >= today { y } else { y + 1 }
            } else {
                return None;
            };
            NaiveDate::from_ymd_opt(year, first, second)?
        };
        return Some((format_date(date), String::new()));
    }
    if let Some(r) = rest.strip_prefix('月') {
        let mut j = 0;
        while j < r.len() && r.as_bytes()[j].is_ascii_digit() {
            j += 1;
        }
        if j == 0 {
            return None;
        }
        let second: u32 = r[..j].parse().ok()?;
        let tail = &r[j..];
        if !(1..=12).contains(&first) || !(1..=31).contains(&second) {
            return None;
        }
        let y = today.year();
        let candidate = NaiveDate::from_ymd_opt(y, first, second)?;
        let year = if candidate >= today { y } else { y + 1 };
        let _ = tail;
        return Some((
            format_date(NaiveDate::from_ymd_opt(year, first, second)?),
            String::new(),
        ));
    }
    None
}

/// 中文星期：一~日/天 或 1~7，返回周一=0 的索引
fn chinese_weekday(s: &str) -> Option<(i64, &str)> {
    let table = [
        ("一", 0i64),
        ("二", 1),
        ("三", 2),
        ("四", 3),
        ("五", 4),
        ("六", 5),
        ("日", 6),
        ("天", 6),
    ];
    for (c, idx) in table {
        if let Some(rest) = s.strip_prefix(c) {
            return Some((idx, rest));
        }
    }
    let d = s.chars().next()?;
    if ('1'..='7').contains(&d) {
        return Some(((d as u8 - b'1') as i64, &s[1..]));
    }
    None
}

/// 时间：9:00 / 21:15 / 9点 / 9点30 / 9点30分 / 9点15 → HH:MM
pub fn parse_time(s: &str) -> Option<String> {
    let s = s.trim();
    if let Some((h, m)) = s.split_once(':') {
        let h: u32 = h.parse().ok()?;
        let m: u32 = m.parse().ok()?;
        if h < 24 && m < 60 {
            return Some(format!("{h:02}:{m:02}"));
        }
        return None;
    }
    if let Some(rest) = s.strip_suffix('分') {
        return parse_time(rest);
    }
    if let Some((h_str, rest)) = s.split_once('点') {
        let h: u32 = h_str.parse().ok()?;
        if h >= 24 {
            return None;
        }
        if rest.is_empty() {
            return Some(format!("{h:02}:00"));
        }
        let m: u32 = rest.parse().ok()?;
        if m < 60 {
            return Some(format!("{h:02}:{m:02}"));
        }
        return None;
    }
    None
}

fn format_date(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn today() -> NaiveDate {
        Local::now().date_naive()
    }

    #[test]
    fn task_with_all_tokens() {
        let d = parse("任 回复审稿意见 #论文A !高 明天", "note");
        assert_eq!(d.kind, "task");
        assert_eq!(d.title, "回复审稿意见");
        assert_eq!(d.project_name.as_deref(), Some("论文A"));
        assert_eq!(d.priority, Some(0));
        assert_eq!(
            d.date.as_deref(),
            Some(format_date(today() + Duration::days(1)).as_str())
        );
        assert_eq!(d.time, None);
    }

    #[test]
    fn experiment_with_attached_project_and_time() {
        let d = parse("实 完成#催化 1150℃×2h 明天 9:00", "task");
        assert_eq!(d.kind, "experiment");
        assert_eq!(d.title, "完成 1150℃×2h");
        assert_eq!(d.project_name.as_deref(), Some("催化"));
        assert_eq!(d.time.as_deref(), Some("09:00"));
    }

    #[test]
    fn note_with_tags() {
        let d = parse("记 灵感记录 *抓取 *想法", "task");
        assert_eq!(d.kind, "note");
        assert_eq!(d.title, "灵感记录");
        assert_eq!(d.tags, vec!["抓取", "想法"]);
    }

    #[test]
    fn bare_text_becomes_default_kind_title() {
        let d = parse("随便一句没有语法的话", "task");
        assert_eq!(d.kind, "task");
        assert_eq!(d.title, "随便一句没有语法的话");
        assert_eq!(d.date, None);
    }

    #[test]
    fn second_hash_becomes_tag() {
        let d = parse("任 测试 #催化 #论文A", "task");
        assert_eq!(d.project_name.as_deref(), Some("催化"));
        assert!(d.tags.contains(&"论文A".to_string()));
    }

    #[test]
    fn time_forms() {
        assert_eq!(parse_time("9:00").as_deref(), Some("09:00"));
        assert_eq!(parse_time("21:15").as_deref(), Some("21:15"));
        assert_eq!(parse_time("9点").as_deref(), Some("09:00"));
        assert_eq!(parse_time("9点30").as_deref(), Some("09:30"));
        assert_eq!(parse_time("9点30分").as_deref(), Some("09:30"));
        assert_eq!(parse_time("25:00"), None);
    }

    #[test]
    fn date_forms() {
        let t = today();
        assert_eq!(
            parse_date_prefix("3月1日", t).map(|(d, _)| d),
            Some(format_date(
                NaiveDate::from_ymd_opt(
                    if t <= NaiveDate::from_ymd_opt(t.year(), 3, 1).unwrap() {
                        t.year()
                    } else {
                        t.year() + 1
                    },
                    3,
                    1
                )
                .unwrap()
            ))
        );
        let iso = parse_date_prefix("2027-05-01", t).unwrap().0;
        assert_eq!(iso, "2027-05-01");
        // 周六：必然是今天或未来的某个周六
        let sat = parse_date_prefix("周六", t).unwrap().0;
        let sat_date = NaiveDate::parse_from_str(&sat, "%Y-%m-%d").unwrap();
        assert!(sat_date >= t);
        assert_eq!(sat_date.weekday().num_days_from_monday(), 5);
    }

    #[test]
    fn date_with_glued_time() {
        let (d, rest) = parse_date_prefix("明天9:00", today()).unwrap();
        assert!(!d.is_empty());
        assert_eq!(rest, "9:00");
    }
}
