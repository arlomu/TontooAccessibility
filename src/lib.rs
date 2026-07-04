use std::collections::HashMap;
use std::path::Path;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

/// Initialize the library by loading language files from the `lang/` directory.
/// This should be called once at application startup.
pub fn init_lang(fallback: &str) -> Result<(), LangError> {
    let lang_dir = Path::new("./lang");
    
    if !lang_dir.exists() {
        std::fs::create_dir_all(lang_dir)?;
    }
    
    let mut files: Vec<LangFile> = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(lang_dir) {
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(file) = LangFile::from_file(&path) {
                    files.push(file);
                }
            }
        }
    }
    
    LangStore::init(files, Some(fallback.to_string()))
}

/// Validate a language file JSON structure without loading it.
/// Returns Ok(()) if valid, Err with description if invalid.
pub fn validate_lang_file<P: AsRef<Path>>(path: P) -> Result<(), LangError> {
    let data = std::fs::read_to_string(path.as_ref())?;
    let value = serde_json::from_str::<serde_json::Value>(&data)?;
    
    if !value.is_object() {
        return Err(LangError::Lang("Root must be an object".to_string()));
    }
    
    let obj = value.as_object().unwrap();
    
    if !obj.contains_key("lang") {
        return Err(LangError::Lang("Missing 'lang' field".to_string()));
    }
    
    if !obj.contains_key("translations") {
        return Err(LangError::Lang("Missing 'translations' field".to_string()));
    }
    
    let lang = obj.get("lang").and_then(|v| v.as_str()).unwrap();
    validate_lang_name(lang)?;
    
    let translations = obj.get("translations").and_then(|v| v.as_object()).unwrap();
    for (key, _) in translations {
        if key.is_empty() {
            return Err(LangError::Lang("Translation key cannot be empty".to_string()));
        }
    }
    
    Ok(())
}

/// Validate all language files in a directory.
/// Returns list of (path, error) tuples for invalid files.
pub fn validate_lang_dir<P: AsRef<Path>>(dir: P) -> Result<Vec<(String, String)>, LangError> {
    let mut errors = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Err(e) = validate_lang_file(&path) {
                    errors.push((path.display().to_string(), e.to_string()));
                }
            }
        }
    }
    
    Ok(errors)
}

/// Setup function for embedding projects - creates lang/ directory and example files.
pub fn setup_lang_dir<P: AsRef<Path>>(dest: P) -> Result<(), LangError> {
    let dest = dest.as_ref();
    
    if !dest.exists() {
        std::fs::create_dir_all(dest)?;
    }
    
    let mut map_en = HashMap::new();
    map_en.insert("app.title".to_string(), "My Application".to_string());
    map_en.insert("button.ok".to_string(), "OK".to_string());
    map_en.insert("button.cancel".to_string(), "Cancel".to_string());
    
    let mut map_de = HashMap::new();
    map_de.insert("app.title".to_string(), "Meine Anwendung".to_string());
    map_de.insert("button.ok".to_string(), "OK".to_string());
    map_de.insert("button.cancel".to_string(), "Abbrechen".to_string());
    
    let example_en = LangFile::new("en_us", map_en);
    let example_de = LangFile::new("de_de", map_de);
    
    let json_en = serde_json::to_string_pretty(&example_en)?;
    let json_de = serde_json::to_string_pretty(&example_de)?;
    
    std::fs::write(dest.join("en_us.json"), json_en)?;
    std::fs::write(dest.join("de_de.json"), json_de)?;
    
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangFile {
    pub lang: String,
    pub translations: HashMap<String, String>,
}

#[derive(Debug, Clone, Default)]
pub struct LangStore {
    files: HashMap<String, LangFile>,
    fallback: Option<String>,
}

static LANG_STORE: Lazy<std::sync::Mutex<LangStore>> = Lazy::new(|| {
    std::sync::Mutex::new(LangStore::default())
});

impl LangFile {
    pub fn new(lang: impl Into<String>, translations: impl Into<HashMap<String, String>>) -> Self {
        Self {
            lang: lang.into(),
            translations: translations.into(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, LangError> {
        let data = std::fs::read_to_string(path.as_ref())?;
        let lang_file = serde_json::from_str::<LangFile>(&data)?;
        verify_no_percent(lang_file.lang.as_str())?;
        Ok(lang_file)
    }

    pub fn write(&self) -> Result<(), LangError> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(self.file_path(), json)?;
        Ok(())
    }

    pub fn file_path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(format!("./lang/{}.json", self.lang))
    }

    pub fn t(&self, key: &str, args: Option<&std::collections::HashMap<String, String>>) -> Option<String> {
        let text = self.translations.get(key)?;
        if let Some(provided) = args {
            Some(apply_placeholders(text, provided))
        } else {
            Some(text.clone())
        }
    }
}

impl LangStore {
    pub fn instance() -> std::sync::MutexGuard<'static, Self> {
        LANG_STORE.lock().unwrap()
    }

    pub fn init(files: Vec<LangFile>, fallback: Option<String>) -> Result<(), LangError> {
        let mut store = Self::instance();

        assert_valid(&files, &fallback)?;

        store.files.clear();
        store.fallback = fallback;

        for file in files {
            store.files.insert(file.lang.clone(), file);
        }

        Ok(())
    }

    pub fn add(&mut self, file: LangFile) -> Result<(), LangError> {
        verify_no_percent(file.lang.as_str())?;
        assert_valid(&vec![], &self.fallback.clone())?;
        self.files.insert(file.lang.clone(), file);
        Ok(())
    }

    pub fn t(&self, lang: &str, key: &str, args: Option<&std::collections::HashMap<String, String>>) -> Option<String> {
        let lang = self.resolve_lang(lang);
        self.files.get(&lang).and_then(|f| f.t(key, args).map(|s| s.to_string()))
    }

    pub fn resolve_lang(&self, lang: &str) -> String {
        if self.files.contains_key(lang) {
            return lang.to_string();
        }

        let base = lang.split('_').next().unwrap_or(lang);
        self.files
            .keys()
            .find(|f| f.starts_with(base))
            .cloned()
            .unwrap_or_else(|| self.fallback.clone().unwrap_or_else(|| "en_us".to_string()))
    }

    pub fn fallback(&self) -> Option<&str> {
        self.fallback.as_deref()
    }

    pub fn all_langs(&self) -> Vec<&str> {
        self.files.keys().map(|s| s.as_str()).collect()
    }
}

#[derive(Debug)]
pub enum LangError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Lang(String),
}

