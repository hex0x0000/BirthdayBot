CREATE TABLE IF NOT EXISTS birthdays
(
    id              INTEGER PRIMARY KEY NOT NULL,
    user_id         REAL                NOT NULL,
    group_id        INTEGER             NOT NULL,
    user_lang       TEXT                NOT NULL,
    year            INTEGER             NOT NULL,
    month           INTEGER             NOT NULL,
    day             INTEGER             NOT NULL 
);