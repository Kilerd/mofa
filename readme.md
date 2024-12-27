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