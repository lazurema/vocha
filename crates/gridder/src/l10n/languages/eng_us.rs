use crate::l10n::{Language, Term};

pub struct English;

impl Language for English {
    fn code(&self) -> &'static str {
        "eng-US"
    }

    fn display_name(&self) -> &'static str {
        "English (American)"
    }

    fn tl(&self, term: &Term) -> String {
        use Term::*;

        match term {
            About => "About".to_string(),
            Settings => "Settings".to_string(),
            Appearance => "Appearance".to_string(),
            DropHintText {
                supported_audio_extensions,
            } => {
                let supported_text = supported_audio_extensions
                    .iter()
                    .map(|ext| format!(".{}", ext))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "Drop an audio file ({}) and/or a TextGrid file here.",
                    supported_text
                )
            }
            ProjectPreviewText {
                has_audio,
                has_textgrid,
            } => {
                let desc = match (has_audio, has_textgrid) {
                    (true, true) => "an audio file and a TextGrid file",
                    (true, false) => "an audio file",
                    (false, true) => "a TextGrid file",
                    (false, false) => "nothing",
                };

                format!("A project of {} will be opened.", desc)
            }
            LoadingThing { thing, path } => format!("Loading {}: {}", thing, path),
            FailedToLoadThing { thing, path, error } => {
                format!("Failed to load {}: {}\n{}", thing, path, error)
            }
            GridderProject { name } => match name {
                Some(name) => format!("Gridder Project - {}", name),
                None => "Gridder Project".to_owned(),
            },
            Theme => "Theme".to_owned(),
        }
    }
}
