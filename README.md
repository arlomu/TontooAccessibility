# TontooAccessibility

A simple Rust internationalization (i18n) library for embedding translations in other projects.

## Made for TontooOS

Explore more at https://github.com/arlomu/TontooOSLibs

## Adding to Your Project

Add to your `Cargo.toml`:

```toml
[dependencies]
tontoo-accessibility = { git = "https://github.com/arlomu/TontooAccessibility" }
```

## Usage

### Initialize

Call `init_lang()` once at application startup. This creates the `./lang/` directory if it doesn't exist and loads all JSON translation files.

```rust
use tontoo_accessibility::{init_lang, t};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize with fallback language
    init_lang("en_us")?;
    
    // Now you can use translations anywhere
    println!("{}", t!("app.title"));
    Ok(())
}
```

### Using Translations

```rust
// With default/fallback language ("en_us" in this example)
t!("button.ok")  // → "OK"

// With specific language
t!("de_de", "button.cancel")  // → "Abbrechen"

// With placeholders
let mut args = std::collections::HashMap::new();
args.insert("name".to_string(), "Alice".to_string());
t!("en_us", "message.welcome", args)  // → "Welcome, Alice!"
```

### Language Files

Place translation files in your project root under `lang/`:

```
lang/
  en_us.json
  de_de.json
```

Example `lang/de_de.json`:
```json
{
  "lang": "de_de",
  "translations": {
    "app.title": "Meine Anwendung",
    "button.ok": "OK",
    "button.cancel": "Abbrechen",
    "message.welcome": "Willkommen, {name}!"
  }
}
```

### Validation

Validate translation files before loading:

```rust
use tontoo_accessibility::{validate_lang_file, validate_lang_dir};

// Validate single file
match validate_lang_file("./lang/de_de.json") {
    Ok(()) => println!("Valid!"),
    Err(e) => eprintln!("Invalid: {}", e),
}

// Validate all files in directory
let errors = validate_lang_dir("./lang")?;
for (file, error) in errors {
    eprintln!("{}: {}", file, error);
}
```

## Repository

https://github.com/arlomu/TontooAccessibility

## License

MIT