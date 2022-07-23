// Copyright: https://git.bounceme.net/hex0x0000/BirthdayBot/src/branch/master/LICENSE
use crate::commands::Command;
use crate::database::Database;
use crate::lang::Langs;
use async_once::AsyncOnce;
use lazy_static::*;
use teloxide::utils::command::BotCommands;
use teloxide::{
    adaptors::{CacheMe, Throttle},
    prelude::*,
};

const LANGS_JSON: &str = include_str!("../lang.json");
pub type Bot = AutoSend<CacheMe<Throttle<teloxide::Bot>>>;
lazy_static! {
    pub static ref LABELS: Langs = Langs::new(LANGS_JSON, &Command::descriptions().to_string())
        .expect("Failed to parse langs.json");
    pub static ref DB: AsyncOnce<Database> =
        AsyncOnce::new(async { Database::new().await.expect("Failed to initialize db") });
}
