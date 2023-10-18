use overlay::OverlayTarget;

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .parse_default_env()
        .init();

    log::info!("Initialize overlay");
    let overlay = overlay::init(&overlay::OverlayOptions {
        title: "Task Manager Overlay".to_string(),
        target: OverlayTarget::WindowTitle("Task Manager".into()),
        font_init: None,
    })?;
    let mut text_input = Default::default();
    overlay.main_loop(
        |controller| {
            controller.toggle_debug_overlay(true);
            true
        },
        move |ui| {
            ui.window("Dummy Window")
                .resizable(true)
                .movable(true)
                .build(|| {
                    ui.text("Taskmanager Overlay!");
                    ui.text(format!("FPS: {:.2}", ui.io().framerate));
                    ui.input_text("Test-Input", &mut text_input).build();
                });
            true
        },
    );
}
