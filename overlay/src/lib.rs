#![feature(str_from_utf16_endian)]
use std::time::Instant;

use ash::vk::{
    self,
};
use clipboard::ClipboardSupport;
use copypasta::ClipboardContext;
use font::FontAtlasBuilder;
use imgui::{
    Context,
    FontAtlas,
    Io,
};
use imgui_rs_vulkan_renderer::{
    Options,
    Renderer,
};
use imgui_winit_support::{
    winit::{
        dpi::PhysicalSize,
        event::{
            Event,
            WindowEvent,
        },
        event_loop::{
            ControlFlow,
            EventLoop,
        },
        platform::{
            run_return::EventLoopExtRunReturn,
            windows::WindowExtWindows,
        },
        window::{
            Window,
            WindowBuilder,
        },
    },
    HiDpiMode,
    WinitPlatform,
};
use input::{
    KeyboardInputSystem,
    MouseInputSystem,
};
use obfstr::obfstr;
use vulkan::render::{
    record_command_buffers,
    Swapchain,
    VulkanContext,
};
use window_tracker::WindowTracker;
use windows::Win32::{
    Foundation::{
        BOOL,
        HWND,
    },
    Graphics::{
        Dwm::{
            DwmEnableBlurBehindWindow,
            DwmIsCompositionEnabled,
            DWM_BB_BLURREGION,
            DWM_BB_ENABLE,
            DWM_BLURBEHIND,
        },
        Gdi::{
            CreateRectRgn,
            DeleteObject,
        },
    },
    UI::{
        Input::KeyboardAndMouse::SetActiveWindow,
        WindowsAndMessaging::{
            GetWindowLongPtrA,
            SetWindowDisplayAffinity,
            SetWindowLongA,
            SetWindowLongPtrA,
            SetWindowPos,
            ShowWindow,
            GWL_EXSTYLE,
            GWL_STYLE,
            HWND_TOPMOST,
            SWP_NOACTIVATE,
            SWP_NOMOVE,
            SWP_NOSIZE,
            SW_SHOWNOACTIVATE,
            WDA_EXCLUDEFROMCAPTURE,
            WDA_NONE,
            WS_CLIPSIBLINGS,
            WS_EX_LAYERED,
            WS_EX_NOACTIVATE,
            WS_EX_TOOLWINDOW,
            WS_EX_TRANSPARENT,
            WS_POPUP,
            WS_VISIBLE,
        },
    },
};

mod clipboard;
mod error;
pub use error::*;
mod input;
mod window_tracker;
pub use window_tracker::OverlayTarget;

mod vulkan;

mod perf;
pub use perf::PerfTracker;

mod font;
mod util;

pub use font::UnicodeTextRenderer;
pub use util::show_error_message;

