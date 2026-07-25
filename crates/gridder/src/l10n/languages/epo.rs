use crate::l10n::{Language, Term};

pub struct Esperanto;

impl Language for Esperanto {
    fn code(&self) -> &'static str {
        "epo"
    }

    fn display_name(&self) -> &'static str {
        "Esperanto"
    }

    fn tl(&self, term: &Term) -> String {
        use Term::*;

        match term {
            About => "Pri".to_string(),
            Settings => "Agordoj".to_string(),
            Appearance => "Aspekto".to_string(),
            DropHintText {
                supported_audio_extensions,
            } => {
                let supported_text = supported_audio_extensions
                    .iter()
                    .map(|ext| format!(".{}", ext))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "Demetu sondosieron ({}) kaj/aŭ TextGrid-dosieron ĉi tien.",
                    supported_text
                )
            }
            ProjectPreviewText {
                has_audio,
                has_textgrid,
            } => {
                let desc = match (has_audio, has_textgrid) {
                    (true, true) => "sondosieron kaj TextGrid-dosieron",
                    (true, false) => "sondosieron",
                    (false, true) => "TextGrid-dosieron",
                    (false, false) => "nenion",
                };

                format!("Projekto de {} malfermiĝos.", desc)
            }
            LoadingThing { thing, path } => format!("Ŝargante {}: {}", thing, path),
            FailedToLoadThing { thing, path, error } => {
                format!("malsukcesis ŝargi {}: {}\n{}", thing, path, error)
            }
            GridderProject { name } => match name {
                Some(name) => format!("Projekto de Gridder - {}", name),
                None => "Projekto de Gridder".to_owned(),
            },
            Language => "Lingvo".to_owned(),
            Theme => "Etoso".to_owned(),
        }
    }
}
