use std::env;

use crate::error::Error;

use super::error::Result;
use chrono::{DateTime, Local};
use turso::{Builder, Connection, Rows};

pub struct Db {
    conn: Connection,
}

impl Db {
    /// Create a new instance of the database
    pub async fn new() -> Result<Self> {
        let mut path = dirs::config_dir().ok_or(Error::Unknown {
            msg: "Failed to find the config path".to_string(),
        })?;
        path.push(env!("CARGO_PKG_NAME"));
        path.push("historic.db");

        let parent_path = &path.parent().unwrap_or(&path);
        if !tokio::fs::try_exists(parent_path).await? {
            tokio::fs::create_dir(parent_path).await?;
        };

        let path_str = path.to_str().ok_or(Error::Unknown {
            msg: "Failed to get the config path".to_string(),
        })?;

        let db = Builder::new_local(path_str).build().await?;
        let conn = db.connect()?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS ranks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    session_id TEXT NOT NULL,
    rank INTEGER NOT NULL,
    cmd TEXT NOT NULL
)",
            (),
        )
        .await?;

        Ok(Self { conn })
    }

    /// Execute a statement and return the affected row count.
    pub async fn execute<P>(&self, sql: &str, params: P) -> Result<u64>
    where
        P: turso::IntoParams,
    {
        let affected = self.conn.execute(sql, params).await?;
        Ok(affected)
    }

    /// Query and return the cursor without consuming rows.
    pub async fn query<P>(&self, sql: &str, params: P) -> Result<Rows>
    where
        P: turso::IntoParams,
    {
        let rows = self.conn.query(sql, params).await?;
        Ok(rows)
    }

    pub async fn get_commands(&self, session_id: &str) -> Result<Rows> {
        let rows = self
            .conn
            .query(
                "SELECT id, timestamp, session_id, rank, cmd FROM ranks WHERE session_id = ? ORDER BY rank ASC",
                (session_id,),
            )
            .await?;
        Ok(rows)
    }

    pub async fn rank_n_save_new(&self, session_id: String, new_cmd: String) -> Result<()> {
        let mut maybe_row = self
            .conn
            .query(
                "select id, timestamp, rank from ranks where session_id=? and cmd=?",
                (session_id.clone(), new_cmd.clone()),
            )
            .await?;

        if let Some(row) = maybe_row.next().await? {
            let id: i64 = row.get(0)?;
            let ts_str: String = row.get(1)?;
            let rank: i64 = row.get(2)?;

            let ts: DateTime<Local> = DateTime::parse_from_rfc3339(&ts_str)
                .map_err(|_| Error::Unknown {
                    msg: "Error converting the time".to_string(),
                })
                .map(|dt| dt.with_timezone(&Local))?;

            let new_rank = calculate_rank(rank, ts);

            self.conn
                .execute("update ranks set rank=? where id=?", (new_rank, id))
                .await?;
        } else {
            let max_rank = self
                .conn
                .query(
                    "SELECT MAX(rank) FROM ranks where session_id=?",
                    (session_id.clone(),),
                )
                .await?
                .next()
                .await?
                .iter()
                .map(|r| r.get(0).unwrap_or(1))
                .collect::<Vec<i64>>()[0];

            self.conn
                .execute(
                    "insert into ranks (timestamp,session_id,rank,cmd) values (?,?,?,?)",
                    (Local::now().to_rfc3339(), session_id, max_rank + 1, new_cmd),
                )
                .await?;
        }

        Ok(())
    }
}

pub fn calculate_rank(rank: i64, ts: DateTime<Local>) -> i64 {
    let age_hours = (Local::now() - ts).num_hours();
    let mut new_rank = rank;
    if age_hours < 1 {
        new_rank = rank.checked_mul(2).unwrap_or(i64::MAX).max(1)
    } else if age_hours < 24 {
        new_rank = rank;
    } else if age_hours < 24 * 7 {
        new_rank = rank.checked_div(2).unwrap_or(i64::MAX).max(1)
    } else {
        new_rank = rank.checked_div(4).unwrap_or(i64::MAX).max(1)
    }

    new_rank
}
