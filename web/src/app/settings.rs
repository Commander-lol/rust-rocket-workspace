use failure::Error;
use rocket::config::Value;
use rocket::Config;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Into;
use std::str::FromStr;

/// The prefix used to pull in environment variables. Any variable prefixed with this value that
/// does not correlate to a property of `Settings` will be added to the `extras` map, which is
/// provided to rocket.
///
/// Ensure that nothing sensitive in your environment has this prefix.
///
/// # Examples
///
/// ```
/// pub const ENV_PREFIX: &'static str = "MY_WEBSITE";
///
/// // Matches "MY_WEBSITE_PORT", "MY_WEBSITE_STATIC_DIR", etc.
/// ```
///
///
pub const ENV_PREFIX: &'static str = "APP";

/// Holds settings for the application. This struct will be passed
/// to rocket, and must contain at least the fields marked [Required].
/// Other fields can be added and removed depending on the application's
/// requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// The disk path that contains static assets
    pub static_dir: String,

    // Rocket Variable are all optional
    // becuase rocket provides defaults
    /// [Required] The address for the app to listen on
    address: Option<String>,
    /// [Required] The port the app will bind to
    port: Option<u16>,
    /// [Required] The level of logging that the web framework should perform.
    /// Should be one of "critical", "normal", "debug" and "off"
    log: Option<String>,
    /// [Required] The number of worker threads that should serve requests
    workers: Option<u16>,
    /// [Required] The app's secret key, used to sign cookies
    secret_key: Option<String>,
    /// [Required] Additional config values for extensions of rocket
    extras: HashMap<String, String>,
}

/// Keys that should be filtered out of the extras map, because they are defined as fields on `Settings`
const FILTER_EXTRA_KEYS: [&'static str; 5] = ["address", "port", "log", "workers", "secret_key"];

impl Settings {
    pub fn new() -> Result<Settings, Error> {
        use config::{Config, Environment, File};
        use std::env::var;

        let mut conf = Config::new();

        conf.set_default("static_dir", concat!(env!("CARGO_MANIFEST_DIR"), "/public"))?;

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
