# TontooAccessibility

A simple Rust internationalization library for embedding in other projects.

## Usage

Add to your `Cargo.toml`:
```toml
[dependencies]
tontoo-accessibility = { git = "https://github.com/{owner}/{repo}", tag = "v{version}" }
```

## Initialization

```rust
use tontoo_accessibility::{init_lang, t};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Creates ./lang dir if needed and loads all JSON files
    init_lang("en_us")?;
    
    // Use translations
    println!("{}", t!("app.title"));
    println!("{}", t!("de_de", "app.title"));
    
    Ok(())
}
```

## Language Files

Place `lang/de_de.json` and `lang/en_us.json` in your project root:

```json
{
  "lang": "de_de",
  "translations": {
    "app.title": "Meine Anwendung",
    "button.ok": "OK"
  }
}
```

## GitHub CI

The library automatically builds and creates a release when a PR title contains `--final`.