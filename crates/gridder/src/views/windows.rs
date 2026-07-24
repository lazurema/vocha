use std::sync::{Arc, atomic::AtomicU8};

pub mod about_window;
pub mod project_window;
pub mod settings_window;
pub mod welcome_window;

#[derive(Clone)]
pub struct SingletonWindowOpenState(Arc<AtomicU8>);

const SINGLETON_WINDOW_OPEN_STATE_CLOSED: u8 = 0;
const SINGLETON_WINDOW_OPEN_STATE_OPEN: u8 = 1;
const SINGLETON_WINDOW_OPEN_STATE_OPEN_DESIRING_FOCUS: u8 = 2;

impl SingletonWindowOpenState {
    pub fn new() -> Self {
        Self(Arc::new(AtomicU8::new(SINGLETON_WINDOW_OPEN_STATE_CLOSED)))
    }

    pub fn open(&self) {
        self.0.store(
            SINGLETON_WINDOW_OPEN_STATE_OPEN_DESIRING_FOCUS,
            std::sync::atomic::Ordering::Relaxed,
        );
    }
    pub fn close(&self) {
        self.0.store(
            SINGLETON_WINDOW_OPEN_STATE_CLOSED,
            std::sync::atomic::Ordering::Relaxed,
        );
    }

    pub fn is_open(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::Relaxed) != SINGLETON_WINDOW_OPEN_STATE_CLOSED
    }
    pub fn pop_is_desiring_focus(&self) -> bool {
        let is_desiring_focus = self.0.load(std::sync::atomic::Ordering::Relaxed)
            == SINGLETON_WINDOW_OPEN_STATE_OPEN_DESIRING_FOCUS;
        if is_desiring_focus {
            self.0.store(
                SINGLETON_WINDOW_OPEN_STATE_OPEN,
                std::sync::atomic::Ordering::Relaxed,
            );
        }
        is_desiring_focus
    }
}
