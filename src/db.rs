use std::str::FromStr;
use std::time::Duration;

use crate::error::Error;

use super::error::Result;
use chrono::{DateTime, Local};
use sqlx::Row;
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteSynchronous,
};

/// Number of times to retry opening the database when it is transiently locked
/// (e.g. many instances performing the first WAL switch simultaneously).
const MAX_INIT_RETRIES: u64 = 10;

pub struct Db {
    pool: SqlitePool,
}

/// Whether an error is a transient SQLite lock/busy condition worth retrying.
fn is_locked(err: &sqlx::Error) -> bool {
    matches!(
        err,
        sqlx::Error::Database(db)
            if matches!(
                db.code().as_deref(),
                // SQLITE_BUSY, SQLITE_LOCKED, SQLITE_BUSY_RECOVERY, SQLITE_BUSY_SNAPSHOT
                Some("5") | Some("6") | Some("261") | Some("517")
            )
    )
}

impl Db {
    /// Create a new instance of the database.
    ///
    /// The connection pool is opened in WAL journal mode with a busy timeout so
    /// multiple `historic` instances (e.g. across tmux/zellij panes) can share
    /// the same database file concurrently: WAL allows readers to run alongside
    /// a writer, and the busy timeout serialises competing writers instead of
    /// failing immediately.
    pub async fn new() -> Result<Self> {
        let mut path = dirs::config_dir().ok_or(Error::Unknown {
            msg: "Failed to find the config path".to_string(),
        })?;
        path.push(env!("CARGO_PKG_NAME"));
        path.push("historic.db");

        let parent_path = &path.parent().unwrap_or(&path);
        if !tokio::fs::try_exists(parent_path).await? {
            tokio::fs::create_dir_all(parent_path).await?;
        };

        let path_str = path.to_str().ok_or(Error::Unknown {
            msg: "Failed to get the config path".to_string(),
        })?;

        let options = SqliteConnectOptions::from_str(&format!("sqlite://{path_str}"))?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .busy_timeout(Duration::from_secs(5));

        // Opening the pool switches the journal to WAL and creates the schema,
        // both of which briefly need a write lock. The busy timeout covers most
        // contention, but the very first WAL switch on a fresh file (many
        // instances starting at once) can still return SQLITE_BUSY immediately.
        // Retry a handful of times so a cold concurrent first-run is safe.
        let mut last_err = None;
        for attempt in 0..MAX_INIT_RETRIES {
            match Self::init(options.clone()).await {
                Ok(db) => return Ok(db),
                Err(Error::Db(err)) if is_locked(&err) => {
                    tokio::time::sleep(Duration::from_millis(50 * (attempt + 1))).await;
                    last_err = Some(Error::Db(err));
                }
                Err(err) => return Err(err),
            }
        }

        Err(last_err.unwrap_or(Error::Unknown {
            msg: "Failed to open the database".to_string(),
        }))
    }

    /// Open the connection pool and ensure the schema exists.
    async fn init(options: SqliteConnectOptions) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS ranks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    session_id TEXT NOT NULL,
    rank INTEGER NOT NULL,
    cmd TEXT NOT NULL
)",
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    /// Return the commands stored for a session ordered by ascending rank.
    pub async fn get_commands(&self, session_id: &str) -> Result<Vec<String>> {
        let rows = sqlx::query(
            "SELECT cmd FROM ranks WHERE session_id = ? ORDER BY rank ASC",
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;

        let commands = rows
            .into_iter()
            .map(|row| row.get::<String, _>("cmd"))
            .collect();

        Ok(commands)
    }

    pub async fn rank_n_save_new(&self, session_id: String, new_cmd: String) -> Result<()> {
        let maybe_row = sqlx::query(
            "select id, timestamp, rank from ranks where session_id=? and cmd=?",
        )
        .bind(&session_id)
        .bind(&new_cmd)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = maybe_row {
            let id: i64 = row.get("id");
            let ts_str: String = row.get("timestamp");
            let rank: i64 = row.get("rank");

            let ts: DateTime<Local> = DateTime::parse_from_rfc3339(&ts_str)
                .map_err(|_| Error::Unknown {
                    msg: "Error converting the time".to_string(),
                })
                .map(|dt| dt.with_timezone(&Local))?;

            let new_rank = calculate_rank(rank, ts);

            sqlx::query("update ranks set rank=? where id=?")
                .bind(new_rank)
                .bind(id)
                .execute(&self.pool)
                .await?;
        } else {
            let max_rank: i64 =
                sqlx::query_scalar("SELECT COALESCE(MAX(rank), 0) FROM ranks where session_id=?")
                    .bind(&session_id)
                    .fetch_one(&self.pool)
                    .await?;

            sqlx::query("insert into ranks (timestamp,session_id,rank,cmd) values (?,?,?,?)")
                .bind(Local::now().to_rfc3339())
                .bind(&session_id)
                .bind(max_rank + 1)
                .bind(&new_cmd)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }
}

pub fn calculate_rank(rank: i64, ts: DateTime<Local>) -> i64 {
    let age_hours = (Local::now() - ts).num_hours();
    if age_hours < 1 {
        rank.checked_mul(2).unwrap_or(i64::MAX).max(1)
    } else if age_hours < 24 {
        rank
    } else if age_hours < 24 * 7 {
        rank.checked_div(2).unwrap_or(i64::MAX).max(1)
    } else {
        rank.checked_div(4).unwrap_or(i64::MAX).max(1)
    }
}
