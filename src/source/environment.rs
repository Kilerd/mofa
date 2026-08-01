use crate::merge_two_value;
use crate::source::Source;
use crate::utils::build_toml_value;
use std::error::Error;
use toml::{Table, Value};

pub struct EnvironmentSource {
    prefix: String,
    separator: String,
}

impl EnvironmentSource {
    /// the separator splitting an environment variable into a config path, a double
    /// underscore leaves single underscores free for snake_case field names:
    /// `APP_SERVER__PORT` targets `$.server.port` while `APP_DATABASE_URL` targets
    /// the top level `$.database_url`.
    pub const DEFAULT_SEPARATOR: &'static str = "__";

    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            separator: Self::DEFAULT_SEPARATOR.to_owned(),
        }
    }

    /// override the path separator, `EnvironmentSource::new("APP").with_separator("_")`
    /// restores the single underscore convention used before 0.2.
    ///
    /// # Panics
    /// panics if the given separator is empty.
    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        let separator = separator.into();
        assert!(
            !separator.is_empty(),
            "environment source separator cannot be empty"
        );
        self.separator = separator;
        self
    }
}

impl Source for EnvironmentSource {
    fn load(&self) -> Result<Value, Box<dyn Error>> {
        let prefix = format!("{}_", self.prefix);
        let environments: Vec<Value> = std::env::vars()
            .filter(|(key, _)| key.starts_with(&prefix))
            .map(|(key, value)| (key.strip_prefix(&prefix).unwrap_or(&key).to_owned(), value))
            .filter(|(key, _)| !key.is_empty())
            .map(|(key, value)| build_toml_value(key.to_lowercase(), value, &self.separator))
            .collect();

        let mut accr = Value::Table(Table::new());
        for x in environments.into_iter() {
            accr = merge_two_value(accr, x, "$")?;
        }

        Ok(accr)
    }
}
