//! TODO: do i18n properly.

use std::sync::{Arc, atomic::AtomicUsize};

#[derive(Clone)]
pub struct L10N(Arc<L10NImpl>);

struct L10NImpl {
    current_language_index: AtomicUsize,
    l: L,
}

struct L {
    eng: Box<dyn Language>,
}

impl L {
    fn new() -> Self {
        Self {
            eng: Box::new(English),
        }
    }
}

impl L10N {
    pub fn new() -> Self {
        Self {
            0: Arc::new(L10NImpl {
                current_language_index: AtomicUsize::new(0),
                l: L::new(),
            }),
        }
    }

    pub fn available_language_codes() -> &'static [&'static str] {
        &["eng"]
    }

    pub fn set_current_language(&mut self, language_code: &'static str) {
        if let Some(index) = Self::available_language_codes()
            .iter()
            .position(|&code| code == language_code)
        {
            self.0
                .current_language_index
                .store(index, std::sync::atomic::Ordering::SeqCst);
        }
    }

    pub fn tl(&self, term: &Term) -> String {
        match self.current_language_code() {
            "eng" => self.0.l.eng.tl(term),
            _ => self.0.l.eng.tl(term),
        }
    }

    pub fn get_language(&self, code: &'static str) -> Option<&Box<dyn Language>> {
        match code {
            "eng" => Some(&self.0.l.eng),
            _ => None,
        }
    }

    pub fn current_language(&self) -> &Box<dyn Language> {
        self.get_language(self.current_language_code()).unwrap()
    }

    fn current_language_code(&self) -> &'static str {
        Self::available_language_codes()[self
            .0
            .current_language_index
            .load(std::sync::atomic::Ordering::SeqCst)]
    }
}

pub enum Term {
    AlwaysOnTopToggleLabel,
    DropHintText {
        supported_audio_extensions: &'static [&'static str],
    },
    ProjectPreviewText {
        has_audio: bool,
        has_textgrid: bool,
    },
    LoadingThing {
        thing: &'static str,
        path: String,
    },
    FailedToLoadThing {
        thing: &'static str,
        path: String,
        error: String,
    },
}

pub trait Language: Send + Sync {
    fn code(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn tl(&self, term: &Term) -> String;
}

struct English;

impl Language for English {
    fn code(&self) -> &'static str {
        "eng"
    }

    fn display_name(&self) -> &'static str {
        "English"
    }

    fn tl(&self, term: &Term) -> String {
        use Term::*;

        match term {
            AlwaysOnTopToggleLabel => "Always on Top".to_string(),
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
        }
    }
}
