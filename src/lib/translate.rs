use dotenv::dotenv;
use reqwest::Client;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
struct TranslateResponse {
    data: TranslateData,
}

#[derive(Debug, Deserialize)]
struct TranslateData {
    translations: Vec<Translation>,
}

#[derive(Debug, Deserialize)]
struct Translation {
    #[serde(rename = "translatedText")]
    translated_text: String,
}

/// Translate given phrase to target language using Google API.
pub async fn translate_phrase(
    text: &str,
    target_lang: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    dotenv().ok();

    let api_key = env::var("GOOGLE_TRANSLATE_API_KEY").expect("GOOGLE_TRANSLATE_API_KEY not provided in .env file.");

    let client = Client::new();
    let url = "https://translation.googleapis.com/language/translate/v2";

    let response = client
        .post(url)
        .query(&[
            ("key", api_key.as_str()),
            ("q", text),
            ("target", target_lang),
        ])
        .send()
        .await?
        .error_for_status()?
        .json::<TranslateResponse>()
        .await?;

    Ok(response.data.translations[0].translated_text.clone())
}
