pub mod config;
pub mod environment;
pub mod init;
pub mod keyhouse;
pub mod notifier;
pub mod utils;
pub mod logger;
#[macro_use]
extern crate error_chain;

pub mod errors {
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
            Json(::serde_json::Error);
            B64DecodeError(::base64::DecodeError);
            UTF8Error(::std::string::FromUtf8Error);
            Reqwest(::reqwest::Error);
            Toml(::toml::de::Error);
            Time(::std::time::SystemTimeError);
        }
    }
}
