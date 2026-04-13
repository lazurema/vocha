//! Based on <https://github.com/woelper/oculante/blob/9bf06c339f215e277e71df9a75041dde75a2e9c1/src/ui/theme.rs>
//! from <https://github.com/woelper/oculante>. Modifications have been made.
//!
//! See: https://github.com/emilk/egui/discussions/1344#discussioncomment-11919481
//!
//! ```license
//! MIT License
//!
//! Copyright (c) 2020 Johann Woelper
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.
//! ```

use std::{fmt::Display, sync::Arc};

use eframe::egui::{FontData, FontDefinitions, FontFamily};
use font_kit::{
    family_name::FamilyName, handle::Handle, properties::Properties, source::SystemSource,
};

/// Attempt to load a system font by any of the given `family_names`, returning the first match.
fn load_font_family(family_names: &[&str]) -> Option<Vec<u8>> {
    let system_source = SystemSource::new();
    for &name in family_names {
        let font_handle = system_source
            .select_best_match(&[FamilyName::Title(name.to_string())], &Properties::new());
        match font_handle {
            Ok(h) => match &h {
                Handle::Memory { bytes, .. } => {
                    tracing::info!("Loaded {name} from memory.");
                    return Some(bytes.to_vec());
                }
                Handle::Path { path, .. } => {
                    tracing::info!("Loaded {name} from path: {:?}", path);
                    if let Ok(data) = std::fs::read(path) {
                        return Some(data);
                    }
                }
            },
            Err(e) => tracing::debug!("Could not load {}: {:?}", name, e),
        }
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum LanguageCode {
    ZhHans,
    ZhHantTW,
    ZhHantHK,
    Ja,
    Ko,
    Ar,
}

impl Display for LanguageCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LanguageCode::ZhHans => write!(f, "zh-Hans"),
            LanguageCode::ZhHantTW => write!(f, "zh-Hant-TW"),
            LanguageCode::ZhHantHK => write!(f, "zh-Hant-HK"),
            LanguageCode::Ja => write!(f, "ja"),
            LanguageCode::Ko => write!(f, "ko"),
            LanguageCode::Ar => write!(f, "ar"),
        }
    }
}

impl LanguageCode {
    fn all() -> Vec<Self> {
        vec![
            Self::ZhHans,
            Self::ZhHantTW,
            Self::ZhHantHK,
            Self::Ja,
            Self::Ko,
            Self::Ar,
        ]
    }

    fn preferred_font_names(&self) -> Vec<&'static str> {
        match self {
            LanguageCode::ZhHans => vec![
                // builtin
                "PingFang SC",        // macOS
                "Microsoft YaHei UI", // Windows
                // Noto Sans and Source Sans
                "Noto Sans CJK SC", // Good coverage for Simplified Chinese
                "Noto Sans SC",
                "Source Han Sans CN",
                // Other fonts
                "WenQuanYi Zen Hei", // Includes both Simplified and Traditional Chinese.
                "SimSun",
            ],
            LanguageCode::ZhHantTW => vec![
                // builtin
                "PingFang TC",           // macOS
                "Microsoft JhengHei UI", // Windows
                // Noto Sans and Source Sans
                "Noto Sans CJK TW",
                "Noto Sans TW",
                "Source Han Sans TW",
            ],
            LanguageCode::ZhHantHK => vec![
                // builtin
                "PingFang HK", // macOS
                // Noto Sans and Source Sans
                "Noto Sans CJK HK",
                "Noto Sans HK",
                "Source Han Sans HK",
            ],
            LanguageCode::Ja => vec![
                // builtin
                "Hiragino Sans", // macOS
                "Yu Gothic UI",  // Windows
                // Noto Sans and Source Sans
                "Noto Sans JP",
                "Noto Sans CJK JP",
                "Source Han Sans JP",
                // Other fonts
                "MS Gothic",
            ],
            LanguageCode::Ko => vec![
                // builtin
                "Apple SD Gothic Neo", // macOS
                "Malgun Gothic",       // Windows
                // Noto Sans and Source Sans
                "Noto Sans KR",
                "Noto Sans CJK KR",
                "Source Han Sans KR",
            ],
            LanguageCode::Ar => vec![
                // Noto Sans
                "Noto Sans Arabic",
                // Other fonts
                "Amiri",
                "Lateef",
                "Al Tarikh",
                "Segoe UI",
            ],
        }
    }

    fn from_str_most_similar(str: &str) -> Option<Self> {
        let parts = str.split('-').collect::<Vec<_>>();
        if parts.is_empty() {
            return None;
        }

        if &parts == &["zh"]
            || &parts == &["zh", "CN"]
            || parts.contains(&"Hans")
            || (parts.contains(&"cmn") && !parts.contains(&"Hant"))
        {
            Some(Self::ZhHans)
        } else if parts[0] == "zh" {
            if parts.contains(&"HK") {
                Some(Self::ZhHantHK)
            } else {
                Some(Self::ZhHantTW)
            }
        } else if parts[0] == "ja" || parts.contains(&"JP") || parts.contains(&"jpn") {
            Some(Self::Ja)
        } else if parts[0] == "ko" || parts.contains(&"KR") || parts.contains(&"kor") {
            Some(Self::Ko)
        } else if parts[0] == "ar" || parts.contains(&"ara") {
            Some(Self::Ar)
        } else {
            None
        }
    }

    fn preferred_language_code_list(strs: &[String]) -> Vec<Self> {
        let mut codes = Vec::new();
        let mut remain = Self::all();
        let mut prefers_hant = false;

        for str in strs {
            if let Some(code) = Self::from_str_most_similar(&str) {
                if !codes.contains(&code) {
                    if code == Self::ZhHantHK || code == Self::ZhHantTW {
                        prefers_hant = true;
                    }
                    codes.push(code);
                }
                remain.retain(|c| c != &code);
            }
        }
        if prefers_hant {
            // If the user prefers any variant of Traditional Chinese,
            // prioritize all Traditional Chinese variants over Simplified
            // Chinese.
            for code in remain
                .iter()
                .filter(|c| **c == Self::ZhHantHK || **c == Self::ZhHantTW)
            {
                codes.push(*code);
            }
            remain.retain(|c| *c != Self::ZhHantHK && *c != Self::ZhHantTW);
        }
        codes.extend(remain);

        debug_assert_eq!(
            codes.iter().collect::<std::collections::HashSet<_>>(),
            Self::all().iter().collect()
        );

        codes
    }

    fn preferred_font_name_list(strs: &[String]) -> Vec<(LanguageCode, Vec<&'static str>)> {
        let mut font_names = Vec::new();
        for code in Self::preferred_language_code_list(strs) {
            font_names.push((code, code.preferred_font_names()));
        }
        font_names
    }
}

