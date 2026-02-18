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

/// Translates a text strings into the target language using Google Translate API.
///
/// This function sends a request to the Google Translate v2 API and returns
/// the translated texts. The API key must be provided via the
/// `GOOGLE_TRANSLATE_API_KEY` environment variable (for example using a `.env` file).
///
/// # Arguments
///
/// * `texts` - Vector of texts to translate
/// * `target_lang` - Target language code (e.g. `"en"`, `"de"`, `"pl"`)
///
/// # Returns
///
/// Returns the translated texts on success.
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
/// let phrases = vec!["Hallo Welt"];
/// let translated = translate_phrases(phrases, "en").await?;
/// assert_eq!(translated, vec!["Hello world"]);
/// # Ok(())
/// # }
/// ```

pub async fn translate_phrases(
    texts: Vec<String>,
    target_lang: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    dotenv().ok();

    let api_key = env!("GOOGLE_TRANSLATE_API_KEY");
    let client = Client::new();
    let url = "https://translation.googleapis.com/language/translate/v2";

    let mut params = vec![
        ("key", api_key.to_string()),
        ("target", target_lang.to_string()),
    ];

    for text in texts {
        params.push(("q", text));
    }

    let response = client
        .post(url)
        .query(&params)
        .send()
        .await?
        .error_for_status()?
        .json::<TranslateResponse>()
        .await?;

    let translations = response.data.translations
        .into_iter()
        .map(|t| t.translated_text)
        .collect();

    Ok(translations)
}
