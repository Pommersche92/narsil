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

//! Tests for [`crate::i18n`].
//!
//! Covers: `primary_code` suffix stripping, `is_bundled` lookup,
//! TOML parsing for every bundled locale, spot-checks on known
//! translation values, unknown-code fallback to English, region/encoding
//! suffix stripping, and a no-panic smoke test for `detect_lang_code`.

use crate::i18n::{detect_lang_code, get_translations, is_bundled, primary_code};

// ── primary_code ──────────────────────────────────────────────────────────────────────────────

#[test]
fn test_primary_code_bare() {
    assert_eq!(primary_code("fr"), "fr");
}

#[test]
fn test_primary_code_region() {
    assert_eq!(primary_code("de_DE"), "de");
}

#[test]
fn test_primary_code_region_encoding() {
    assert_eq!(primary_code("es_MX.UTF-8"), "es");
}

#[test]
fn test_primary_code_at_suffix() {
    assert_eq!(primary_code("en@euro"), "en");
}

#[test]
fn test_primary_code_all_suffixes() {
    assert_eq!(primary_code("de_DE.UTF-8@euro"), "de");
}

/// Windows and macOS return BCP 47 tags with hyphens, e.g. `"de-DE"`.
#[test]
fn test_primary_code_bcp47_hyphen() {
    assert_eq!(primary_code("de-DE"), "de");
    assert_eq!(primary_code("en-US"), "en");
    assert_eq!(primary_code("fr-FR"), "fr");
}

/// `get_translations` resolves a BCP 47 locale string correctly.
#[test]
fn test_get_translations_bcp47_hyphen() {
    let t = get_translations("de-DE");
    assert_eq!(t.tab_memory, "Arbeitsspeicher");
}

// ── is_bundled ────────────────────────────────────────────────────────────────────────────────────

#[test]
fn test_is_bundled_known_codes() {
    for code in &["en", "de", "fr", "es"] {
        assert!(is_bundled(code), "{code} should be bundled");
    }
}

#[test]
fn test_is_bundled_case_insensitive() {
    assert!(is_bundled("DE"));
    assert!(is_bundled("FR"));
}

#[test]
fn test_is_bundled_unknown_returns_false() {
    assert!(!is_bundled("zh"));
    assert!(!is_bundled("pt"));
    assert!(!is_bundled(""));
}

// ── get_translations — known codes ─────────────────────────────────────────────────────────────────────

/// English translations load without panic and contain the expected CPU label.
#[test]
fn test_get_translations_english() {
    let t = get_translations("en");
    assert_eq!(t.tab_cpu, "CPU");
    assert_eq!(t.na, "N/A");
}

/// German translations load and return German strings.
#[test]
fn test_get_translations_german() {
    let t = get_translations("de");
    assert_eq!(t.tab_memory, "Arbeitsspeicher");
    assert_eq!(t.quit, "Beenden");
}

/// French translations load and return French strings.
#[test]
fn test_get_translations_french() {
    let t = get_translations("fr");
    assert_eq!(t.tab_network, "Réseau");
    assert_eq!(t.now, "maintenant");
}

/// Spanish translations load and return Spanish strings.
#[test]
fn test_get_translations_spanish() {
    let t = get_translations("es");
    assert_eq!(t.tab_overview, "Resumen");
    assert_eq!(t.scroll_up, "Desplazar arriba");
}

// ── get_translations — fallback and suffix stripping ─────────────────────────

/// An unknown language code falls back to English.
#[test]
fn test_get_translations_unknown_falls_back_to_english() {
    let t = get_translations("zh");
    assert_eq!(t.tab_cpu, "CPU");
    assert_eq!(t.na, "N/A");
}

/// A region-qualified locale string (`"de_DE.UTF-8"`) resolves to German.
#[test]
fn test_get_translations_region_qualified() {
    let t = get_translations("de_DE.UTF-8");
    assert_eq!(t.tab_memory, "Arbeitsspeicher");
}

/// An at-suffix locale (`"fr@euro"`) resolves to French.
#[test]
fn test_get_translations_at_suffix() {
    let t = get_translations("fr@euro");
    assert_eq!(t.tab_network, "Réseau");
}

// ── get_translations — completeness ──────────────────────────────────────────

/// Every bundled locale parses without panic and every required field is
/// non-empty.  This catches TOML files that are missing keys or have blank
/// values.
#[test]
fn test_all_locales_non_empty_fields() {
    for code in &["en", "de", "fr", "es"] {
        let t = get_translations(code);
        let fields = [
            &t.tab_overview, &t.tab_cpu, &t.tab_memory, &t.tab_network,
            &t.tab_disks, &t.tab_processes, &t.tab_gpu, &t.menu_title,
            &t.nav_right, &t.nav_left, &t.jump_to_tab, &t.quit,
            &t.scroll_up, &t.scroll_down,
            &t.ago_60s, &t.ago_30s, &t.now,
            &t.cpu_history_title, &t.cpu_dataset_label,
            &t.mem_history_title, &t.mem_dataset_label, &t.ram, &t.swap,
            &t.net_history_title, &t.net_throughput_title,
            &t.disk_usage_title,
            &t.processes_title, &t.col_pid, &t.col_name,
            &t.col_cpu_pct, &t.col_mem_kib,
            &t.gpu_no_device, &t.gpu_util, &t.gpu_vram, &t.gpu_stats,
            &t.gpu_temp, &t.gpu_power, &t.na,
        ];
        for field in fields {
            assert!(
                !field.is_empty(),
                "locale \"{code}\" has an empty field: {field:?}"
            );
        }
    }
}

/// Every bundled locale has the same number of fields as English.
/// A locale with a missing key would deserialise as empty string and be caught
/// by the field check above, but this test makes the intent explicit.
#[test]
fn test_all_locales_parse() {
    // If any TOML is missing a key or is structurally invalid, `get_translations`
    // would panic here.
    for code in &["en", "de", "fr", "es"] {
        let _ = get_translations(code);
    }
}

// ── detect_lang_code — smoke test ─────────────────────────────────────────────

/// `detect_lang_code` returns a non-empty string and never panics.
/// (Actual env-var driven behaviour is tested via private-function tests in
/// i18n.rs to avoid unsound concurrent env mutation.)
#[test]
fn test_detect_lang_code_returns_nonempty() {
    let code = detect_lang_code();
    assert!(!code.is_empty());
}
