// Copyright: https://github.com/hex0x0000/BirthdayBot/src/branch/master/LICENSE
use crate::commands::Command;
use crate::database::Database;
use crate::lang::Langs;
use teloxide::adaptors::{CacheMe, Throttle};
use teloxide::utils::command::BotCommands;
use tokio::sync::OnceCell;

const LANGS_JSON: &str = include_str!("../lang.json");
pub type Bot = CacheMe<Throttle<teloxide::Bot>>;
pub static LABELS: OnceCell<Langs> = OnceCell::const_new();
pub static DB: OnceCell<Database> = OnceCell::const_new();

pub async fn init_globals() {
    LABELS
        .set(
            Langs::new(LANGS_JSON, &Command::descriptions().to_string())
                .expect("Failed to initialize langs"),
        )
        .expect("Failed to set global LABELS value");
    let db = Database::new().await.expect("Failed to initialize db");
    DB.set(db).expect("Failed to set DB global value");
}
