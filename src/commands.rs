// Copyright: https://github.com/hex0x0000/BirthdayBot/src/branch/master/LICENSE
use std::num::ParseIntError;

use crate::database::RemoveBirthday;
use crate::globals::{Bot, DB, LABELS};
use crate::send;
use anyhow::Context;
use teloxide::types::{MessageEntityKind, ParseMode};
use teloxide::{
    prelude::*,
    types::{ChatMember, Me},
    utils::command::BotCommands,
};

#[derive(BotCommands, Clone, Debug)]
#[command(description = "Bot's commands:")]
pub enum Command {
    #[command(description = "displays available commands", rename = "lowercase")]
    Help,
    #[command(description = "starts bot", rename = "lowercase")]
    Start,
    #[command(description = "bot's info", rename = "lowercase")]
    Info,
    #[command(
        description = "adds your birthday (YYYY/MM/DD). Example /addmybirthday 2000/01/01",
        rename = "lowercase"
    )]
    AddMyBirthday(String),
    #[command(
        description = "adds someone else's birthday (YYYY/MM/DD). Example /addbirthday @user 2000/01/01",
        parse_with = "split",
        rename = "lowercase"
    )]
    AddBirthday { username: String, date: String },
    #[command(
        description = "removes your birthday from a group",
        rename = "lowercase"
    )]
    RemoveMyBirthday,
    #[command(
        description = "removes all the group's birthdays (admins only)",
        rename = "lowercase"
    )]
    RemoveGroup,
    #[command(
        description = "removes your birthdays from any group",
        rename = "lowercase"
    )]
    RemoveAllMyBirthdays,
}

