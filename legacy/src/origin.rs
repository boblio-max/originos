use crate::MainWindow;

/// Origin — Rust backend for `origin_circle.slint:1` (`OriginCircle`).
/// Mirrors `settingswindow.rs` / `systeminfo.rs` pattern: a small state struct
/// that owns the `origin_screen_toggle` visibility flag and exposes
/// `toggle`/`close` helpers that sync through `MainWindow`.
///
/// `OriginCircle` reuses the dock's `active`/`pill-active`/`selection` bindings
/// (`main.slint:213-221`), so this module only owns the dedicated
/// `origin-screen-active` screen. That keeps the circle itself passive
/// (rendered via `visible: root.pill-active`) and the centered 400x240
/// `origin_screen` rectangle (`origin_circle.slint:67`) driven from Rust.
pub struct Origin {
    screen_active: bool,
}

impl Origin {
    pub fn new() -> Self {
        Self {
            screen_active: false,
        }
    }

    pub fn screen_active(&self) -> bool {
        self.screen_active
    }

    /// Toggle the centered origin screen (`origin_circle.slint:67`).
    /// Linked via `main.slint:origin-screen-active <=> origin_circle.origin_screen_toggle`.
    pub fn toggle_screen(&mut self, window: &MainWindow) {
        self.screen_active = !self.screen_active;
        window.set_origin_screen_active(self.screen_active);
    }

    pub fn open_screen(&mut self, window: &MainWindow) {
        if !self.screen_active {
            self.toggle_screen(window);
        }
    }

    pub fn close_screen(&mut self, window: &MainWindow) {
        if self.screen_active {
            self.screen_active = false;
            window.set_origin_screen_active(false);
        }
    }

    /// Close is called from taskbar/dock dismiss or global close-all.
    pub fn close(&mut self, window: &MainWindow) {
        self.close_screen(window);
    }

    /// Keyboard helper — called from `main.rs` when origin is the focused surface.
    /// `Escape` or `o` closes; `Return` toggles. Returns true if handled.
    pub fn handle_key(&mut self, window: &MainWindow, key: &str) -> bool {
        if !self.screen_active {
            return false;
        }
        match key {
            "Escape" | "\u{001b}" | "o" | "O" => {
                self.close_screen(window);
                true
            }
            "Return" | "Enter" | "\n" | "\r" => {
                self.close_screen(window);
                true
            }
            _ => false,
        }
    }
}
