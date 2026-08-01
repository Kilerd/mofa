use itertools::{EitherOrBoth, Itertools};
use std::cmp::max;
use std::fmt::{Display, Formatter};
use toml::value::Datetime;
use toml::{Table, Value};

#[derive(Debug, PartialEq)]
pub struct Error {
    pub path: String,
    pub existed_type: &'static str,
    pub appended_type: &'static str,
    /// the raw string which failed to be parsed into `existed_type`, only set when the
    /// appended value is a string being coerced into an existed typed value.
    pub unparsable_value: Option<String>,
}

impl Error {
    pub fn new(path: String, existed_type: &'static str, appended_type: &'static str) -> Self {
        Self {
            path,
            existed_type,
            appended_type,
            unparsable_value: None,
        }
    }

    pub fn unparsable(path: String, existed_type: &'static str, value: String) -> Self {
        Self {
            path,
            existed_type,
            appended_type: "string",
            unparsable_value: Some(value),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.unparsable_value {
            Some(value) => write!(
                f,
                "merge fail, path={}, cannot parse {:?} into existed type={}",
                self.path, value, self.existed_type
            ),
            None => write!(
                f,
                "merge fail, path={}, existed type={} appended type={}",
                self.path, self.existed_type, self.appended_type
            ),
        }
    }
}

impl std::error::Error for Error {}

/// build a nested toml value out of a flatten `key`, the `separator` splits the key into
/// the path of the value: given `__` as separator, `server__port` builds `$.server.port`,
/// while `database_url` stays as the top level `$.database_url`.
///
/// empty segments are ignored, so a key made of separators only builds an empty table,
/// which merges into anything as a no-op.
pub fn build_toml_value(key: String, value: String, separator: &str) -> toml::Value {
    let segments = key
        .split(separator)
        .filter(|segment| !segment.is_empty())
        .collect_vec();

    let mut rev = segments.into_iter().rev();

    let Some(leaf) = rev.next() else {
        return Value::Table(Table::new());
    };

    let mut accr = Table::new();
    accr.insert(leaf.to_owned(), Value::String(value));

    rev.fold(Value::Table(accr), |accr, text| {
        let mut map = Table::new();
        map.insert(text.to_owned(), accr);
        Value::Table(map)
    })
}

/// sources like the environment one can only provide strings, so an appended string gets
/// parsed into the type of the value it overrides, instead of failing the whole merge.
fn coerce_string(
    raw: String,
    existed_type: &'static str,
    path: &str,
    parse: impl Fn(&str) -> Option<Value>,
) -> Result<Value, Error> {
    parse(raw.trim()).ok_or_else(|| Error::unparsable(path.to_owned(), existed_type, raw))
}

fn parse_boolean(raw: &str) -> Option<Value> {
    match raw.to_ascii_lowercase().as_str() {
        "true" | "1" => Some(Value::Boolean(true)),
        "false" | "0" => Some(Value::Boolean(false)),
        _ => None,
    }
}

fn merge_into_table_inner(value: &mut Table, other: Table, path: &str) -> Result<(), Error> {
    for (name, inner) in other {
        if let Some(existing) = value.remove(&name) {
            let inner_path = format!("{path}.{name}");
            value.insert(name, merge_two_value(existing, inner, &inner_path)?);
        } else {
            value.insert(name, inner);
        }
    }
    Ok(())
}

pub fn merge_two_value(base: Value, append: Value, path: &str) -> Result<Value, Error> {
    match (base, append) {
        (Value::String(_), Value::String(inner)) => Ok(Value::String(inner)),
        (Value::Integer(_), Value::Integer(inner)) => Ok(Value::Integer(inner)),
        (Value::Float(_), Value::Float(inner)) => Ok(Value::Float(inner)),
        (Value::Boolean(_), Value::Boolean(inner)) => Ok(Value::Boolean(inner)),
        (Value::Datetime(_), Value::Datetime(inner)) => Ok(Value::Datetime(inner)),
        (Value::Integer(_), Value::String(raw)) => coerce_string(raw, "integer", path, |it| {
            it.parse::<i64>().ok().map(Value::Integer)
        }),
        (Value::Float(_), Value::String(raw)) => coerce_string(raw, "float", path, |it| {
            it.parse::<f64>().ok().map(Value::Float)
        }),
        (Value::Boolean(_), Value::String(raw)) => {
            coerce_string(raw, "boolean", path, parse_boolean)
        }
        (Value::Datetime(_), Value::String(raw)) => coerce_string(raw, "datetime", path, |it| {
            it.parse::<Datetime>().ok().map(Value::Datetime)
        }),
        (Value::Array(existing), Value::Array(inner)) => {
            let mut ret = Vec::with_capacity(max(existing.len(), inner.len()));
            for pair in existing
                .into_iter()
                .enumerate()
                .zip_longest(inner.into_iter().enumerate())
            {
                let element = match pair {
                    EitherOrBoth::Both(l, r) => {
                        merge_two_value(l.1, r.1, &format!("{}.[{}]", path, l.0))?
                    }
                    EitherOrBoth::Left(l) => l.1,
                    EitherOrBoth::Right(r) => r.1,
                };
                ret.push(element);
            }
            Ok(Value::Array(ret))
        }
        (Value::Table(mut existing), Value::Table(inner)) => {
            merge_into_table_inner(&mut existing, inner, path)?;
            Ok(Value::Table(existing))
        }
        (v, o) => Err(Error::new(path.to_owned(), v.type_str(), o.type_str())),
    }
}

#[cfg(test)]
mod tests {
    use crate::source::environment::EnvironmentSource;
    use crate::utils::{build_toml_value, merge_two_value, Error};
    use toml::Value;

