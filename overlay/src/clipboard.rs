use copypasta::{
    ClipboardContext,
    ClipboardProvider,
};
use imgui::ClipboardBackend;

pub struct ClipboardSupport(pub ClipboardContext);
impl ClipboardBackend for ClipboardSupport {
    fn get(&mut self) -> Option<String> {
        self.0.get_contents().ok()
    }
    fn set(&mut self, text: &str) {
        if let Err(error) = self.0.set_contents(text.to_owned()) {
            log::warn!("Failed to set clipboard data: {}", error);
        }
    }
}
