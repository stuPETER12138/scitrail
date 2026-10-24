use rusqlite::Row;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub r#type: String,
    pub role: Option<String>,
    pub status: String,
    pub color: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub note: Option<String>,
}

impl Project {
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            r#type: row.get(2)?,
            role: row.get(3)?,
            status: row.get(4)?,
            color: row.get(5)?,
            start_date: row.get(6)?,
            end_date: row.get(7)?,
            note: row.get(8)?,
        })
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub note: Option<String>,
    pub project_id: Option<i64>,
    pub project_name: Option<String>,
    pub milestone_id: Option<i64>,
    pub priority: i64,
    pub due_date: Option<String>,
    pub status: String,
    pub repeat_rule: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: Option<String>,
}

impl Task {
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Task {
            id: row.get(0)?,
            title: row.get(1)?,
            note: row.get(2)?,
            project_id: row.get(3)?,
            project_name: row.get(4)?,
            milestone_id: row.get(5)?,
            priority: row.get(6)?,
            due_date: row.get(7)?,
            status: row.get(8)?,
            repeat_rule: row.get(9)?,
            completed_at: row.get(10)?,
            created_at: row.get(11)?,
        })
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ddl {
    pub id: i64,
    pub name: String,
    pub category: Option<String>,
    pub due_date: String,
    pub lead_days: String,
    pub project_id: Option<i64>,
    pub status: String,
    pub repeat_yearly: bool,
    pub note: Option<String>,
}

impl Ddl {
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Ddl {
            id: row.get(0)?,
            name: row.get(1)?,
            category: row.get(2)?,
            due_date: row.get(3)?,
            lead_days: row.get(4)?,
            project_id: row.get(5)?,
            status: row.get(6)?,
            repeat_yearly: row.get::<_, i64>(7)? != 0,
            note: row.get(8)?,
        })
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Milestone {
    pub id: i64,
    pub name: String,
    pub target_date: Option<String>,
    pub status: String,
    pub done_at: Option<String>,
    pub note: Option<String>,
}

impl Milestone {
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Milestone {
            id: row.get(0)?,
            name: row.get(1)?,
            target_date: row.get(2)?,
            status: row.get(3)?,
            done_at: row.get(4)?,
            note: row.get(5)?,
        })
    }
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: i64,
    pub date: String,
    pub time: Option<String>,
    pub kind: String,
    pub title: String,
    pub content: String,
    pub project_id: Option<i64>,
    pub tags: Vec<String>,
    pub fields: serde_json::Value,
    pub refs: Vec<crate::services::entries::RefItem>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl Entry {
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let tags: String = row.get(7)?;
        let fields: String = row.get(8)?;
        let refs: String = row.get(9)?;
        Ok(Entry {
            id: row.get(0)?,
            date: row.get(1)?,
            time: row.get(2)?,
            kind: row.get(3)?,
            title: row.get(4)?,
            content: row.get(5)?,
            project_id: row.get(6)?,
            tags: serde_json::from_str(&tags).unwrap_or_default(),
            fields: serde_json::from_str(&fields).unwrap_or(serde_json::Value::Null),
            refs: serde_json::from_str(&refs).unwrap_or_default(),
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    }
}
