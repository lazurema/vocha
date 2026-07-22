use eframe::egui::{self, Widget as _};

use crate::{
    l10n::{L10N, Term},
    states::projects::ProjectPreviewState,
};

pub struct ProjectPreviewWidget;

impl ProjectPreviewWidget {
    pub fn ui(project: ProjectPreviewState, ui: &mut egui::Ui, l10n: &L10N) {
        egui::Label::new(l10n.tl(&Term::ProjectPreviewText {
            has_audio: project.audio_file_path().is_some(),
            has_textgrid: project.textgrid_file_path().is_some(),
        }))
        .selectable(false)
        .ui(ui);
    }
}
