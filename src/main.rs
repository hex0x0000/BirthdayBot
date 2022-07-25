// Copyright: https://git.bounceme.net/hex0x0000/BirthdayBot/src/branch/master/LICENSE
#[macro_use]
mod commands;
mod database;
mod globals;
mod lang;
mod lookup;
mod macros;
use crate::lookup::initialize;
use crate::{commands::*, database::Database};
use anyhow::Context;
use chrono::{prelude::*, Duration};
use database::RemoveBirthday;
use dotenv::dotenv;
use globals::{Bot, DB, LABELS};
use teloxide::{
    adaptors::throttle::Limits,
    prelude::*,
    types::{ChatMember, MessageEntity, MessageEntityKind, Recipient},
    utils::command::BotCommands,
    ApiError, RequestError,
};
use tokio::time::sleep;

fn format(fmt: String, args: &[String]) -> (String, Vec<(usize, usize)>) {
    let mut new = String::new();
    let mut args_pos: Vec<(usize, usize)> = vec![];
    let mut utf16_size: usize = 0;
    let mut arg_n: usize = 0;
    for c in fmt.chars().collect::<Vec<char>>().iter() {
        if *c == '&' && arg_n < args.len() {
            let len: usize = args[arg_n].chars().map(|x| x.len_utf16()).sum();
            new.push_str(&args[arg_n]);
            args_pos.push((utf16_size, len));
            utf16_size += len;
            arg_n += 1;
        } else {
            new.push(*c);
            utf16_size += c.len_utf16();
        }
    }
    (new, args_pos)
}

async fn wish_happy_birthday(bot: &Bot) -> anyhow::Result<()> {
    let now: DateTime<Utc> = Utc::now();
    let birthdays = DB
        .get()
        .await
        .get_birthdays(now.month(), now.day())
        .await
        .context("Failed to get birthdays")?;
    for birthday in birthdays {
        let user: ChatMember = match bot
            .get_chat_member(
                Recipient::Id(ChatId(birthday.group_id)),
                UserId(birthday.user_id as u64),
            )
            .await
        {
            Ok(user) => user,
            Err(err) => {
                if let RequestError::Api(api_err) = err {
                    match api_err {
                        ApiError::ChatNotFound => {
                            DB.get()
                                .await
                                .rm_birthday(RemoveBirthday::RemoveGroup(birthday.group_id))
                                .await?;
                            log::info!("Group removed: {}", birthday.group_id);
                            continue;
                        }
                        ApiError::UserNotFound => {
                            DB.get()
                                .await
                                .rm_birthday(RemoveBirthday::RemoveUserInGroup {
                                    group_id: birthday.group_id,
                                    user_id: birthday.user_id,
                                })
                                .await?;
                            log::info!("{} in {} removed", birthday.user_id, birthday.group_id);
                            continue;
                        }
                        _ => log::error!("API error while iterating birthdays: {:?}", api_err),
                    }
                    continue;
                } else {
                    log::error!("Request error: {:?}", err);
                    continue;
                }
            }
        };
        let (fmt_happy_birthday, args_pos) = format(
            LABELS.get(&birthday.user_lang, "WISH_HAPPY_BDAY"),
            &[
                user.user.first_name.clone(),
                format!("{}", now.year() - birthday.year as i32),
                format!("{}/{}/{}", now.year(), now.month(), now.day()),
            ],
        );
        let (offset, length) = args_pos[0];
        let msg: Message = match bot
            .send_message(Recipient::Id(ChatId(birthday.group_id)), fmt_happy_birthday)
            .entities(vec![MessageEntity {
                kind: MessageEntityKind::TextMention { user: user.user },
                offset,
                length,
            }])
            .await
        {
            Ok(msg) => msg,
            Err(err) => {
                if let RequestError::Api(api_err) = &err {
                    match api_err {
                        ApiError::ChatNotFound => {
                            DB.get()
                                .await
                                .rm_birthday(RemoveBirthday::RemoveGroup(birthday.group_id))
                                .await?;
                            log::info!("Group removed: {}", birthday.group_id);
                            continue;
                        }
                        _ => {}
                    }
                }
                log::error!("Request error: {:?}", err);
                continue;
            }
        };
        if let Err(err) = bot
            .pin_chat_message(Recipient::Id(ChatId(birthday.group_id)), msg.id)
            .await
        {
            if let RequestError::Api(api_err) = &err {
                match api_err {
                    ApiError::NotEnoughRightsToManagePins => {
                        send!(
                            bot,
                            Recipient::Id(ChatId(birthday.group_id)),
                            &birthday.user_lang,
                            "NO_PIN_PERM"
                        );
                        continue;
                    }
                    _ => {}
                }
            }
            log::error!("Request error: {:?}", err);
            continue;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    log::info!("birthday-bot Copyright (C) 2022 Valentino Peggi");
    log::info!("This program comes with ABSOLUTELY NO WARRANTY");
    log::info!(
        "This is free software, and you are welcome to redistribute it under certain conditions"
    );
    Database::new().await.unwrap();
    let mut lookup_server = match initialize().await {
        Ok(child) => child,
        Err(err) => {
            log::error!("Error while initializing: {}", err);
            log::error!("Root cause: {}", err.root_cause());
            return;
        }
    };
    let bot = teloxide::Bot::from_env()
        .throttle(Limits::default())
        .cache_me()
        .auto_send();
    let bot_clone = bot.clone();
    let handler = tokio::spawn(async move {
        let bot = bot_clone;
        loop {
            let now: DateTime<Utc> = Utc::now();
            let tomorrow: DateTime<Utc> = now
                .checked_add_signed(
                    Duration::days(1)
                        - (Duration::minutes(now.minute().into())
                            + Duration::hours(now.hour().into())),
                )
                .expect("Failed to get tomorrow's date");
            sleep(tokio::time::Duration::from_secs(
                (tomorrow.timestamp() - now.timestamp()) as u64,
            ))
            .await;
            log::info!("Starting wish_happy_birthday...");
            if let Err(err) = wish_happy_birthday(&bot).await {
                log::error!("Happy birthday wishing failed: {}", err);
                log::error!("Root cause: {}", err.root_cause());
            }
        }
    });
    teloxide::commands_repl(bot, answer, Command::ty()).await;
    lookup_server.kill().await.log_on_error().await;
    handler.abort();
}
