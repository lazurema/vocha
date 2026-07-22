use crate::l10n::L10N;

pub struct Settings {
    /// TODO: `set_current_language` should only be allowed to be called from
    /// here.
    #[expect(dead_code)]
    l10n: L10N,
    pub is_welcome_window_always_on_top: bool,
}

impl Settings {
    pub fn new(l10n: L10N) -> Self {
        Self {
            l10n,
            is_welcome_window_always_on_top: true,
        }
    }

    #[expect(dead_code)]
    pub fn l10n(&self) -> &L10N {
        &self.l10n
    }
}
