fn main() -> anyhow::Result<()> {
    println!("Initialize overlay");
    let overlay = overlay::init("Task Manager Overlay", "Task Manager")?;
    overlay.main_loop(
        |ctx| {
            true
        },
        move |ui| {
            ui.window("Dummy Window").build(|| {
                ui.text("Taskmanager Overlay!");
                ui.text(format!("FPS: {:.2}", ui.io().framerate));
            });
            true
        },
    );

    Ok(())
}