use clap::Parser;
use q_translate::utils;
use q_translate::utils::{gather_translations, perform_translations};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    source_lang: String,

    #[arg(short, long)]
    target_lang: String,
}

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
    let args = Args::parse();

    let assets_path = if fs::exists("src/assets")? {
        "src/assets/i18n"
    } else if fs::exists("assets")? {
        "assets/i18n"
    } else {
        panic!("Assets directory not found!");
    };

    let source_path = format!("{}/{}.json", assets_path, args.source_lang);
    let target_path = format!("{}/{}.json", assets_path, args.target_lang);

    if !fs::exists(&source_path)? {
        panic!("Source file {} does not exists!", source_path);
    }

    let source_json = serde_json::from_str(&fs::read_to_string(source_path)?)?;

    let mut target_json = match fs::exists(&target_path)? {
        true => serde_json::from_str(&fs::read_to_string(&target_path)?)?,
        false => serde_json::from_str("{}")?,
    };

    let mut translations: HashMap<String, String> = HashMap::default();

    gather_translations(&source_json, &mut translations);
    perform_translations(&mut translations, &args.target_lang).await.unwrap();

    utils::traverse(&source_json, &mut target_json, None, 0, &args.target_lang, &translations);

    let mut target_file = File::create(&target_path)?;
    target_file.write_all(serde_json::to_string_pretty(&target_json)?.as_bytes())?;

    Ok(())
}
