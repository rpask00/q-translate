use q_translate::utils;
use std::fs;
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let SOURCE_LANG = "pl";
    let TARGET_LANG = "ua";

    let source_json =
        serde_json::from_str(&fs::read_to_string(format!("assets/{SOURCE_LANG}.json"))?)?;
    let mut target_json =
        serde_json::from_str(&fs::read_to_string(format!("assets/{TARGET_LANG}.json"))?)?;

    utils::traverse(&source_json, &mut target_json, None, 0, TARGET_LANG).await;

    let mut target_file = File::create("assets/ua.json")?;
    target_file.write_all(serde_json::to_string_pretty(&target_json)?.as_bytes())?;

    Ok(())
}
