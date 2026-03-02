use dotenv::dotenv;
use futures::stream::{self, Stream, StreamExt};
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
    phrases: &Vec<String>,
    target_lang: &str,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    dotenv().ok();

    let api_key = env!("GOOGLE_TRANSLATE_API_KEY");
    let client = Client::new();
    let url = "https://translation.googleapis.com/language/translate/v2";

    let mut params = vec![
        ("key", api_key.to_string()),
        ("target", target_lang.to_string()),
    ];

    for text in phrases {
        params.push(("q", text.to_owned()));
    }

    let response = client
        .post(url)
        .query(&params)
        .send()
        .await?
        .error_for_status()?
        .json::<TranslateResponse>()
        .await?;

    let translation_pairs: Vec<(String, String)> = phrases
        .iter()
        .cloned()
        .zip(
            response
                .data
                .translations
                .into_iter()
                .map(|t| t.translated_text),
        )
        .collect();

    Ok(translation_pairs)
}


/// Translates a collection of phrases into the target language using a concurrent stream.
///
/// This function optimizes API usage by:
/// * **Batching**: Grouping phrases into chunks of 128 (Google API limit).
/// * **Concurrency**: Executing up to 5 translation requests simultaneously.
/// * **Ordering**: Uses `buffer_unordered` for maximum throughput; results are emitted as soon as they are ready.
///
/// # Arguments
/// * `phrases` - A vector of strings to be translated.
/// * `target_lang` - Target language code (e.g., "en", "pl").
///
/// # Returns
/// A `Stream` of `(original, translated)` string pairs. If a batch fails,
/// the second element will contain `"Error"`.
pub fn translate_stream(
    phrases: Vec<String>,
    target_lang: String,
) -> impl Stream<Item = (String, String)> {
    let mut it = phrases.into_iter();

    let mut chunks = Vec::new();
    while it.as_slice().len() > 0 {
        let chunk: Vec<String> = it.by_ref().take(128).collect();
        chunks.push(chunk);
    }
    stream::iter(chunks)
        .map(move |chunk| {
            let lang = target_lang.clone();
            async move {
                translate_phrases(&chunk, &lang).await.unwrap_or_else(|_| {
                    chunk
                        .into_iter()
                        .map(|s| (s, "Error".to_string()))
                        .collect()
                })
            }
        })
        .buffer_unordered(5)
        .flat_map(stream::iter)
}
