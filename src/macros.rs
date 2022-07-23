// Copyright: https://git.bounceme.net/hex0x0000/BirthdayBot/src/branch/master/LICENSE
#[macro_export]
macro_rules! send {
    ($bot:expr, $chat_id:expr, $lang:expr, $msg:expr) => {
        $bot.send_message($chat_id, LABELS.get($lang, $msg))
            .await
            .context(format!("Failed to send {}", $msg))?;
    };
}
