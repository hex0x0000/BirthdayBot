// Copyright: https://git.bounceme.net/hex0x0000/BirthdayBot/src/branch/master/LICENSE
use json::*;

pub struct Langs {
    json: JsonValue,
}

impl Langs {
    pub fn new(json: &str, help_msg: &str) -> Result<Self> {
        let mut json = match parse(json) {
            Ok(var) => var,
            Err(err) => return Err(err),
        };
        json["en"]["HELP"] = help_msg.into();
        Ok(Self { json })
    }

    pub fn get(&self, lang: &str, msg: &str) -> String {
        if self.json[lang][msg].is_null() {
            let msg = &self.json["en"][msg];
            if msg.is_null() {
                panic!("{} label doesn't exist", msg);
            }
            return msg.to_string();
        }
        self.json[lang][msg].to_string()
    }
}
