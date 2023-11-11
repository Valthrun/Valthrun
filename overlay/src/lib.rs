use std::{
    ffi::CString,
    time::Instant,
};

use ash::vk;
use clipboard::ClipboardSupport;
use copypasta::ClipboardContext;
use imgui::{
    Context,
    FontConfig,
    FontSource,
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
        platform::windows::WindowExtWindows,
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
use window_tracker::WindowTracker;
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{
            BOOL,
            HWND,
        },
        Graphics::{
            Dwm::{
                DwmEnableBlurBehindWindow,
                DWM_BB_BLURREGION,
                DWM_BB_ENABLE,
                DWM_BLURBEHIND,
            },
            Gdi::CreateRectRgn,
        },
        UI::{
            Input::KeyboardAndMouse::SetActiveWindow,
            WindowsAndMessaging::{
                GetWindowLongPtrA,
                MessageBoxA,
                SetWindowDisplayAffinity,
                SetWindowLongA,
                SetWindowLongPtrA,
                SetWindowPos,
                ShowWindow,
                GWL_EXSTYLE,
                GWL_STYLE,
                HWND_TOPMOST,
                MB_ICONERROR,
                MB_OK,
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

mod vulkan_render;
use vulkan_render::*;

mod util;
mod vulkan_driver;

pub fn show_error_message(title: &str, message: &str) {
    let title = CString::new(title).unwrap_or_else(|_| CString::new("[[ NulError ]]").unwrap());
    let message = CString::new(message).unwrap_or_else(|_| CString::new("[[ NulError ]]").unwrap());
    unsafe {
        MessageBoxA(
            HWND::default(),
            PCSTR::from_raw(message.as_ptr() as *const u8),
            PCSTR::from_raw(title.as_ptr() as *const u8),
            MB_ICONERROR | MB_OK,
        );
    }
}

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

            let mut bb: DWM_BLURBEHIND = Default::default();
            bb.dwFlags = DWM_BB_ENABLE | DWM_BB_BLURREGION;
            bb.fEnable = BOOL::from(true);
            bb.hRgnBlur = CreateRectRgn(0, 0, 1, 1);
            DwmEnableBlurBehindWindow(hwnd, &bb)?;

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
    pub font_init: Option<Box<dyn Fn(&mut imgui::Context) -> ()>>,
}

fn create_imgui_context(options: &OverlayOptions) -> Result<(WinitPlatform, imgui::Context)> {
    let mut imgui = Context::create();
    imgui.set_ini_filename(None);

    let platform = WinitPlatform::init(&mut imgui);

    match ClipboardContext::new() {
        Ok(backend) => imgui.set_clipboard_backend(ClipboardSupport(backend)),
        Err(error) => log::warn!("Failed to initialize clipboard: {}", error),
    };

    // Fixed font size. Note imgui_winit_support uses "logical
    // pixels", which are physical pixels scaled by the devices
    // scaling factor. Meaning, 13.0 pixels should look the same size
    // on two different screens, and thus we do not need to scale this
    // value (as the scaling is handled by winit)
    let font_size = 18.0;
    imgui.fonts().add_font(&[FontSource::TtfData {
        data: include_bytes!("../resources/Roboto-Regular.ttf"),
        size_pixels: font_size,
        config: Some(FontConfig {
            // As imgui-glium-renderer isn't gamma-correct with
            // it's font rendering, we apply an arbitrary
            // multiplier to make the font a bit "heavier". With
            // default imgui-glow-renderer this is unnecessary.
            rasterizer_multiply: 1.5,
            // Oversampling font helps improve text rendering at
            // expense of larger font atlas texture.
            oversample_h: 4,
            oversample_v: 4,
            ..FontConfig::default()
        }),
    }]);
    if let Some(callback) = &options.font_init {
        callback(&mut imgui);
    }

    Ok((platform, imgui))
}

pub struct System {
    pub event_loop: EventLoop<()>,

    pub window: Window,
    pub platform: WinitPlatform,

    pub vulkan_context: VulkanContext,
    command_buffer: vk::CommandBuffer,
    swapchain: Swapchain,
    image_available_semaphore: vk::Semaphore,
    render_finished_semaphore: vk::Semaphore,
    fence: vk::Fence,

    pub imgui: Context,
    pub renderer: Renderer,

    pub window_tracker: WindowTracker,
}

pub fn init(options: &OverlayOptions) -> Result<System> {
    let window_tracker = WindowTracker::new(&options.target)?;

    let event_loop = EventLoop::new();
    let window = create_window(&event_loop, &options.title)?;

    let vulkan_context = VulkanContext::new(&window, &options.title)?;
    let command_buffer = {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(vulkan_context.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        unsafe {
            vulkan_context
                .device
                .allocate_command_buffers(&allocate_info)?[0]
        }
    };

    let swapchain = Swapchain::new(&vulkan_context)?;
    let image_available_semaphore = {
        let semaphore_info = vk::SemaphoreCreateInfo::builder();
        unsafe {
            vulkan_context
                .device
                .create_semaphore(&semaphore_info, None)?
        }
    };
    let render_finished_semaphore = {
        let semaphore_info = vk::SemaphoreCreateInfo::builder();
        unsafe {
            vulkan_context
                .device
                .create_semaphore(&semaphore_info, None)?
        }
    };
    let fence = {
        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        unsafe { vulkan_context.device.create_fence(&fence_info, None)? }
    };

    let (mut platform, mut imgui) = create_imgui_context(&options)?;
    platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Default);

    let renderer = Renderer::with_default_allocator(
        &vulkan_context.instance,
        vulkan_context.physical_device,
        vulkan_context.device.clone(),
        vulkan_context.graphics_queue,
        vulkan_context.command_pool,
        swapchain.render_pass,
        &mut imgui,
        Some(Options {
            in_flight_frames: 1,
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
        command_buffer,
        image_available_semaphore,
        render_finished_semaphore,
        fence,

        imgui,
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

impl System {
    pub fn main_loop<U, R>(self, mut update: U, mut render: R) -> !
    where
        U: FnMut(&mut SystemRuntimeController) -> bool + 'static,
        R: FnMut(&mut imgui::Ui) -> bool + 'static,
    {
        let System {
            event_loop,
            window,

            vulkan_context,
            mut swapchain,
            command_buffer,
            fence,
            image_available_semaphore,
            render_finished_semaphore,

            imgui,
            mut platform,
            mut renderer,

            window_tracker,
            ..
        } = self;
        let mut last_frame = Instant::now();

        let mut runtime_controller = SystemRuntimeController {
            hwnd: HWND(window.hwnd() as isize),
            imgui,

            active_tracker: OverlayActiveTracker::new(),
            key_input_system: KeyboardInputSystem::new(),
            mouse_input_system: MouseInputSystem::new(),
            window_tracker,

            frame_count: 0,
            debug_overlay_shown: false,
        };

        let mut dirty_swapchain = false;

        let mut perf = PerfTracker::new(PERF_RECORDS);
        event_loop.run(move |event, _, control_flow| {
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
                    perf.mark("events cleared");

                    /* Update */
                    {
                        if !runtime_controller.update_state(&window) {
                            log::info!("Target window has been closed. Exiting overlay.");
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

                        if let Err(error) =
                            platform.prepare_frame(runtime_controller.imgui.io_mut(), &window)
                        {
                            *control_flow = ControlFlow::ExitWithCode(1);
                            log::error!("Platform implementation prepare_frame failed: {}", error);
                            return;
                        }

                        let ui = runtime_controller.imgui.frame();
                        let run = render(ui);
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
                                .wait_for_fences(&[fence], true, std::u64::MAX)
                                .expect("Failed to wait ")
                        };

                        perf.mark("fence");
                        let next_image_result = unsafe {
                            swapchain.loader.acquire_next_image(
                                swapchain.khr,
                                std::u64::MAX,
                                image_available_semaphore,
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
                                .reset_fences(&[fence])
                                .expect("Failed to reset fences")
                        };

                        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
                        let wait_semaphores = [image_available_semaphore];
                        let signal_semaphores = [render_finished_semaphore];

                        // Re-record commands to draw geometry
                        record_command_buffers(
                            &vulkan_context.device,
                            vulkan_context.command_pool,
                            command_buffer,
                            swapchain.framebuffers[image_index as usize],
                            swapchain.render_pass,
                            swapchain.extent,
                            &mut renderer,
                            &draw_data,
                        )
                        .expect("Failed to record command buffer");

                        let command_buffers = [command_buffer];
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
                                .queue_submit(vulkan_context.graphics_queue, &submit_info, fence)
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
        })
    }
}

pub struct SystemRuntimeController {
    pub hwnd: HWND,

    pub imgui: imgui::Context,
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
