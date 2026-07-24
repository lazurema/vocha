use crate::l10n::L10NMut;

pub struct Settings {
    pub l10n_mut: L10NMut,
}

impl Settings {
    pub fn new(l10n_mut: L10NMut) -> Self {
        Self { l10n_mut }
    }
}