fn create_window(event_loop: &EventLoop<()>, title: &str) -> Result<Window> {
    let window = WindowBuilder::new()
        .with_title(title.to_owned())
        .with_visible(false)
        .build(&event_loop)?;

    {
        let hwnd = HWND(window.hwnd());
        unsafe {
            // Make it transparent
            SetWindowLongA(
                hwnd,
                GWL_STYLE,
                (WS_POPUP | WS_VISIBLE | WS_CLIPSIBLINGS).0 as i32,
            );
            SetWindowLongPtrA(
                hwnd,
                GWL_EXSTYLE,
                (WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT).0
                    as isize,
            );

            if !DwmIsCompositionEnabled()?.as_bool() {
                return Err(OverlayError::DwmCompositionDisabled);
            }

            let mut bb: DWM_BLURBEHIND = Default::default();
            bb.dwFlags = DWM_BB_ENABLE | DWM_BB_BLURREGION;
            bb.fEnable = BOOL::from(true);
            bb.hRgnBlur = CreateRectRgn(0, 0, 1, 1);
            DwmEnableBlurBehindWindow(hwnd, &bb)?;
            DeleteObject(bb.hRgnBlur);

            // Move the window to the top
            SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }

    Ok(window)
}

pub struct OverlayOptions {
    pub title: String,
    pub target: OverlayTarget,
    pub register_fonts_callback: Option<Box<dyn Fn(&mut FontAtlas) -> ()>>,
}

fn create_imgui_context(_options: &OverlayOptions) -> Result<(WinitPlatform, imgui::Context)> {
    let mut imgui = Context::create();
    imgui.set_ini_filename(None);

    let platform = WinitPlatform::init(&mut imgui);

    match ClipboardContext::new() {
        Ok(backend) => imgui.set_clipboard_backend(ClipboardSupport(backend)),
        Err(error) => log::warn!("Failed to initialize clipboard: {}", error),
    };

    Ok((platform, imgui))
}

pub struct System {
    pub event_loop: EventLoop<()>,

    pub window: Window,
    pub platform: WinitPlatform,

    pub vulkan_context: VulkanContext,
    swapchain: Swapchain,

    frame_data: Vec<FrameData>,
    frame_data_index: usize,

    pub imgui: Context,
    pub imgui_fonts: FontAtlasBuilder,
    pub imgui_register_fonts_callback: Option<Box<dyn Fn(&mut FontAtlas) -> ()>>,

    pub renderer: Renderer,

    pub window_tracker: WindowTracker,
}

pub fn init(options: OverlayOptions) -> Result<System> {
    let window_tracker = WindowTracker::new(&options.target)?;

    let event_loop = EventLoop::new();
    let window = create_window(&event_loop, &options.title)?;

    let vulkan_context = VulkanContext::new(&window)?;
    let frame_data = vec![
        FrameData::new(&vulkan_context)?,
        // FrameData::new(&vulkan_context)?,
    ];
    let swapchain = Swapchain::new(&vulkan_context)?;

    let (mut platform, mut imgui) = create_imgui_context(&options)?;
    platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Default);

    let mut imgui_fonts = FontAtlasBuilder::new();
    imgui_fonts.register_font(include_bytes!("../resources/Roboto-Regular.ttf"))?;
    imgui_fonts.register_font(include_bytes!("../resources/NotoSansTC-Regular.ttf"))?;
    /* fallback if we do not have the roboto version of the glyph */
    imgui_fonts.register_font(include_bytes!("../resources/unifont-15.1.05.otf"))?;
    imgui_fonts.register_codepoints(1..255);

    let renderer = Renderer::with_default_allocator(
        &vulkan_context.instance,
        vulkan_context.physical_device,
        vulkan_context.device.clone(),
        vulkan_context.graphics_queue,
        frame_data[0].command_pool, // Just any pool will do. Only one time thing
        swapchain.render_pass,
        &mut imgui,
        Some(Options {
            in_flight_frames: frame_data.len(),
            ..Default::default()
        }),
    )?;

    /* The Vulkan backend can handle 32bit vertex offsets, but forgets to insert that flag... */
    imgui
        .io_mut()
        .backend_flags
        .insert(imgui::BackendFlags::RENDERER_HAS_VTX_OFFSET);

    Ok(System {
        event_loop,
        window,

        vulkan_context,
        swapchain,

        frame_data,
        frame_data_index: 0,

        imgui,
        imgui_fonts,
        imgui_register_fonts_callback: options.register_fonts_callback,

        platform,
        renderer,

        window_tracker,
    })
}

/// Toggles the overlay noactive and transparent state
/// according to whenever ImGui wants mouse/cursor grab.
struct OverlayActiveTracker {
    currently_active: bool,
}

impl OverlayActiveTracker {
    pub fn new() -> Self {
        Self {
            currently_active: true,
        }
    }

    pub fn update(&mut self, window: &Window, io: &Io) {
        let window_active = io.want_capture_mouse | io.want_capture_keyboard;
        if window_active == self.currently_active {
            return;
        }

        self.currently_active = window_active;
        unsafe {
            let hwnd = HWND(window.hwnd());
            let mut style = GetWindowLongPtrA(hwnd, GWL_EXSTYLE);
            if window_active {
                style &= !((WS_EX_NOACTIVATE | WS_EX_TRANSPARENT).0 as isize);
            } else {
                style |= (WS_EX_NOACTIVATE | WS_EX_TRANSPARENT).0 as isize;
            }

            log::trace!("Set UI active: {window_active}");
            SetWindowLongPtrA(hwnd, GWL_EXSTYLE, style);
            if window_active {
                SetActiveWindow(hwnd);
            }
        }
    }
}