    fn env_value(key: &str, value: &str) -> Value {
        build_toml_value(
            key.to_owned(),
            value.to_owned(),
            EnvironmentSource::DEFAULT_SEPARATOR,
        )
    }

    fn toml_value(content: &str) -> Value {
        toml::from_str(content).unwrap()
    }

    mod build {
        use super::*;

        #[test]
        fn should_build_top_level_value() {
            assert_eq!(toml_value("name = \"mofa\""), env_value("name", "mofa"));
        }

        #[test]
        fn should_keep_snake_case_key_as_top_level_one() {
            assert_eq!(
                toml_value("database_url = \"postgres://\""),
                env_value("database_url", "postgres://")
            );
        }

        #[test]
        fn should_nest_value_on_separator() {
            assert_eq!(
                toml_value("[server]\nport = \"9090\""),
                env_value("server__port", "9090")
            );
            assert_eq!(
                toml_value("[server.tls]\ncert_file = \"/tmp/cert\""),
                env_value("server__tls__cert_file", "/tmp/cert")
            );
        }

        #[test]
        fn should_build_with_customized_separator() {
            assert_eq!(
                toml_value("[server]\nport = \"9090\""),
                build_toml_value("server_port".to_owned(), "9090".to_owned(), "_")
            );
        }

        #[test]
        fn should_ignore_empty_segment() {
            assert_eq!(
                toml_value("[server]\nport = \"9090\""),
                env_value("__server____port__", "9090")
            );
            assert_eq!(toml_value(""), env_value("__", "9090"));
        }
    }

    mod merge {
        use super::*;

        #[test]
        fn should_coerce_string_into_existed_scalar() {
            assert_eq!(
                Ok(Value::Integer(42)),
                merge_two_value(Value::Integer(1), Value::String("42".to_owned()), "$.value")
            );
            assert_eq!(
                Ok(Value::Integer(-42)),
                merge_two_value(
                    Value::Integer(1),
                    Value::String(" -42 ".to_owned()),
                    "$.value"
                )
            );
            assert_eq!(
                Ok(Value::Float(1.5)),
                merge_two_value(
                    Value::Float(0.5),
                    Value::String("1.5".to_owned()),
                    "$.value"
                )
            );
            assert_eq!(
                Ok(toml_value("value = 2024-01-01T00:00:00Z")
                    .get("value")
                    .unwrap()
                    .clone()),
                merge_two_value(
                    toml_value("value = 1979-05-27T07:32:00Z")
                        .get("value")
                        .unwrap()
                        .clone(),
                    Value::String("2024-01-01T00:00:00Z".to_owned()),
                    "$.value"
                )
            );
        }

        #[test]
        fn should_coerce_string_into_existed_boolean() {
            for raw in ["true", "TRUE", "1"] {
                assert_eq!(
                    Ok(Value::Boolean(true)),
                    merge_two_value(
                        Value::Boolean(false),
                        Value::String(raw.to_owned()),
                        "$.value"
                    )
                );
            }
            for raw in ["false", "False", "0"] {
                assert_eq!(
                    Ok(Value::Boolean(false)),
                    merge_two_value(
                        Value::Boolean(true),
                        Value::String(raw.to_owned()),
                        "$.value"
                    )
                );
            }
        }

        #[test]
        fn should_not_coerce_string_into_existed_string() {
            assert_eq!(
                Ok(Value::String("42".to_owned())),
                merge_two_value(
                    Value::String("1".to_owned()),
                    Value::String("42".to_owned()),
                    "$.value"
                )
            );
        }

        #[test]
        fn should_fail_on_unparsable_string() {
            let error = merge_two_value(
                Value::Integer(1),
                Value::String("abc".to_owned()),
                "$.value",
            )
            .unwrap_err();

            assert_eq!(
                Error::unparsable("$.value".to_owned(), "integer", "abc".to_owned()),
                error
            );
            assert_eq!(
                "merge fail, path=$.value, cannot parse \"abc\" into existed type=integer",
                error.to_string()
            );
        }

        #[test]
        fn should_keep_reporting_incompatible_types() {
            let error = merge_two_value(toml_value("value = 1"), toml_value("value = [1]"), "$")
                .unwrap_err();

            assert_eq!(Error::new("$.value".to_owned(), "integer", "array"), error);
            assert_eq!(
                "merge fail, path=$.value, existed type=integer appended type=array",
                error.to_string()
            );
        }

        #[test]
        fn should_coerce_nested_string_within_table() {
            assert_eq!(
                Ok(toml_value("[server]\nport = 9090\nhost = \"localhost\"")),
                merge_two_value(
                    toml_value("[server]\nport = 8080\nhost = \"localhost\""),
                    env_value("server__port", "9090"),
                    "$"
                )
            );
        }
    }
}
