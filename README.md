# i18n Translation Recreator

## Overview

This tool recreates an existing i18n translation file by automatically translating all text values into a chosen target language.

It reads a structured JSON file (for example, an i18n resource file), walks through it recursively, and produces a new file with the **same structure and key order**. All string values are translated into the target language, while non-string values (numbers, booleans, arrays, nulls) are preserved as-is.

The result is a ready-to-use translation file that mirrors the original exactly, but with all human-readable text translated.

---

## What this tool does

- Preserves the original JSON structure and nesting
- Keeps key insertion order intact
- Translates only string values
- Copies non-string values without modification
- Outputs a fully reconstructed file in the target language

---

## Typical use case

Use this tool when you already have a base i18n file (for example in English) and want to quickly generate a translated version (for example German or Polish) without manually translating each entry.

This is especially useful for:
- Bootstrapping new language files
- Keeping translations structurally consistent
- Reducing repetitive translation work

---

## Example

### Input (`en.json`)
```json
{
  "title": "Welcome",
  "menu": {
    "file": "File",
    "edit": "Edit"
  }
}
```

### Output (`de.json`)
```json
{
  "title": "Willkommen",
  "menu": {
    "file": "Datei",
    "edit": "Bearbeiten"
  }
}
```

---

## Requirements

- A valid Google Translate API key
- The API key must be provided via the `GOOGLE_TRANSLATE_API_KEY` environment variable (for example using a `.env` file)

---

## Important notes

- Translation quality depends on the external translation service
- The generated file is intended as a starting point
- Human review is recommended before using translations in production

---

## Intended audience

This tool is designed for developers and teams working with JSON-based i18n files who want to automate the creation of new language variants while preserving exact file structure and ordering.

