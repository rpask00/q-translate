use q_translate::utils;
use std::fs;
use std::fs::File;
use std::io::Write;

/// # Description
///
/// This tool recreates an existing translation file by automatically translating all text values into a chosen target language.
///
/// It reads a structured JSON file (for example, an i18n resource file), walks through it recursively, and produces a new file with the same structure and key order. All string values are translated into the target language, while non-string values (numbers, booleans, arrays, nulls) are preserved as-is.
///
/// The result is a ready-to-use translation file that mirrors the original exactly, but with all human-readable text translated.
///
/// # What the tool does
///
/// - Preserves the original JSON structure and nesting
/// - Keeps key insertion order intact
/// - Translates only string values
/// - Copies non-string values without modification
/// - Outputs a fully reconstructed file in the target language
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let source_lang = "pl";
    let target_lang = "es";

    let source_path = format!("assets/{source_lang}.json");
    let target_path = format!("assets/{target_lang}.json");

    let source_json = serde_json::from_str(&fs::read_to_string(source_path)?)?;

    let mut target_json = match fs::exists(&target_path)? {
        true => serde_json::from_str(&fs::read_to_string(&target_path)?)?,
        false => serde_json::from_str("{}")?,
    };

    utils::traverse(&source_json, &mut target_json, None, 0, target_lang).await;

    let mut target_file = File::create(&target_path)?;
    target_file.write_all(serde_json::to_string_pretty(&target_json)?.as_bytes())?;

    Ok(())
}
