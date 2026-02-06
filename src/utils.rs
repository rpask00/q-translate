use serde_json::{json, Map, Value};
use crate::translate::translate_phrase;

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
pub async fn traverse(
    source: &Value,
    mut target: &mut Map<String, Value>,
    key: Option<&String>,
    index: usize,
    target_lang: &str,
) {
    match source {
        Value::Object(value) => {
            if let Some(key) = key {
                if target.get(key).is_none() {
                    target.insert(key.to_owned(), json!({}));
                }
                target = target.get_mut(key).unwrap().as_object_mut().unwrap()
            }

            for (i, (key, v)) in value.iter().enumerate() {
                Box::pin(traverse(v, &mut target, Some(key), i, target_lang)).await
            }
        }
        Value::String(value) => {
            let key = key.unwrap().to_owned();
            if target.get(&key).is_none() {
                let translated = translate_phrase(value, target_lang).await.unwrap();
                insert_at(target, index, key, json!(translated))
            }
        },
        other => {
            // if  Null, Bool, Number or Array - simply clone; 
            let key = key.unwrap().to_owned();
            if target.get(&key).is_none() {
                insert_at(target, index, key, other.to_owned())
            }
        },
    }
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
fn insert_at(map: &mut Map<String, Value>, index: usize, key: String, value: Value) {
    let old = std::mem::take(map);

    let mut entries: Vec<_> = old.into_iter().collect();
    entries.insert(index, (key, value));

    *map = entries.into_iter().collect();
}
