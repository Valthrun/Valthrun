use imgui::Ui;

pub trait KeyboardInput {
    fn is_key_down(&self, key: imgui::Key) -> bool;
    fn is_key_pressed(&self, key: imgui::Key, repeating: bool) -> bool;
}

impl KeyboardInput for Ui {
    fn is_key_down(&self, key: imgui::Key) -> bool {
        Ui::is_key_down(self, key)
    }

    fn is_key_pressed(&self, key: imgui::Key, repeating: bool) -> bool {
        if repeating {
            Ui::is_key_pressed(self, key)
        } else {
            Ui::is_key_pressed_no_repeat(self, key)
        }
    }
}
