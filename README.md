# BirthdayBot
A Telegram Bot that wishes happy birthdays in groups.

[Official instance](https://t.me/happybirthday_bbot)

# How to create your own instance
## Dependencies
If you're using nix just run:
```bash
$ nix-shell
```

On any other distro:
1. [Install Rust](https://rustup.rs/)
2. Install python3.9 and virtualenv (on the host machine)
3. Install gcc, perl, perl-core and make (on the compiling machine)

## Obtaining necessary tokens
1. Make a new bot with [BotFather](https://t.me/BotFather).
2. Make a new [Telegram Application](https://core.telegram.org/api/obtaining_api_id).
After that:
```bash
$ mv username_lookup/config.json.example username_lookup/config.json
$ vim username_lookup/config.json # insert your api_id and api_hash here
```

## Compiling
```bash
$ cargo build --release
$ mv target/release/birthday-bot /path/to/server/files # anywhere you want, as long as the program has enough permissions to write on the same directory
```

## Running
```bash
$ export TELOXIDE_TOKEN="your telegram token made with BotFather here"
$ export DATABASE_URL="sqlite:/path/to/birthdays.db"
$ export RUST_LOG="info" # if you want the log level to be info
$ cd /path/to/server/files
$ ./birthday-bot # the first time it must be run manually because you have to login into telegram, after that you can call it from any init script you want (as long as the program has access to $PATH and the other exported variables)
```