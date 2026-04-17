// Copyright (C) 2026 Raimo Geisel
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Internationalisation (i18n) support.
//!
//! Provides [`detect_lang_code`] to auto-detect the user's locale and
//! [`get_translations`] to load the matching [`Translations`] struct.
//! All supported locales are embedded at compile time from `locales/<code>.toml`.
//! Adding a new language only requires a TOML file and a new entry in [`LOCALES`].
//!
//! Language detection priority (highest first):
//! 1. `--lang` CLI flag
//! 2. `LANGUAGE` environment variable (colon-separated list, GNU extension)
//! 3. `LC_ALL` environment variable
//! 4. `LC_MESSAGES` environment variable
//! 5. `LANG` environment variable
//! 6. System locale API via `sys-locale` (covers Windows native + macOS outside a shell)
//! 7. English fallback (`"en"`)

use std::env;

/// All locale TOML files compiled into the binary.
///
/// Each entry is `(ISO-639-1 code, TOML source)`. To add a language: create
/// `locales/<code>.toml` and add an entry here — no other code changes needed.
static LOCALES: &[(&str, &str)] = &[
    ("en", include_str!("../locales/en.toml")),
    ("de", include_str!("../locales/de.toml")),
    ("fr", include_str!("../locales/fr.toml")),
    ("es", include_str!("../locales/es.toml")),
];

/// Extracts the primary language subtag from a locale string.
/// Handles both POSIX-style (`"de_DE.UTF-8"`) and BCP 47 (`"de-DE"`) formats.
/// `"de_DE.UTF-8"` → `"de"`, `"de-DE"` → `"de"`, `"fr"` → `"fr"`.
pub(crate) fn primary_code(locale: &str) -> &str {
    locale.split(['-', '_', '.', '@']).next().unwrap_or(locale)
}

/// Returns `true` if `code` matches a locale compiled into this binary.
pub(crate) fn is_bundled(code: &str) -> bool {
    LOCALES.iter().any(|(c, _)| c.eq_ignore_ascii_case(code))
}

/// Returns the ISO 639-1 language code for the current process environment.
///
/// Detection priority (highest first):
/// 1. `LANGUAGE` environment variable (colon-separated list, GNU extension)
/// 2. `LC_ALL`
/// 3. `LC_MESSAGES`
/// 4. `LANG`
/// 5. Native OS locale API via `sys-locale`
/// 6. `"en"` final fallback
pub fn detect_lang_code() -> String {
    // LANGUAGE is a colon-separated preference list (GNU extension).
    // Iterate and return the first code that is bundled in this binary.
    if let Ok(val) = env::var("LANGUAGE") {
        for part in val.split(':') {
            let code = primary_code(part);
            if is_bundled(code) {
                return code.to_ascii_lowercase();
            }
        }
    }

    // Single-valued POSIX locale variables.
    for var in &["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(val) = env::var(var) {
            if !val.is_empty() && val != "C" && val != "POSIX" {
                return primary_code(&val).to_ascii_lowercase();
            }
        }
    }

    // Native OS locale API (Windows: GetUserDefaultLocaleName, macOS: CFLocale).
    if let Some(locale) = sys_locale::get_locale() {
        return primary_code(&locale).to_ascii_lowercase();
    }

    "en".to_string()
}

/// All translatable UI strings for a single language.
///
/// Deserialized at startup from the matching file in `locales/`.
/// Fields use `String` so the values are owned and can be freely used
/// throughout the UI without lifetime annotations.
#[derive(serde::Deserialize)]
pub struct Translations {
    // ── Tab bar ──────────────────────────────────────────────────────────────
    pub tab_overview: String,
    pub tab_cpu: String,
    pub tab_memory: String,
    pub tab_network: String,
    pub tab_disks: String,
    pub tab_processes: String,
    pub tab_gpu: String,
    pub menu_title: String,

    // ── Status bar ───────────────────────────────────────────────────────────
    pub nav_right: String,
    pub nav_left: String,
    pub jump_to_tab: String,
    pub quit: String,
    pub scroll_up: String,
    pub scroll_down: String,

    // ── Common time axis labels ───────────────────────────────────────────────
    pub ago_60s: String,
    pub ago_30s: String,
    pub now: String,

    // ── CPU tab ───────────────────────────────────────────────────────────────
    pub cpu_history_title: String,
    pub cpu_dataset_label: String,

    // ── Memory tab ────────────────────────────────────────────────────────────
    pub mem_history_title: String,
    pub mem_dataset_label: String,
    pub ram: String,
    pub swap: String,

    // ── Network tab ───────────────────────────────────────────────────────────
    pub net_history_title: String,
    pub net_throughput_title: String,

    // ── Disks tab ─────────────────────────────────────────────────────────────
    /// Base title for the Disk Usage block (scroll indicator appended at runtime).
    pub disk_usage_title: String,

    // ── Processes tab ─────────────────────────────────────────────────────────
    /// Base title for the Processes block (scroll indicator appended at runtime).
    pub processes_title: String,
    pub col_pid: String,
    pub col_name: String,
    pub col_cpu_pct: String,
    pub col_mem_kib: String,

    // ── GPU tab ───────────────────────────────────────────────────────────────
    pub gpu_no_device: String,
    pub gpu_util: String,
    pub gpu_vram: String,
    pub gpu_gtt: String,
    pub gpu_stats: String,
    pub gpu_temp: String,
    pub gpu_power: String,
    pub na: String,
}

/// Returns the [`Translations`] for `lang_code`.
///
/// `lang_code` may contain region/encoding info (`"de_DE.UTF-8"`); only the
/// primary subtag is used. Falls back to English for unknown codes.
///
/// Locale files are embedded at compile time — no runtime file I/O.
///
/// # Panics
/// Panics if a bundled locale file is invalid TOML — caught in development.
pub fn get_translations(lang_code: &str) -> Translations {
    let code = primary_code(lang_code).to_ascii_lowercase();
    let src = LOCALES
        .iter()
        .find(|(c, _)| *c == code.as_str())
        .map(|(_, s)| *s)
        .unwrap_or(LOCALES[0].1); // fallback to "en"
    toml::from_str(src).expect("bundled locale TOML is invalid")
}

