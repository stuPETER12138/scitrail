use rusqlite_migration::{M, Migrations};

const V1: &str = include_str!("../migrations/v1.sql");
const V2: &str = include_str!("../migrations/v2.sql");

pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![M::up(V1), M::up(V2)])
}
