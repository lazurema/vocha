//! TODO: do i18n properly.

pub mod languages;

use std::sync::{Arc, atomic::AtomicUsize};

struct L {
    cmn_hans: Box<dyn Language>,
    eng_us: Box<dyn Language>,
    epo: Box<dyn Language>,
}

impl L {
    fn new() -> Self {
        Self {
            cmn_hans: Box::new(languages::cmn_hans::ChineseMandarinSimplified),
            eng_us: Box::new(languages::eng_us::English),
            epo: Box::new(languages::epo::Esperanto),
        }
    }
}

#[derive(Clone)]
pub struct L10N(Arc<L10NImpl>);

struct L10NImpl {
    current_language_index: AtomicUsize,
    l: L,
}

impl L10N {
    fn new() -> Self {
        Self {
            0: Arc::new(L10NImpl {
                current_language_index: AtomicUsize::new(
                    Self::available_language_codes()
                        .iter()
                        .position(|&code| code == "eng-US")
                        .expect("`eng-US` should always be available."),
                ),
                l: L::new(),
            }),
        }
    }

    pub fn available_language_codes() -> &'static [&'static str] {
        &["cmn-Hans", "eng-US", "epo"]
    }

    pub fn tl(&self, term: &Term) -> String {
        match self.current_language_code() {
            "cmn-Hans" => self.0.l.cmn_hans.tl(term),
            "eng-US" => self.0.l.eng_us.tl(term),
            "epo" => self.0.l.epo.tl(term),
            _ => self.0.l.eng_us.tl(term),
        }
    }

    pub fn get_language(&self, code: &'static str) -> Option<&Box<dyn Language>> {
        match code {
            "cmn-Hans" => Some(&self.0.l.cmn_hans),
            "eng-US" => Some(&self.0.l.eng_us),
            "epo" => Some(&self.0.l.epo),
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

#[derive(Clone)]
pub struct L10NMut(L10N);

impl L10NMut {
    pub fn new() -> Self {
        Self(L10N::new())
    }

    pub fn inner(&self) -> &L10N {
        &self.0
    }

    pub fn set_current_language(&mut self, language_code: &'static str) {
        if let Some(index) = L10N::available_language_codes()
            .iter()
            .position(|&code| code == language_code)
        {
            self.0
                .0
                .current_language_index
                .store(index, std::sync::atomic::Ordering::SeqCst);
        }
    }
}

pub enum Term {
    About,
    Settings,
    Appearance,
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
    GridderProject {
        name: Option<String>,
    },
    Theme,
}

pub trait Language: Send + Sync {
    fn code(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn tl(&self, term: &Term) -> String;
}
