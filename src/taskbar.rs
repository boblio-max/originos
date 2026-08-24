use crate::MainWindow;
use crate::settingswindow::SettingsWindow;
use crate::systeminfo::SystemInfo;
use crate::window2::Window2;
use crate::window3::Window3;
use crate::window4::Window4;
use crate::window5::Window5;

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

    pub fn handle_key(&mut self, window: &MainWindow, key: &str, term_window: &mut SettingsWindow, term_window2: &mut SystemInfo, term_window3: &mut Window2, term_window4: &mut Window3, term_window5: &mut Window4, term_window6: &mut Window5) -> Option<usize> {
        if !self.active {
            return None;
        }

        let mut activated_index = None;
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
                match activated {
                    0 => {
                        term_window.close(window);
                        term_window2.close(window);
                        term_window3.close(window);
                        term_window4.close(window);
                        term_window5.close(window);
                        term_window6.close(window);
                    }
                    1 => term_window.toggle(window),
                    2 => term_window2.toggle(window),
                    3 => term_window3.toggle(window),
                    4 => term_window4.toggle(window),
                    5 => term_window5.toggle(window),
                    6 => term_window6.toggle(window),
                    _ => {}
                }
                if activated != 0 {
                    activated_index = Some(activated);
                }
            }
            _ => {
                self.close(window);
            }
        }

        activated_index
    }

    fn close(&mut self, window: &MainWindow) {
        self.active = false;
        window.set_dock_active(false);
        window.set_taskbar_active(false);
    }
}