const PERF_RECORDS: usize = 2048;

struct FrameData {
    device: ash::Device,

    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,

    semaphore_image_available: vk::Semaphore,
    semaphore_render_finished: vk::Semaphore,

    render_fence: vk::Fence,
}

impl FrameData {
    pub fn new(instance: &VulkanContext) -> Result<Self> {
        let device = instance.device.clone();

        let command_pool = {
            let command_pool_info = vk::CommandPoolCreateInfo::builder()
                .queue_family_index(instance.graphics_q_index)
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
            unsafe { device.create_command_pool(&command_pool_info, None)? }
        };

        let command_buffer = {
            let allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);

            unsafe { device.allocate_command_buffers(&allocate_info)?[0] }
        };

        let semaphore_image_available = {
            let semaphore_info = vk::SemaphoreCreateInfo::builder();
            unsafe { device.create_semaphore(&semaphore_info, None)? }
        };

        let semaphore_render_finished = {
            let semaphore_info = vk::SemaphoreCreateInfo::builder();
            unsafe { device.create_semaphore(&semaphore_info, None)? }
        };

        let render_fence = {
            let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
            unsafe { device.create_fence(&fence_info, None)? }
        };

        Ok(Self {
            device,

            command_pool,
            command_buffer,

            semaphore_image_available,
            semaphore_render_finished,

            render_fence,
        })
    }
}

impl Drop for FrameData {
    fn drop(&mut self) {
        log::debug!("Dropping FrameData");
        unsafe {
            if let Err(err) = self
                .device
                .wait_for_fences(&[self.render_fence], true, 10_000_000)
            {
                log::error!("Failed to wait on fence for frame data destory: {}", err);
            }

            self.device.destroy_fence(self.render_fence, None);

            self.device
                .destroy_semaphore(self.semaphore_image_available, None);
            self.device
                .destroy_semaphore(self.semaphore_render_finished, None);

            self.device
                .free_command_buffers(self.command_pool, &[self.command_buffer]);
            self.device.destroy_command_pool(self.command_pool, None);
        }
    }
}

