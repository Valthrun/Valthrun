use overlay::OverlayTarget;

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .parse_default_env()
        .init();

    log::info!("Initialize overlay");
    let overlay = overlay::init(overlay::OverlayOptions {
        title: "Task Manager Overlay".to_string(),
        target: OverlayTarget::WindowTitle("Task Manager".into()),
        register_fonts_callback: None,
    })?;
    let mut text_input = Default::default();
    let mut run_loop = true;

    overlay.main_loop(
        |controller| {
            controller.toggle_debug_overlay(true);
            true
        },
        move |ui, unicode_text| {
            ui.window("Dummy Window")
                .resizable(true)
                .movable(true)
                .build(|| {
                    unicode_text.text("Taskmanager Overlay!");
                    unicode_text.text(format!("FPS: {:.2}", ui.io().framerate));

                    ui.input_text("Test-Input", &mut text_input).build();
                    unicode_text.register_unicode_text(&text_input);

                    if ui.button("Close") {
                        run_loop = false
                    }

                    unicode_text.text("Привет, мир!");
                    unicode_text.text("Chào thế giới!");
                    unicode_text.text("Chào thế giới!");
                    unicode_text.text("ສະ​ບາຍ​ດີ​ຊາວ​ໂລກ!");
                    unicode_text.text("Салом Ҷаҳон!");
                    unicode_text.text("こんにちは世界!");
                    unicode_text.text("你好世界!");
                    unicode_text.text("﷽, ♛ LAZ ♛,  ♛ ॐ,  ♛ ॐ");
                    unicode_text.text(" ♣▄♠░ ");
                    unicode_text.text("♣♠░:D ︻デ── ");
                });

            run_loop
        },
    );
    Ok(())
}
