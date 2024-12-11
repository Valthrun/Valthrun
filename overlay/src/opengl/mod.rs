use std::num::NonZeroU32;

use glutin::{
    config::ConfigTemplateBuilder,
    context::{
        ContextAttributesBuilder,
        PossiblyCurrentContext,
    },
    display::GetGlDisplay,
    prelude::{
        GlDisplay,
        NotCurrentGlContext,
    },
    surface::{
        GlSurface,
        Surface,
        SurfaceAttributesBuilder,
        WindowSurface,
    },
};
use imgui_glow_renderer::{
    glow::{
        self,
        HasContext,
    },
    AutoRenderer,
};
use winit::{
    event_loop::EventLoop,
    raw_window_handle::HasWindowHandle,
    window::Window,
};

use crate::{
    RenderBackend,
    Result,
};

pub struct OpenGLRenderBackend {
    surface: Surface<WindowSurface>,
    context: PossiblyCurrentContext,
    imgui_renderer: Option<AutoRenderer>,
}

impl OpenGLRenderBackend {
    pub fn new(event_loop: &EventLoop<()>, window: &Window) -> Result<Self> {
        let (_, cfg) = glutin_winit::DisplayBuilder::new()
            .build(event_loop, ConfigTemplateBuilder::new(), |mut configs| {
                configs.next().unwrap()
            })
            .expect("Failed to create OpenGL window");

        let context_attribs =
            ContextAttributesBuilder::new().build(Some(window.window_handle().unwrap().as_raw()));
        let context = unsafe {
            cfg.display()
                .create_context(&cfg, &context_attribs)
                .expect("Failed to create OpenGL context")
        };

        let surface_attribs = SurfaceAttributesBuilder::<WindowSurface>::new()
            .with_srgb(Some(true))
            .build(
                window.window_handle().unwrap().as_raw(),
                NonZeroU32::new(1024).unwrap(),
                NonZeroU32::new(768).unwrap(),
            );
        let surface = unsafe {
            cfg.display()
                .create_window_surface(&cfg, &surface_attribs)
                .expect("Failed to create OpenGL surface")
        };

        let context = context
            .make_current(&surface)
            .expect("Failed to make OpenGL context current");

        Ok(Self {
            surface: surface,
            context,

            imgui_renderer: None,
        })
    }
}

impl RenderBackend for OpenGLRenderBackend {
    fn render_frame(
        &mut self,
        _perf: &mut crate::PerfTracker,
        _window: &Window,
        draw_data: &imgui::DrawData,
    ) {
        if let Some(renderer) = &mut self.imgui_renderer {
            unsafe { renderer.gl_context().clear(glow::COLOR_BUFFER_BIT) };
            renderer.render(draw_data).unwrap();
        }

        self.surface.swap_buffers(&self.context).unwrap();
    }

    fn update_fonts_texture(&mut self, imgui: &mut imgui::Context) {
        self.imgui_renderer = Some(
            AutoRenderer::new(glow_context(&self.context), imgui)
                .expect("failed to create renderer"),
        );
    }
}

fn glow_context(context: &PossiblyCurrentContext) -> glow::Context {
    unsafe {
        glow::Context::from_loader_function_cstr(|s| context.display().get_proc_address(s).cast())
    }
}