impl From<std::io::Error> for LangError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for LangError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl std::fmt::Display for LangError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::Json(e) => write!(f, "JSON error: {}", e),
            Self::Lang(m) => write!(f, "Lang error: {}", m),
        }
    }
}

impl std::error::Error for LangError {}

fn apply_placeholders(template: &str, args: &std::collections::HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in args {
        let pattern = format!("%{}%", key);
        result = result.replace(&pattern, value);
    }
    result
}

fn validate_lang_name(lang: &str) -> Result<(), LangError> {
    verify_no_percent(lang)?;
    if !lang.bytes().all(|b| b.is_ascii_lowercase() || b == b'_') {
        return Err(LangError::Lang("Invalid lang name: only [a-z_] allowed".to_string()));
    }
    Ok(())
}

fn verify_no_percent(value: &str) -> Result<(), LangError> {
    if value.contains('%') {
        return Err(LangError::Lang("Value must not contain '%'".to_string()));
    }
    Ok(())
}

fn assert_valid(files: &[LangFile], fallback: &Option<String>) -> Result<(), LangError> {
    for file in files {
        validate_lang_name(&file.lang)?;
    }

    if let Some(fb) = fallback {
        validate_lang_name(fb)?;
        let valid = files.iter().any(|file| file.lang == *fb);
        if !valid {
            return Err(LangError::Lang(format!("Invalid fallback lang: {}", fb)));
        }
    }

    Ok(())
}

/// Macro for translations.
/// Usage:
///   t!("key") - translates with default language
///   t!("en_us", "key") - translates with specific language
///   t!("en_us", "key", { "arg" => "value" }) - translates with args
#[macro_export]
macro_rules! t {
    ($key:expr) => {{
        use crate::LangStore;
        LangStore::instance().t("en_us", $key, None).unwrap_or_else(|| format!("{:?}", $key))
    }};
    
    ($lang:expr, $key:expr) => {{
        use crate::LangStore;
        LangStore::instance().t($lang, $key, None).unwrap_or_else(|| format!("{:?}", $key))
    }};
    
    ($lang:expr, $key:expr, $args:expr) => {{
        use crate::LangStore;
        LangStore::instance().t($lang, $key, Some($args)).unwrap_or_else(|| format!("{:?}", $key))
    }};
}