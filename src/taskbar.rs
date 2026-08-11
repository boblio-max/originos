use crate::MainWindow;
use crate::SettingsWindow;

pub struct Taskbar {
    active: bool,
    selection: usize,
}


impl Taskbar {
    pub fn new() -> Self {
        Self {
            active: false,
            selection: 0,
        }
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn toggle(&mut self, window: &MainWindow) {
        self.active = !self.active;
        self.selection = 0;
        window.set_dock_active(self.active);
        window.set_taskbar_active(self.active);
        window.set_dock_selection(self.selection as i32);
    }

    pub fn handle_key(&mut self, window: &MainWindow, key: &str, term_window: &mut SettingsWindow) -> bool {
        if !self.active {
            return false;
        }

        match key {
            "\u{f702}" => {
                self.selection = (self.selection + 6) % 7;
                window.set_dock_selection(self.selection as i32);
            }
            "\u{f703}" => {
                self.selection = (self.selection + 1) % 7;
                window.set_dock_selection(self.selection as i32);
            }
            "\n" | "\r" => {
                let activated = self.selection;
                self.close(window);
                if activated == 1 {
                    term_window.toggle(window);
                } else {
                    window.invoke_icon_activated(activated as i32);
                }
            }
            _ => {
                self.close(window);
            }
        }

        true
    }

    fn close(&mut self, window: &MainWindow) {
        self.active = false;
        window.set_dock_active(false);
        window.set_taskbar_active(false);
    }
}
