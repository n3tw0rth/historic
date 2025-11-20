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

    pub async fn rank_n_save_new(&self, session_id: String, new_cmd: String) -> Result<()> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, timestamp, session_id, rank, cmd FROM ranks WHERE session_id = ?",
                (session_id.clone(),),
            )
            .await?;
        let mut vec = Vec::new();
        while let Some(row) = rows.next().await? {
            vec.push(row);
        }

        for row in vec {
            let id: i64 = row.get(0)?;
            let ts_str: String = row.get(1)?;
            let rank: i64 = row.get(3)?;

            let ts: DateTime<Local> = DateTime::parse_from_rfc3339(&ts_str)
                .map_err(|_| Error::Unknown {
                    msg: "Error converting the time".to_string(),
                })
                .map(|dt| dt.with_timezone(&Local))?;

            let age_secs = (Local::now() - ts).num_seconds();

            // NOTE: dummy rank calculation
            let new_rank = rank + (100 - age_secs.max(0)) as i64;

            self.conn
                .execute("UPDATE ranks set rank=? where id=?", (new_rank, id))
                .await?;
        }

        self.conn
            .execute(
                "insert into ranks (timestamp,session_id,rank,cmd) values (?,?,?,?)",
                (Local::now().to_rfc3339(), session_id, 0, new_cmd),
            )
            .await?;

        Ok(())
    }
}
