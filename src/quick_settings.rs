use crate::MainWindow;

pub struct QuickSettings {
    active: bool,
    selection: usize,
}

impl QuickSettings {
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
        window.set_qsettings_active(self.active);
        window.set_qs_active(self.active);
        window.set_qs_selection(self.selection as i32);
    }

    pub fn handle_key(&mut self, window: &MainWindow, key: &str) -> bool {
        if !self.active {
            return false;
        }

        match key {
            "\u{f702}" => {
                self.selection = (self.selection + 4) % 5;
                window.set_qs_selection(self.selection as i32);
            }
            "\u{f703}" => {
                self.selection = (self.selection + 1) % 5;
                window.set_qs_selection(self.selection as i32);
            }
            "\u{f700}" | "\u{f701}" => {
                self.selection = (self.selection + 5) % 10;
                window.set_qs_selection(self.selection as i32);
            }
            "\n" | "\r" => {
                let activated = self.selection;
                self.close(window);
                window.invoke_icon_activated(activated as i32);
            }
            _ => {
                self.close(window);
            }
        }

        true
    }

    fn close(&mut self, window: &MainWindow) {
        self.active = false;
        window.set_qsettings_active(false);
        window.set_qs_active(false);
    }
}
