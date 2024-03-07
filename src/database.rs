// Copyright: https://github.com/hex0x0000/BirthdayBot/src/branch/master/LICENSE
use anyhow::Context;
use async_sqlite::{
    rusqlite::{params, OptionalExtension},
    JournalMode, Pool, PoolBuilder,
};
use std::env;

#[derive(Debug)]
pub struct Database {
    pool: Pool,
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
    pub year: u32,
    pub month: u8,
    pub day: u8,
    pub timezone: i8,
}

impl Database {
    pub async fn new() -> anyhow::Result<Self> {
        let database_path = env::var("DATABASE_PATH").context("DATABASE_PATH env var not found")?;
        let pool = PoolBuilder::new()
            .path(database_path)
            .journal_mode(JournalMode::Wal)
            .open()
            .await
            .context("Failed to create DB connection")?;
        pool.conn(|conn| {
            conn.execute_batch(
                "BEGIN;
CREATE TABLE IF NOT EXISTS birthdays (
    id              INTEGER PRIMARY KEY NOT NULL,
    user_id         REAL                NOT NULL,
    group_id        INTEGER             NOT NULL,
    user_lang       TEXT                NOT NULL,
    year            INTEGER             NOT NULL,
    month           INTEGER             NOT NULL,
    day             INTEGER             NOT NULL,
    timezone        INTEGER DEFAULT 0   NOT NULL,
    UNIQUE(user_id, group_id)
);
CREATE TABLE IF NOT EXISTS user-timezones (
    id              INTEGER PRIMARY KEY NOT NULL,
    user_id         REAL                NOT NULL,
    timezone        INTEGER             NOT NULL,
    UNIQUE(user_id)
);
CREATE TABLE IF NOT EXISTS group-timezones (
    id              INTEGER PRIMARY KEY NOT NULL,
    group_id        REAL                NOT NULL,
    timezone        INTEGER             NOT NULL,
    UNIQUE(group_id)
);
COMMIT;",
            )
        })
        .await
        .context("Failed to create database tables")?;
        Ok(Self { pool })
    }

    async fn get_timezone(&self, user_id: f64, group_id: i64) -> anyhow::Result<i8> {
        let user_timezone: Option<i8> = self
            .pool
            .conn(|conn| {
                conn.query_row(
                    "SELECT timezone FROM user-timezones WHERE user_id=?1",
                    [user_id],
                    |row| row.get(0)?,
                )
                .optional()
            })
            .await
            .context("Failed to get user's timezone")?;
        if let Some(timezone) = user_timezone {
            Ok(timezone)
        } else {
            let group_timezone: Option<i8> = self
                .pool
                .conn(|conn| {
                    conn.query_row(
                        "SELECT timezone FROM group-timezones WHERE group_id=?1",
                        [group_id],
                        |row| row.get(0)?,
                    )
                    .optional()
                })
                .await
                .context("Failed to get group's timezone")?;
            if let Some(timezone) = group_timezone {
                Ok(timezone)
            } else {
                Ok(0)
            }
        }
    }

    pub async fn add_birthday(&self, birthday: Birthday) -> anyhow::Result<bool> {
        let already_exists: Option<()> = self
            .pool
            .conn(move |conn| {
                conn.query_row(
                    "SELECT * FROM birthdays WHERE user_id = ?1 AND group_id = ?2",
                    params![birthday.user_id, birthday.group_id],
                    |_| Ok(()),
                )
                .optional()
            })
            .await
            .context("Failed to check if user already exists")?;

        if let None = already_exists {
            let timezone = self
                .get_timezone(birthday.user_id, birthday.group_id)
                .await?;
            self.pool
                .conn(move |conn| {
                    conn.execute(
                        "INSERT INTO birthdays (user_id, group_id, user_lang, year, month, day, timezone) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![birthday.user_id, birthday.group_id, birthday.user_lang, birthday.year, birthday.month, birthday.day, timezone],
                    )
                })
                .await
                .context("Failed to insert birthday")?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn rm_birthday(&self, action: RemoveBirthday) -> anyhow::Result<()> {
        match action {
            RemoveBirthday::RemoveGroup(group_id) => self
                .pool
                .conn(move |conn| {
                    conn.execute("DELETE FROM birthdays WHERE group_id = ?1", [group_id])
                })
                .await
                .context("Failed to remove birthday")?,
            RemoveBirthday::RemoveUser(user_id) => self
                .pool
                .conn(move |conn| {
                    conn.execute("DELETE FROM birthdays WHERE user_id = ?1", [user_id])
                })
                .await
                .context("Failed to remove birthday")?,
            RemoveBirthday::RemoveUserInGroup { group_id, user_id } => self
                .pool
                .conn(move |conn| {
                    conn.execute(
                        "DELETE FROM birthdays WHERE user_id = ?1 AND group_id = ?2",
                        params![user_id, group_id],
                    )
                })
                .await
                .context("Failed to remove birthday")?,
        };
        Ok(())
    }

    pub async fn get_birthdays(
        &self,
        month: u32,
        day: u32,
    ) -> anyhow::Result<Vec<Option<Birthday>>> {
        Ok(self.pool.conn(move |conn| { 
            let mut stmt = conn.prepare("SELECT user_id, group_id, user_lang, year, month, day, timezone FROM birthdays WHERE month = ?1 AND day = ?2")?;
            let query = stmt.query_map([month, day], |row| Ok(
                Birthday {
                    user_id: row.get(0)?,
                    group_id: row.get(1)?,
                    user_lang: row.get(2)?,
                    year: row.get(3)?,
                    month: row.get(4)?,
                    day: row.get(5)?,
                    timezone: row.get(6)?,
                }
            ))?;
            Ok(query.map(|b| b.ok()).collect())
        })
        .await
        .context("Failed to get birthdays")?)
    }
}
