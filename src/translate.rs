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

/// Translates a text string into the target language using Google Translate API.
///
/// This function sends a request to the Google Translate v2 API and returns
/// the translated text. The API key must be provided via the
/// `GOOGLE_TRANSLATE_API_KEY` environment variable (for example using a `.env` file).
///
/// # Arguments
///
/// * `text` - The text to translate
/// * `target_lang` - Target language code (e.g. `"en"`, `"de"`, `"pl"`)
///
/// # Returns
///
/// Returns the translated text on success.
///
/// # Errors
///
/// Returns an error if:
/// - The HTTP request fails
/// - The API responds with a non-success status
/// - The response body cannot be parsed
///
/// # Panics
///
/// Panics if the `GOOGLE_TRANSLATE_API_KEY` environment variable is not set.
///
/// # Examples
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let translated = translate_phrase("Hallo Welt", "en").await?;
/// assert_eq!(translated, "Hello world");
/// # Ok(())
/// # }
/// ```
pub async fn translate_phrase(
    text: &str,
    target_lang: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    dotenv().ok();

    let api_key = env!("GOOGLE_TRANSLATE_API_KEY");

    let client = Client::new();
    let url = "https://translation.googleapis.com/language/translate/v2";

    let response = client
        .post(url)
        .query(&[("key", api_key), ("q", text), ("target", target_lang)])
        .send()
        .await?
        .error_for_status()?
        .json::<TranslateResponse>()
        .await?;

    Ok(response.data.translations[0].translated_text.clone())
}
