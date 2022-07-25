// Copyright: https://git.bounceme.net/hex0x0000/BirthdayBot/src/branch/master/LICENSE
use anyhow::Context;
use sqlx::sqlite::SqlitePool;
use std::{env, path::PathBuf};
use tokio::{fs::File, io::AsyncWriteExt};

const DB_FILE: &[u8] = include_bytes!("../birthdays.db");

pub struct Database {
    pool: SqlitePool,
}

pub enum RemoveBirthday {
    RemoveGroup(i64),
    RemoveUser(f64),
    RemoveUserInGroup { group_id: i64, user_id: f64 },
}

pub struct Birthday {
    pub user_id: f64,
    pub group_id: i64,
    pub user_lang: String,
    pub year: i64,
    pub month: i64,
    pub day: i64,
}

impl Database {
    pub async fn new() -> anyhow::Result<Self> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL env var not found");
        let database_path = PathBuf::from({
            // Removes sqlite from sqlite:/path/to/database.db
            let mut copy: Vec<&str> = database_url.split(':').collect();
            if copy.remove(0) == "sqlite" {
                copy.join(":")
            } else {
                panic!("Invalid DATABASE_URL, it must be sqlite:/path/to/database.db");
            }
        });
        if !database_path.exists() {
            File::create(database_path)
                .await
                .context("Failed to create DB file")?
                .write_all(DB_FILE)
                .await
                .context("Failed to write to DB file")?;
        }
        let pool = SqlitePool::connect(&database_url)
            .await
            .context("Failed to create DB connection")?;
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .context("Failed to migrate DB")?;
        Ok(Self { pool })
    }

    pub async fn add_birthday(
        &self,
        user_id: f64,
        group_id: i64,
        user_lang: &str,
        year: u16,
        month: u16,
        day: u16,
    ) -> anyhow::Result<bool> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .context("Failed to acquire connection")?;
        if sqlx::query!(
            "SELECT *
            FROM birthdays
            WHERE user_id = ?1 AND group_id = ?2",
            user_id,
            group_id
        )
        .fetch_all(&mut conn)
        .await
        .context("Failed to check if user already exists")?
        .len()
            > 0
        {
            Ok(false)
        } else {
            let id = sqlx::query!(
                "INSERT INTO birthdays (user_id, group_id, user_lang, year, month, day)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                user_id,
                group_id,
                user_lang,
                year,
                month,
                day
            )
            .execute(&mut conn)
            .await
            .context("Failed to insert birthday")?
            .last_insert_rowid();
            log::info!("last rowid: {}", id);
            Ok(true)
        }
    }

    pub async fn rm_birthday(&self, action: RemoveBirthday) -> anyhow::Result<()> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .context("Failed to acquire connection")?;

        match action {
            RemoveBirthday::RemoveGroup(group_id) => sqlx::query!(
                "DELETE FROM birthdays
                WHERE group_id = ?1",
                group_id
            )
            .execute(&mut conn)
            .await
            .context("Failed to remove birthday")?,
            RemoveBirthday::RemoveUser(user_id) => sqlx::query!(
                "DELETE FROM birthdays
                WHERE user_id = ?1",
                user_id
            )
            .execute(&mut conn)
            .await
            .context("Failed to remove birthday")?,
            RemoveBirthday::RemoveUserInGroup { group_id, user_id } => sqlx::query!(
                "DELETE FROM birthdays
                WHERE user_id = ?1 AND group_id = ?2",
                user_id,
                group_id
            )
            .execute(&mut conn)
            .await
            .context("Failed to remove birthday")?,
        };
        Ok(())
    }

    pub async fn get_birthdays(&self, month: u32, day: u32) -> anyhow::Result<Vec<Birthday>> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .context("Failed to acquire connection")?;
        Ok(sqlx::query_as!(
            Birthday,
            "SELECT user_id, group_id, user_lang, year, month, day
            FROM birthdays
            WHERE month = ?1 AND day = ?2",
            month,
            day
        )
        .fetch_all(&mut conn)
        .await
        .context("Failed to get birthdays")?)
    }
}
