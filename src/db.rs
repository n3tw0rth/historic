use std::env;

use crate::error::Error;

use super::error::Result;
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

        if !tokio::fs::try_exists(&path).await? {
            tokio::fs::create_dir(&path.parent().unwrap_or(&path)).await?;
        };

        let path_str = path.to_str().ok_or(Error::Unknown {
            msg: "Failed to get the config path".to_string(),
        })?;

        let db = Builder::new_local(path_str).build().await?;
        let conn = db.connect()?;
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
}
