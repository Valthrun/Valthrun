fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .parse_default_env()
        .init();

    log::info!("Initialize overlay");
    let overlay = overlay::init("Task Manager Overlay", "Task Manager")?;
    let mut text_input = Default::default();
    overlay.main_loop(
        |ctx| true,
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

    Ok(())
}
