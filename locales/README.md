# Narsil — Translation Guide

Each `.toml` file in this directory contains the UI strings for one language.
The filename is the [ISO 639-1](https://en.wikipedia.org/wiki/List_of_ISO_639_language_codes)
two-letter language code (e.g. `de.toml` for German).

## Rules for translators

- **Do NOT change key names** (left side of `=`). Only translate values (right side).
- **Do NOT add or remove keys.** Every key in `en.toml` must be present in every other file.
- `\n` inside a value produces a newline in the UI.
- Keys marked *"appended automatically at runtime"* in their section comment have extra
  text (e.g. a scroll indicator) added by the application — just translate the base text.

## Adding a new language

1. Copy `en.toml` to `<code>.toml` (e.g. `pt.toml` for Portuguese).
2. Translate all values. Leave keys unchanged.
3. In `src/i18n.rs`, add one line to the `LOCALES` table:
   ```rust
   ("pt", include_str!("../locales/pt.toml")),
   ```
4. Run `cargo check` to verify the new file parses correctly.

No other code changes are needed. The binary automatically falls back to English
for any language code that does not have a matching `.toml` file.

## File format notes

- Plain [TOML](https://toml.io) key/value pairs — any text editor works.
- UTF-8 encoding required.
- Spacing around UI widgets (borders, labels) is handled by the application code,
  not the translation values. Keep values as plain text without leading/trailing spaces.
