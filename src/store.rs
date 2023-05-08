use eyre::{bail, Result};
use futures::TryStreamExt;
use log::warn;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    ConnectOptions, SqlitePool,
};
use std::str::FromStr;
use std::{env, fs, path::Path};

const STORE_NAME: &str = "store.db";

async fn create_db_and_mk_tables(conn_string: &str) -> Result<()> {
    let mut conn = SqliteConnectOptions::from_str(conn_string)?
        .create_if_missing(true)
        .connect()
        .await?;

    warn!("Creating table..");
    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS tokens (
            token TEXT PRIMARY KEY,
            user_id INTEGER
        )
        ",
    )
    .execute(&mut conn)
    .await?;

    Ok(())
}

pub struct Store {
    pool: SqlitePool,
}

impl Store {
    pub async fn connect(data_dir: &Path) -> Result<Self> {
        warn!("Creating DB connection in data-dir {data_dir:?}..");
        fs::create_dir_all(data_dir)?;

        let db_path = if data_dir.is_absolute() {
            data_dir.into()
        } else {
            let cwd = env::current_dir()?;
            cwd.join(data_dir)
        }
        .join(STORE_NAME);

        let conn_string = format!("sqlite://{}", db_path.to_string_lossy());

        let is_new_db = !db_path.exists();
        if !db_path.exists() {
            create_db_and_mk_tables(&conn_string).await?;
        }

        warn!("Creating DB pool {conn_string} (new? {is_new_db})");
        let pool = SqlitePoolOptions::new().connect(&conn_string).await?;

        Ok(Self { pool })
    }

    /// Checks whether the user is already allocated
    pub async fn has_allocated_token(&self, user_id: &str) -> Result<bool> {
        let mut results = sqlx::query("SELECT * FROM tokens WHERE user_id = ?")
            .bind(user_id)
            .fetch(&self.pool);

        if (results.try_next().await?).is_some() {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Checks whether a provided token is valid and not allocated
    pub async fn is_token_valid(&self, token: &str) -> Result<bool> {
        let mut results = sqlx::query_as::<_, TokenEntry>("SELECT * FROM tokens WHERE token = ?")
            .bind(token)
            .fetch(&self.pool);

        if let Some(entry) = results.try_next().await? {
            Ok(entry.user_id.is_none())
        } else {
            Ok(false)
        }
    }

    /// Allocates a given user to a specific token
    pub async fn allocate(&self, user_id: &str, token: &str) -> Result<()> {
        let mut tx_conn = self.pool.begin().await?;

        {
            let mut results =
                sqlx::query_as::<_, TokenEntry>("SELECT * FROM tokens WHERE token = ?")
                    .bind(token)
                    .fetch(&mut tx_conn);

            if let Some(entry) = results.try_next().await? {
                if entry.user_id.is_some() {
                    bail!("Token is already allocated");
                }
            } else {
                bail!("Token is invalid");
            }
        }

        sqlx::query("UPDATE tokens SET user_id = ? WHERE token = ?")
            .bind(user_id)
            .bind(token)
            .execute(&mut tx_conn)
            .await?;

        tx_conn.commit().await?;
        Ok(())
    }

    /// Ensures that all the provided tokens are provided in the DB, adding them if necessary. Does
    /// not overwrite allocations.
    pub async fn ensure_tokens(&self, tokens: &[String]) -> Result<()> {
        warn!("Ensuring {} tokens are in the DB..", tokens.len());

        for token in tokens {
            sqlx::query("INSERT OR IGNORE INTO tokens (token) VALUES(?)")
                .bind(token)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
struct TokenEntry {
    token: String,
    user_id: Option<String>,
}
