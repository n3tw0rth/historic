use super::error::Result;
use turso::{Builder, Connection, Rows};

pub struct Db {
    conn: Connection,
}

impl Db {
    pub async fn new() -> Result<Self> {
        let db = Builder::new_local("historic.db").build().await?;
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
