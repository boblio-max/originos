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

        // Real grid: 2 cols x 5 rows = 10 items (8 tiles + 2 footer).
        // Mirrors taskbar/systeminfo nav but tuned for 2-col layout:
        // left/right toggles column (XOR 1), up/down moves row (+/-2).
        // Accepts both new Key names and legacy private unicode for compat.
        match key {
            "\u{f702}" | "Left" => {
                // Left — toggle column within row
                self.selection ^= 1;
                window.set_qs_selection(self.selection as i32);
            }
            "\u{f703}" | "Right" => {
                // Right — toggle column within row
                self.selection ^= 1;
                window.set_qs_selection(self.selection as i32);
            }
            "\u{f700}" | "Up" => {
                // Up — previous row
                self.selection = (self.selection + 8) % 10;
                window.set_qs_selection(self.selection as i32);
            }
            "\u{f701}" | "Down" => {
                // Down — next row
                self.selection = (self.selection + 2) % 10;
                window.set_qs_selection(self.selection as i32);
            }
            "\n" | "\r" | "Return" | "Enter" => {
                let activated = self.selection;
                self.close(window);
                window.invoke_icon_activated(activated as i32);
            }
            "Escape" | "\u{001b}" => {
                self.close(window);
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
