use crate::MainWindow;

pub struct Window5 {
    active: bool,
    x_translation: i32,
    y_translation: i32,
    x_scale: i32,
    y_scale: i32,
}

impl Window5 {
    pub fn new() -> Self {
        Self {
            active: false,
            x_translation: 0,
            y_translation: 0,
            x_scale: 0,
            y_scale: 0,
        }
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn close(&mut self, window: &MainWindow) {
        self.active = false;
        window.set_window_5_active(false);
    }

    pub fn toggle(&mut self, window: &MainWindow) {
        self.active = !self.active;
        window.set_window_5_active(self.active);
    }

    pub fn translate(&mut self, window: &MainWindow, x: i32, y: i32) {
        self.x_translation = x;
        self.y_translation = y;
        window.set_window_5_x_translation(x);
        window.set_window_5_y_translation(y);
    }

    pub fn scale(&mut self, window: &MainWindow, x: i32, y: i32) {
        self.x_scale = x;
        self.y_scale = y;
        window.set_window_5_x_scale(x);
        window.set_window_5_y_scale(y);
    }

}