pub async fn answer(bot: Bot, message: Message, command: Command) -> anyhow::Result<()> {
    let me: Me = bot.get_me().await?;
    let me: ChatMember = bot.get_chat_member(message.chat.id, me.id).await?;
    let lang: String = match message.from() {
        Some(user) => match user.language_code.clone() {
            Some(lang) => lang,
            None => "en".to_string(),
        },
        None => "en".to_string(),
    };
    log::info!("Issued command: {:?}", command);
    match command {
        Command::Help => {
            send!(bot, message.chat.id, &lang, "HELP");
        }
        Command::Start => {
            if message.chat.is_group() || message.chat.is_supergroup() {
                if !me.can_pin_messages() {
                    send!(bot, message.chat.id, &lang, "NO_PIN_PERM");
                } else {
                    send!(bot, message.chat.id, &lang, "START_MESSAGE_GRP");
                }
            } else {
                send!(bot, message.chat.id, &lang, "START_MESSAGE_PVT");
            }
        }
        Command::Info => {
            bot.send_message(message.chat.id, LABELS.get(&lang, "INFO"))
                .parse_mode(ParseMode::Html)
                .disable_web_page_preview(true)
                .await
                .context("Failed to send INFO")?;
        }
        Command::AddMyBirthday(date) => {
            if message.chat.is_group() || message.chat.is_supergroup() {
                let (year, month, day) = match {
                    let list: Vec<Result<u16, ParseIntError>> =
                        date.split('/').map(|x| x.parse::<u16>()).collect();
                    let (year, month, day) = (list[0].clone()?, list[1].clone()?, list[2].clone()?);
                    if (month < 1 || month > 12) || (day < 1 || day > 31) {
                        Err(anyhow::anyhow!("Invalid date"))
                    } else {
                        Ok((year, month, day))
                    }
                } {
                    Ok(var) => var,
                    Err(_) => {
                        send!(bot, message.chat.id, &lang, "ERR_INVALID_DATE");
                        return Ok(());
                    }
                };
                let was_added = DB
                    .get()
                    .await
                    .add_birthday(
                        if let Some(user) = message.from() {
                            user.id.0 as f64
                        } else {
                            send!(bot, message.chat.id, &lang, "ERR_COULDNT_GET_USERID");
                            return Ok(());
                        },
                        message.chat.id.0,
                        &lang,
                        year,
                        month,
                        day,
                    )
                    .await?;
                if was_added {
                    send!(bot, message.chat.id, &lang, "BIRTHDAY_ADD_SUCCESS");
                } else {
                    send!(bot, message.chat.id, &lang, "BIRTHDAY_EXISTS");
                }
            } else {
                send!(bot, message.chat.id, &lang, "ERR_ONLY_GROUPS");
            }
        }
        Command::AddBirthday { username, date } => {
            if message.chat.is_group() || message.chat.is_supergroup() {
                let (year, month, day) = match {
                    let list: Vec<Result<u16, ParseIntError>> =
                        date.split('/').map(|x| x.parse::<u16>()).collect();
                    let (year, month, day) = (list[0].clone()?, list[1].clone()?, list[2].clone()?);
                    if (month < 1 || month > 12) || (day < 1 || day > 31) {
                        Err(anyhow::anyhow!("Invalid date"))
                    } else {
                        Ok((year, month, day))
                    }
                } {
                    Ok(var) => var,
                    Err(_) => {
                        send!(bot, message.chat.id, &lang, "ERR_INVALID_DATE");
                        return Ok(());
                    }
                };
                if let Some(entities) = message.entities() {
                    for entity in entities {
                        match entity.kind.clone() {
                            MessageEntityKind::Mention => {
                                if is_username_valid(&username) {
                                    let id = match get_id(&username).await {
                                        Ok(id) => id,
                                        Err(err) => {
                                            log::error!(
                                                "Failed to get {}'s id. Error: {}. Root cause: {}",
                                                username,
                                                err,
                                                err.root_cause()
                                            );
                                            send!(
                                                bot,
                                                message.chat.id,
                                                &lang,
                                                "ERR_COULDNT_GET_USERID"
                                            );
                                            return Ok(());
                                        }
                                    };
                                    if id == 0.0 {
                                        send!(bot, message.chat.id, &lang, "ERR_USER_NOT_FOUND");
                                        return Ok(());
                                    }
                                    let was_added = DB
                                        .get()
                                        .await
                                        .add_birthday(
                                            id,
                                            message.chat.id.0,
                                            &lang,
                                            year,
                                            month,
                                            day,
                                        )
                                        .await?;
                                    if was_added {
                                        send!(bot, message.chat.id, &lang, "BIRTHDAY_ADD_SUCCESS");
                                    } else {
                                        send!(bot, message.chat.id, &lang, "BIRTHDAY_EXISTS");
                                    }
                                } else {
                                    send!(bot, message.chat.id, &lang, "ERR_USERNAME_INVALID");
                                }
                                break;
                            }
                            MessageEntityKind::TextMention { user } => {
                                DB.get()
                                    .await
                                    .add_birthday(
                                        user.id.0 as f64,
                                        message.chat.id.0,
                                        &lang,
                                        year,
                                        month,
                                        day,
                                    )
                                    .await?;
                                send!(bot, message.chat.id, &lang, "BIRTHDAY_ADD_SUCCESS");
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            } else {
                send!(bot, message.chat.id, &lang, "ERR_ONLY_GROUPS");
            }
        }
        Command::RemoveMyBirthday => {
            if message.chat.is_group() || message.chat.is_supergroup() {
                DB.get()
                    .await
                    .rm_birthday(RemoveBirthday::RemoveUserInGroup {
                        group_id: message.chat.id.0,
                        user_id: if let Some(user) = message.from() {
                            user.id.0 as f64
                        } else {
                            send!(bot, message.chat.id, &lang, "ERR_COULDNT_GET_USERID");
                            return Ok(());
                        },
                    })
                    .await?;
                send!(bot, message.chat.id, &lang, "DONE");
            } else {
                send!(bot, message.chat.id, &lang, "ERR_ONLY_GROUPS");
            }
        }
        Command::RemoveGroup => {
            if message.chat.is_group() || message.chat.is_supergroup() {
                let admins: Vec<u64> = bot
                    .get_chat_administrators(message.chat.id)
                    .await?
                    .iter()
                    .map(|x| x.user.id.0)
                    .collect();
                let user_id = if let Some(user) = message.from() {
                    user.id.0
                } else {
                    send!(bot, message.chat.id, &lang, "ERR_COULDNT_GET_USERID");
                    return Ok(());
                };
                if admins.contains(&user_id) {
                    DB.get()
                        .await
                        .rm_birthday(RemoveBirthday::RemoveGroup(message.chat.id.0))
                        .await?;
                    send!(bot, message.chat.id, &lang, "DONE");
                } else {
                    send!(bot, message.chat.id, &lang, "ERR_DENIED");
                    return Ok(());
                }
            } else {
                send!(bot, message.chat.id, &lang, "ERR_ONLY_GROUPS");
            }
        }
        Command::RemoveAllMyBirthdays => {
            if let Some(user) = message.from() {
                DB.get()
                    .await
                    .rm_birthday(RemoveBirthday::RemoveUser(user.id.0 as f64))
                    .await?;
                send!(bot, message.chat.id, &lang, "DONE");
            } else {
                send!(bot, message.chat.id, &lang, "ERR_COULDNT_GET_USERID");
            }
        }
    }
    Ok(())
}
