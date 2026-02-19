use crate::translate::translate_phrases;
use serde_json::{Map, Value, json};
use std::cmp::min;
use std::collections::HashMap;

/// Recursively walks a JSON value and builds a translated target structure.
///
/// This function traverses `source` depth-first and mirrors its structure into
/// `target`. When a string value is encountered, it is translated into
/// `target_lang` and inserted at the same logical position. Non-string primitive
/// values (`Null`, `Bool`, `Number`, `Array`) are cloned without modification.
///
/// Object insertion order is preserved by inserting entries at the provided
/// `index`.
///
/// # Arguments
///
/// * `source` - The source JSON value to traverse
/// * `target` - The target JSON object being constructed
/// * `key` - The key under which the current value should be inserted
/// * `index` - Position at which the value should be inserted in the target object
/// * `target_lang` - Target language code used for string translation
/// * `translations` - HashMap with translated phrases
///
/// # Panics
///
/// Panics if:
/// - `key` is `None` when a non-root value is processed
/// - The target JSON structure does not match expected object layouts
/// - Translation fails (errors from `translate_phrase` are unwrapped)
///
/// # Async behavior
///
/// This function performs asynchronous network calls when translating string
/// values. Recursive calls are explicitly boxed to allow async recursion.
pub fn apply_translations(
    source: &Value,
    mut target: &mut Map<String, Value>,
    key: &String,
    index: usize,
    target_lang: &str,
    translations: &HashMap<String, String>,
) {
    match source {
        Value::Object(value) => {
            target = extract_or_instantiate_object_under_key(target, key);

            for (i, (key, v)) in value.iter().enumerate() {
                apply_translations(v, &mut target, key, i, target_lang, translations)
            }
        }
        Value::String(value) => {
            if target.get(key).is_none() {
                let translated = translations
                    .get(value)
                    .expect(format!("Translation for phrase {}, not found!", value).as_str());

                insert_at(target, index, key, json!(translated))
            }
        }
        other => {
            // if  Null, Bool, Number or Array - simply clone;
            if target.get(key).is_none() {
                insert_at(target, index, key, other.to_owned())
            }
        }
    }
}

/// Recursively traverses a source JSON structure and collects translation
/// entries for all string values.
///
/// The function mirrors the object structure from `source` into `target`,
/// creating missing intermediate objects as needed.
///
/// For each string in `source`:
/// - If a corresponding value exists in `target`, it is inserted into
///   `translations`.
/// - Otherwise, an empty string is inserted as a placeholder.
///
/// Non-object and non-string values are ignored.
///
/// # Panics
/// May panic if `key` is `None` when processing a string
/// or if the target structure is not an object where expected.
pub fn gather_translations(
    source: &Value,
    mut target: &mut Map<String, Value>,
    key: &String,
    target_lang: &str,
    translations: &mut HashMap<String, String>,
) {
    match source {
        Value::Object(value) => {
            target = extract_or_instantiate_object_under_key(target, key);

            for (key, v) in value.iter() {
                gather_translations(v, &mut target, key, target_lang, translations)
            }
        }
        Value::String(value) => match target.get(key) {
            None => {
                translations.insert(value.clone(), String::default());
            }
            Some(target_value) => {
                translations.insert(value.clone(), target_value.to_string());
            }
        },
        _ => {}
    }
}

/// Translates all missing entries in the provided `translations` map.
///
/// Collects phrases whose translation value is empty (`""`), sends them
/// in batches to `translate_phrases`, and updates the map with the
/// returned translations.
///
/// Translations are processed in batches of 128 for the most effective API usage.
///
/// # Errors
/// Returns an error if the underlying translation request fails.
///
/// # Behavior
/// - Only entries with empty values are translated.
/// - The `translations` map is updated in place.
/// - Already translated entries are skipped.
pub async fn perform_translations(
    translations: &mut HashMap<String, String>,
    target_lang: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut phrases = vec![];

    for (phrase, translated_phrase) in translations.iter() {
        if *translated_phrase == String::default() {
            phrases.push(phrase.to_owned());
        }
    }

    let batch_size = 128;
    while phrases.len() > 0 {
        let mut batch = phrases
            .splice(0..min(phrases.len(), batch_size), vec![])
            .collect();

        let mut translated = translate_phrases(&batch, target_lang).await?;

        for _ in 0..translated.len() {
            translations.insert(batch.pop().unwrap(), translated.pop().unwrap());
        }
    }

    Ok(())
}

/// Returns a mutable reference to a JSON object stored under the given `key`.
///
/// If the key does not exist in `target`, a new empty JSON object is inserted.
/// If the key exists but the value is not a JSON object, it will be replaced
/// with a new empty object.
///
/// If `key` is empty, the function returns `target` unchanged.
///
/// # Behavior
///
/// - Ensures that `target[key]` exists and is a JSON object.
/// - Overwrites non-object values under the key.
/// - Never returns `None`.
///
/// # Arguments
///
/// * `target` - The parent JSON object (`serde_json::Map`) to operate on.
/// * `key` - The key under which an object should exist.
///
/// # Returns
///
/// A mutable reference to the JSON object stored under `key`,
/// or to `target` itself if `key` is empty.
///
/// # Example
///
/// ```rust
/// use serde_json::{Map, Value};
///
/// let mut root = Map::new();
///
/// let child = extract_or_instantiate_object_under_key(
///     &mut root,
///     &"config".to_string(),
/// );
///
/// child.insert("enabled".to_string(), Value::Bool(true));
///
/// assert!(root["config"].is_object());
/// ```
fn extract_or_instantiate_object_under_key<'a>(
    target: &'a mut Map<String, Value>,
    key: &String,
) -> &'a mut Map<String, Value> {
    if key.is_empty() {
        return target;
    }

    let value = target
        .entry(key.to_owned())
        .or_insert_with(|| Value::Object(Map::new()));

    if !value.is_object() {
        *value = Value::Object(Map::new());
    }

    value.as_object_mut().unwrap()
}

/// Inserts a key–value pair into a [`serde_json::Map`] at the given position.
///
/// This function preserves insertion order by temporarily draining the map,
/// inserting the new entry at `index`, and rebuilding the map.
///
/// # Arguments
///
/// * `map` - The JSON map to modify
/// * `index` - Zero-based position at which to insert the new entry
/// * `key` - The key to insert
/// * `value` - The value to insert
///
/// # Panics
///
/// Panics if `index` is greater than the number of elements in the map.
///
/// # Examples
///
/// ```
/// use serde_json::{Map, Value};
///
/// let mut map = Map::new();
/// map.insert("a".to_string(), Value::from(1));
/// map.insert("b".to_string(), Value::from(2));
///
/// insert_at(&mut map, 1, "x".to_string(), Value::from(42));
///
/// let keys: Vec<_> = map.keys().cloned().collect();
/// assert_eq!(keys, vec!["a", "x", "b"]);
/// ```
fn insert_at(map: &mut Map<String, Value>, index: usize, key: &String, value: Value) {
    let old = std::mem::take(map);

    let mut entries: Vec<_> = old.into_iter().collect();
    entries.insert(index, (key.to_owned(), value));

    *map = entries.into_iter().collect();
}
