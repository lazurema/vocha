use crate::l10n::L10NMut;

pub struct Settings {
    pub l10n_mut: L10NMut,
    pub is_welcome_window_always_on_top: bool,
}

impl Settings {
    pub fn new(l10n_mut: L10NMut) -> Self {
        Self {
            l10n_mut,
            is_welcome_window_always_on_top: true,
        }
    }
}
