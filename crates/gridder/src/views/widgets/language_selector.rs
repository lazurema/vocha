use std::sync::RwLock;

use eframe::egui;

use crate::{l10n::L10N, states::settings::Settings};

pub struct LanguageSelector;

impl LanguageSelector {
    pub fn language_selector(l: L10N, settings: impl AsRef<RwLock<Settings>>, ui: &mut egui::Ui) {
        egui::ComboBox::from_id_salt("language_selector")
            .selected_text(format!(
                "{} {}",
                egui_phosphor::regular::TRANSLATE,
                l.current_language().display_name()
            ))
            .show_ui(ui, |ui| {
                let mut new_language_code = l.current_language().code();
                for language_code in L10N::available_language_codes() {
                    if let Some(language) = l.get_language(language_code) {
                        ui.selectable_value(
                            &mut new_language_code,
                            language.code(),
                            language.display_name(),
                        );
                    }
                }
                if new_language_code != l.current_language().code() {
                    settings
                        .as_ref()
                        .write()
                        .expect("Failed to acquire write lock on settings.")
                        .set_current_language(new_language_code);
                }
            });
    }
}