impl System {
    pub fn main_loop<U, R>(self, mut update: U, mut render: R) -> i32
    where
        U: FnMut(&mut SystemRuntimeController) -> bool + 'static,
        R: FnMut(&imgui::Ui, &UnicodeTextRenderer) -> bool + 'static,
    {
        let System {
            mut event_loop,
            window,

            vulkan_context,
            mut swapchain,

            imgui,
            imgui_fonts,
            imgui_register_fonts_callback,

            mut platform,
            mut renderer,

            frame_data: frame_datas,
            mut frame_data_index,

            window_tracker,
            ..
        } = self;

        let mut last_frame = Instant::now();

        let mut runtime_controller = SystemRuntimeController {
            hwnd: HWND(window.hwnd() as isize),

            imgui,
            imgui_fonts,

            active_tracker: OverlayActiveTracker::new(),
            key_input_system: KeyboardInputSystem::new(),
            mouse_input_system: MouseInputSystem::new(),
            window_tracker,

            frame_count: 0,
            debug_overlay_shown: false,
        };

        let mut dirty_swapchain = false;

        let vulkan_context = &vulkan_context;
        let swapchain = &mut swapchain;
        let renderer = &mut renderer;
        let frame_datas = &frame_datas;

        let mut perf = PerfTracker::new(PERF_RECORDS);
        let result = event_loop.run_return(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            platform.handle_event(runtime_controller.imgui.io_mut(), &window, &event);

            match event {
                // New frame
                Event::NewEvents(_) => {
                    perf.begin();
                    let now = Instant::now();
                    runtime_controller
                        .imgui
                        .io_mut()
                        .update_delta_time(now - last_frame);
                    last_frame = now;
                }

                // End of event processing
                Event::MainEventsCleared => {
                    frame_data_index = frame_data_index.wrapping_add(1);
                    let frame_data = &frame_datas[frame_data_index % frame_datas.len()];

                    perf.mark("events cleared");

                    /* Update */
                    {
                        if !runtime_controller.update_state(&window) {
                            *control_flow = ControlFlow::Exit;
                            return;
                        }

                        if !update(&mut runtime_controller) {
                            *control_flow = ControlFlow::Exit;
                            return;
                        }

                        perf.mark("update");
                    }

                    /* render */
                    {
                        // If swapchain must be recreated wait for windows to not be minimized anymore
                        if dirty_swapchain {
                            let PhysicalSize { width, height } = window.inner_size();
                            if width > 0 && height > 0 {
                                log::debug!("Recreate swapchain");
                                swapchain
                                    .recreate(&vulkan_context)
                                    .expect("Failed to recreate swapchain");
                                renderer
                                    .set_render_pass(swapchain.render_pass)
                                    .expect("Failed to rebuild renderer pipeline");
                                dirty_swapchain = false;
                            } else {
                                return;
                            }
                        }

                        if runtime_controller.imgui_fonts.fetch_reset_flag_updated() {
                            let font_atlas = runtime_controller.imgui.fonts();
                            font_atlas.clear();

                            let (font_sources, _glyph_memory) =
                                runtime_controller.imgui_fonts.build_font_source(18.0);

                            font_atlas.add_font(&font_sources);
                            if let Some(user_callback) = &imgui_register_fonts_callback {
                                user_callback(font_atlas);
                            }

                            if let Err(err) = renderer.update_fonts_texture(
                                vulkan_context.graphics_queue,
                                frame_data.command_pool,
                                &mut runtime_controller.imgui,
                            ) {
                                log::warn!("Failed to update fonts texture: {}", err);
                            } else {
                                log::debug!("Updated font texture successfully");
                            }
                        }

                        if let Err(error) =
                            platform.prepare_frame(runtime_controller.imgui.io_mut(), &window)
                        {
                            *control_flow = ControlFlow::ExitWithCode(1);
                            log::error!("Platform implementation prepare_frame failed: {}", error);
                            return;
                        }

                        let ui = runtime_controller.imgui.frame();
                        let unicode_text =
                            UnicodeTextRenderer::new(ui, &mut runtime_controller.imgui_fonts);

                        let run = render(ui, &unicode_text);
                        if !run {
                            *control_flow = ControlFlow::ExitWithCode(0);
                            return;
                        }
                        if runtime_controller.debug_overlay_shown {
                            ui.window("Render Debug")
                                .position([200.0, 200.0], imgui::Condition::FirstUseEver)
                                .size([400.0, 400.0], imgui::Condition::FirstUseEver)
                                .build(|| {
                                    ui.text(format!("FPS: {: >4.2}", ui.io().framerate));
                                    ui.same_line_with_pos(100.0);

                                    ui.text(format!(
                                        "Frame Time: {:.2}ms",
                                        ui.io().delta_time * 1000.0
                                    ));
                                    ui.same_line_with_pos(275.0);

                                    ui.text("History length:");
                                    ui.same_line();
                                    let mut history_length = perf.history_length();
                                    ui.set_next_item_width(75.0);
                                    if ui
                                        .input_scalar("##history_length", &mut history_length)
                                        .build()
                                    {
                                        perf.set_history_length(history_length);
                                    }
                                    perf.render(ui, ui.content_region_avail());
                                });
                        }
                        perf.mark("render frame");

                        platform.prepare_render(ui, &window);
                        let draw_data = runtime_controller.imgui.render();

                        unsafe {
                            vulkan_context
                                .device
                                .wait_for_fences(&[frame_data.render_fence], true, u64::MAX)
                                .expect("failed to wait for render fence");
                        };

                        perf.mark("fence");
                        let next_image_result = unsafe {
                            swapchain.loader.acquire_next_image(
                                swapchain.khr,
                                std::u64::MAX,
                                frame_data.semaphore_image_available,
                                vk::Fence::null(),
                            )
                        };
                        let image_index = match next_image_result {
                            Ok((image_index, _)) => image_index,
                            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                                dirty_swapchain = true;
                                return;
                            }
                            Err(error) => {
                                panic!("Error while acquiring next image. Cause: {}", error)
                            }
                        };
                        unsafe {
                            vulkan_context
                                .device
                                .reset_fences(&[frame_data.render_fence])
                                .expect("failed to reset fences");
                        };

                        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
                        let wait_semaphores = [frame_data.semaphore_image_available];
                        let signal_semaphores = [frame_data.semaphore_render_finished];

                        // Re-record commands to draw geometry
                        record_command_buffers(
                            &vulkan_context.device,
                            frame_data.command_pool,
                            frame_data.command_buffer,
                            swapchain.framebuffers[image_index as usize],
                            swapchain.render_pass,
                            swapchain.extent,
                            renderer,
                            &draw_data,
                        )
                        .expect("Failed to record command buffer");

                        let command_buffers = [frame_data.command_buffer];
                        let submit_info = [vk::SubmitInfo::builder()
                            .wait_semaphores(&wait_semaphores)
                            .wait_dst_stage_mask(&wait_stages)
                            .command_buffers(&command_buffers)
                            .signal_semaphores(&signal_semaphores)
                            .build()];

                        perf.mark("before submit");
                        unsafe {
                            vulkan_context
                                .device
                                .queue_submit(
                                    vulkan_context.graphics_queue,
                                    &submit_info,
                                    frame_data.render_fence,
                                )
                                .expect("Failed to submit work to gpu.")
                        };
                        perf.mark("after submit");

                        let swapchains = [swapchain.khr];
                        let images_indices = [image_index];
                        let present_info = vk::PresentInfoKHR::builder()
                            .wait_semaphores(&signal_semaphores)
                            .swapchains(&swapchains)
                            .image_indices(&images_indices);

                        let present_result = unsafe {
                            swapchain
                                .loader
                                .queue_present(vulkan_context.present_queue, &present_info)
                        };
                        match present_result {
                            Ok(is_suboptimal) if is_suboptimal => {
                                dirty_swapchain = true;
                            }
                            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                                dirty_swapchain = true;
                            }
                            Err(error) => panic!("Failed to present queue. Cause: {}", error),
                            _ => {}
                        }
                        perf.finish("present");

                        runtime_controller.frame_rendered();
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => {}
            }
        });