pub fn load_system_fonts(mut fonts: FontDefinitions) -> FontDefinitions {
    tracing::debug!("Attempting to load fonts");

    let font_name_list = LanguageCode::preferred_font_name_list(&get_user_system_language_codes());

    // The current implementation is slow (at least) on macOS, so let's just
    // load a single font that supports Chinese characters.
    let mut has_loaded_font_supporting_chinese_characters = false;

    for (code, font_names) in font_name_list {
        tracing::info!("Inserting font for language code `{code}`");
        let supports_chinese_characters = matches!(
            code,
            LanguageCode::ZhHans
                | LanguageCode::ZhHantTW
                | LanguageCode::ZhHantHK
                | LanguageCode::Ja
        );
        if supports_chinese_characters {
            if has_loaded_font_supporting_chinese_characters {
                tracing::info!(
                    "Already loaded a font supporting Chinese characters, skipping additional font for `{code}`"
                );
                continue;
            }
        }

        if let Some(font_data) = load_font_family(&font_names) {
            fonts
                .font_data
                .insert(code.to_string(), Arc::new(FontData::from_owned(font_data)));

            fonts
                .families
                .get_mut(&FontFamily::Proportional)
                .unwrap()
                .push(code.to_string());
            if supports_chinese_characters {
                has_loaded_font_supporting_chinese_characters = true;
            }
        } else {
            tracing::warn!(
                "Could not load a font for language code `{code}`. If you experience incorrect file names, try installing one of these fonts: [{}]",
                font_names.join(", ")
            )
        }
    }
    fonts
}

#[cfg(target_os = "macos")]
fn get_user_system_language_codes() -> Vec<String> {
    objc2_foundation::NSLocale::preferredLanguages()
        .iter()
        .map(|s| s.to_string())
        .collect()
}

#[cfg(target_os = "linux")]
fn get_user_system_language_codes() -> Vec<String> {
    fn parse_lc(lc: &str) -> Option<String> {
        let part_before_codeset = lc.split('.').next().unwrap();
        if part_before_codeset.is_empty() {
            return None;
        }
        return Some(part_before_codeset.replace('_', "-"));
    }

    if let Some(env_lc_all) = std::env::var_os("LC_ALL")
        && let Some(parsed) = parse_lc(&env_lc_all.to_string_lossy())
    {
        return vec![parsed];
    }

    let mut result: Vec<String> = vec![];

    if let Some(env_language) = std::env::var_os("LANGUAGE") {
        for lang in env_language.to_string_lossy().split(':') {
            if let Some(parsed) = parse_lc(lang) {
                result.push(parsed);
            }
        }
    }

    if let Some(env_lc_message) = std::env::var_os("LC_MESSAGES")
        && let Some(parsed) = parse_lc(&env_lc_message.to_string_lossy())
    {
        result.push(parsed);
    }

    if let Some(env_lang) = std::env::var_os("LANG")
        && let Some(parsed) = parse_lc(&env_lang.to_string_lossy())
    {
        result.push(parsed);
    }

    result
}

#[cfg(target_os = "windows")]
fn get_user_system_language_codes() -> Vec<String> {
    windows::Globalization::ApplicationLanguages::Languages()
        .unwrap()
        .into_iter()
        .map(|l| l.to_string())
        .collect()
}
