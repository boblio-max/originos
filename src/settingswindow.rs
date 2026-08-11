use crate::MainWindow;

pub struct SettingsWindow {
    active: bool,
}

impl SettingsWindow {
    pub fn new() -> Self {
        Self { active: false }
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn toggle(&mut self, window: &MainWindow) {
        self.active = !self.active;
        window.set_settings_window_active(self.active);
    }
}