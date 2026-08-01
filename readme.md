<div align="center">
    <h1>mofa</h1>
    <p>the config lib that <b>M</b>erges <b>o</b>ptions <b>f</b>rom <b>a</b>nywhere</p>
    <img alt="GitHub Workflow Status (with branch)" src="https://img.shields.io/github/actions/workflow/status/kilerd/mofa/rust.yaml?branch=main">
    <a href="https://crates.io/crates/mofa"><img alt="Crates.io" src="https://img.shields.io/crates/v/mofa"></a>
    <a href="https://codecov.io/gh/kilerd/mofa" >
    <img src="https://codecov.io/gh/kilerd/mofa/branch/main/graph/badge.svg"/>
    </a>
    <a href="https://crates.io/crates/mofa">
    <img alt="Crates.io (recent)" src="https://img.shields.io/crates/dr/mofa"></a>
    <a href="https://docs.rs/mofa"><img alt="docs.rs" src="https://img.shields.io/docsrs/mofa"></a>
    <img alt="Crates.io" src="https://img.shields.io/crates/l/mofa">
</div>

## Features
- **hierarchy file supported**: file source allow users to load config from multiple files
- **environment variable resolver**: mofa can resolve the placeholder like `${APP_KEY_NAME}`
- **path variable resolver**: mofa can resolve the placeholder like `${parent.sub.key_name}`
- **typed environment override**: `APP_SERVER__PORT=9090` overrides an integer field, environment
  values get parsed into the type of the value they override

## Installation
```sh
$ cargo add mofa
```

## Example
```rust
#[derive(Debug, Deserialize)]
struct Config {
    original: String,
    value: String,
}
fn main() {
    let mut loader = ConfigLoader::new();
    loader.add_source(FileSource::new("configuration/application.toml"));
    loader.add_source(FileSource::new("configuration/application_database.toml"));
    loader.add_source(EnvironmentSource::new("APP"));
    loader.enable_environment_variable_processor();
    loader.enable_path_variable_processor();

    let config: Config = loader.construct()?;
}
```

## Environment source

`EnvironmentSource::new("APP")` picks up every `APP_`-prefixed variable, lowercases the rest of
the name and splits it on a **double underscore** to get the path of the config field, which
leaves single underscores free for snake_case field names:

| variable                     | config path             |
|------------------------------|-------------------------|
| `APP_NAME`                   | `$.name`                |
| `APP_DATABASE_URL`           | `$.database_url`        |
| `APP_SERVER__PORT`           | `$.server.port`         |
| `APP_SERVER__TLS__CERT_FILE` | `$.server.tls.cert_file` |

`EnvironmentSource::new("APP").with_separator("_")` restores the single underscore convention
used before `0.2.0`.

### typed values

An environment variable is always a string, so a variable overriding a non-string field is
parsed into the type of the field it overrides. Given:

```toml
# configurations/application.toml
port = 8080
ratio = 0.5
debug = false
```

`APP_PORT=9090 APP_RATIO=1.5 APP_DEBUG=true` overrides all three of them.

- `integer` and `float` are parsed by rust's `parse`
- `boolean` accepts `true` and `false` case insensitively, plus `1` and `0`
- `datetime` is parsed as a toml datetime
- surrounding whitespace is trimmed before parsing
- a field which is already a string keeps the value as is, `"42"` stays a string

When the value cannot be parsed, the load fails with
`merge fail, path=$.port, cannot parse "abc" into existed type=integer`.

The parsing is driven by the value being overridden, so a field which **only** exists in the
environment stays a string, there is nothing to tell it apart from a genuine string. Give such
a field a value in a config file to type it.

## Contributing
Want to join us? Check out our ["Contributing" guide][contributing] and take a
look at some of these issues:
- [Issues labeled "good first issue"][good-first-issue]
- [Issues labeled "help wanted"][help-wanted]


## License
This project is licensed under MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT]).


[contributing]: https://github.com/kilerd/mofa/blob/master.github/CONTRIBUTING.md
[good-first-issue]: https://github.com/kilerd/mofa/labels/good%20first%20issue
[help-wanted]: https://github.com/kilerd/mofa/labels/help%20wanted