        if let Err(err) = unsafe { vulkan_context.device.device_wait_idle() } {
            log::warn!("Failed to wait for device idle: {}", err);
        };
        result
    }
}

pub struct SystemRuntimeController {
    pub hwnd: HWND,

    pub imgui: imgui::Context,
    pub imgui_fonts: FontAtlasBuilder,

    debug_overlay_shown: bool,

    active_tracker: OverlayActiveTracker,
    mouse_input_system: MouseInputSystem,
    key_input_system: KeyboardInputSystem,

    window_tracker: WindowTracker,

    frame_count: u64,
}

impl SystemRuntimeController {
    fn update_state(&mut self, window: &Window) -> bool {
        self.mouse_input_system.update(window, self.imgui.io_mut());
        self.key_input_system.update(window, self.imgui.io_mut());
        self.active_tracker.update(window, self.imgui.io());
        if !self.window_tracker.update(window) {
            log::info!("Target window has been closed. Exiting overlay.");
            return false;
        }

        true
    }

    fn frame_rendered(&mut self) {
        self.frame_count += 1;
        if self.frame_count == 1 {
            /* initial frame */
            unsafe { ShowWindow(self.hwnd, SW_SHOWNOACTIVATE) };

            self.window_tracker.mark_force_update();
        }
    }

    pub fn toggle_screen_capture_visibility(&self, should_be_visible: bool) {
        unsafe {
            let (target_state, state_name) = if should_be_visible {
                (WDA_NONE, "normal")
            } else {
                (WDA_EXCLUDEFROMCAPTURE, "exclude from capture")
            };

            if !SetWindowDisplayAffinity(self.hwnd, target_state).as_bool() {
                log::warn!(
                    "{} '{}'.",
                    obfstr!("Failed to change overlay display affinity to"),
                    state_name
                );
            }
        }
    }

    pub fn toggle_debug_overlay(&mut self, visible: bool) {
        self.debug_overlay_shown = visible;
    }

    pub fn debug_overlay_shown(&self) -> bool {
        self.debug_overlay_shown
    }
}
