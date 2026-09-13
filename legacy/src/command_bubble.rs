use crate::MainWindow;

pub struct CommandBubble {
    active: bool,
}

impl CommandBubble {
    pub fn new() -> Self {
        Self { active: false }
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn toggle(&mut self, window: &MainWindow) {
        self.active = !self.active;
        window.set_shortcut_pressed(self.active);
        if !self.active {
            window.set_output_generated(false);
            window.set_bubble_gen_text(String::new().into());
        }
    }
}
