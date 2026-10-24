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
