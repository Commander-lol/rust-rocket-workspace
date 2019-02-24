use failure::Error;
use rocket::config::Value;
use rocket::Config;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Into;
use std::str::FromStr;

/// Map one or more settings value names to environment variables directly
///
/// # Examples
///
/// Using `map_to_env` in non-strict mode will ignore errors retrieving the
/// variable from the environment (commonly the variable not being set). If
/// an error is found, the value will not be set on the config map;
/// this essentially delegates existence checks to your settings config
/// conversion.
///
/// ```
/// let mut conf = Config::new();
///
/// map_to_env!(conf, {
///     "port" => "PORT",
///     "static_dir" => "CONTAINER_VOLUME_MOUNT"
/// });
/// ```
///
/// Conversely, using `map_to_env` in strict mode will propagate errors from reading
/// the environment. This means that a variable not being set will be raised as an
/// error.
///
/// ```
/// let mut conf = Config::new();
///
/// map_to_env!(strict conf, {
///     "dburl" => "DATABASE_URL"
/// });
/// ```
///
/// If you need both strict and non-strict mappings, use two blocks to make it
/// explicit which variables are required
///
/// ```
/// let mut conf = Config::new();
///
/// map_to_env!(conf, {
///     "port" => "PORT",
///     "static_dir" => "CONTAINER_VOLUME_MOUNT"
/// });
///
/// map_to_env!(strict conf, {
///     "dburl" => "DATABASE_URL"
/// });
/// ```
macro_rules! map_to_env {
    ($settings:ident, {$( $setting_name:expr => $env_name:expr ),+}) => {
        {
            use std::env::var;
            $(
            if let Ok(env_var) = var($env_name) {
                $settings.set($setting_name, env_var)?;
            }
            )+
        }
    };
    (strict $settings:ident, {$( $setting_name:expr => $env_name:expr ),+}) => {
        {
            use std::env::var;
            $(
                $settings.set($setting_name, var($env_name)?)?;
            )+
        }
    };
}

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
    /// The route prefix to use when mounting the static file handler
    pub static_route: String,

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
        conf.set_default("static_route", String::from("/static"))?;

        map_to_env!(conf, {
            "port" => "PORT"
        });

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
