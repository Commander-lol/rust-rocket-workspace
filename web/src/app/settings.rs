use failure::Error;
use rocket::config::Value;
use rocket::Config;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Into;
use std::str::FromStr;

pub const ENV_PREFIX: &'static str = "APP";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    address: Option<String>,
    port: Option<u16>,
    log: Option<String>,
    workers: Option<u16>,
    secret_key: Option<String>,
    extras: HashMap<String, String>,
}

const FILTER_EXTRA_KEYS: [&'static str; 5] = ["address", "port", "log", "workers", "secret_key"];

impl Settings {
    pub fn new() -> Result<Settings, Error> {
        use config::{Config, Environment, File};
        use std::env::var;

        let mut conf = Config::new();

        conf.merge(File::with_name("config").required(false))?;

        match var("APP_ENV").unwrap_or(String::from("")).as_str() {
            env @ "development" | env @ "production" | env @ "staging" => {
                conf.merge(File::with_name(&format!("config-{}", env)).required(false))?;
            }
            _ => (),
        };

        conf.merge(Environment::with_prefix(ENV_PREFIX).ignore_empty(true))?;

        let mut extras_config = Config::new();
        extras_config.merge(Environment::with_prefix(ENV_PREFIX).ignore_empty(true))?;

        let mut extras_map: HashMap<String, String> = extras_config.try_into()?;

        for key in FILTER_EXTRA_KEYS.iter() {
            extras_map.remove(&String::from(*key));
        }

        conf.set("extras", extras_map)?;

        Ok(conf.try_into()?)
    }
}

impl Into<Config> for Settings {
    fn into(self) -> Config {
        use rocket::config::{Environment, LoggingLevel};
        let env = Environment::active().unwrap_or(Environment::Production);
        let mut conf = Config::new(env);

        if let Some(address) = self.address {
            conf.set_address(address);
        }
        if let Some(port) = self.port {
            conf.set_port(port);
        }
        if let Some(log) = self.log {
            conf.set_log_level(LoggingLevel::from_str(&log).unwrap_or(LoggingLevel::Normal));
        }
        if let Some(workers) = self.workers {
            conf.set_workers(workers);
        }
        if let Some(secret_key) = self.secret_key {
            conf.set_secret_key(secret_key);
        }

        let table = self
            .extras
            .iter()
            .map(|(key, value)| match Value::try_from(value) {
                Ok(v) => Ok((key.clone(), v)),
                Err(e) => Err(e),
            })
            .collect();

        match table {
            Ok(table) => conf.set_extras(table),
            Err(e) => eprintln!("{}", e),
        }

        conf
    }
}
