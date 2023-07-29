use copypasta::{ClipboardContext, ClipboardProvider};
use imgui::{ClipboardBackend, Context, Ui, FontSource, FontConfig};
use imgui_dx11_renderer::Renderer;
use imgui_winit_support::{winit::{event_loop::{EventLoop, ControlFlow}, event::{Event, WindowEvent}, window::{WindowBuilder, Window}, platform::windows::WindowExtWindows}, WinitPlatform, HiDpiMode};
use std::time::Instant;
use windows::core::ComInterface;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;

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
type Result<T> = std::result::Result<T, windows::core::Error>;

fn create_device_with_type(drive_type: D3D_DRIVER_TYPE) -> Result<ID3D11Device> {
    let mut flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT;

    if cfg!(debug_assertions) {
        flags |= D3D11_CREATE_DEVICE_DEBUG;
    }

    let mut device = None;
    let feature_levels = [D3D_FEATURE_LEVEL_11_1, D3D_FEATURE_LEVEL_10_0];
    let mut fl = D3D_FEATURE_LEVEL_11_1;
    unsafe {
        D3D11CreateDevice(
            None,
            drive_type,
            HMODULE::default(),
            flags,
            Some(feature_levels.as_slice()),
            D3D11_SDK_VERSION,
            Some(&mut device),
            Some(&mut fl),
            Some(&mut None),
        )
        .map(|()| device.unwrap())
    }
}

fn create_device() -> Result<ID3D11Device> {
    create_device_with_type(D3D_DRIVER_TYPE_HARDWARE)
}

fn create_swapchain(device: &ID3D11Device, window: HWND) -> Result<IDXGISwapChain> {
    let factory = get_dxgi_factory(device)?;

    let sc_desc = DXGI_SWAP_CHAIN_DESC {
        BufferDesc: DXGI_MODE_DESC {
            Width: 0,
            Height: 0,
            RefreshRate: DXGI_RATIONAL { Numerator: 60, Denominator: 1 },
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            ..Default::default()
        },
        SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
        BufferCount: 3,
        OutputWindow: window,
        Windowed: true.into(),
        SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
        Flags: DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH.0 as u32,
    };

    let mut swap_chain: Option<IDXGISwapChain> = None;
    unsafe { 
        factory.CreateSwapChain(device, &sc_desc, &mut swap_chain)
            .ok()
            .map(|_| swap_chain.unwrap())
    }
}

fn get_dxgi_factory(device: &ID3D11Device) -> Result<IDXGIFactory2> {
    let dxdevice = device.cast::<IDXGIDevice>()?;
    unsafe { dxdevice.GetAdapter()?.GetParent() }
}

fn create_render_target(
    swapchain: &IDXGISwapChain,
    device: &ID3D11Device,
) -> Result<ID3D11RenderTargetView> {
    let mut target: Option<ID3D11RenderTargetView> = None;
    unsafe {
        let backbuffer: ID3D11Resource = swapchain.GetBuffer(0)?;
        device.CreateRenderTargetView(&backbuffer, None, Some(&mut target))
            .map(|_| target.unwrap())
    }
}


pub struct System {
    pub event_loop: EventLoop<()>,
    pub imgui: Context,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
    pub font_size: f32,

    pub window: Window,
    pub device_ctx: ID3D11DeviceContext,
    pub target: ID3D11RenderTargetView,
    pub swapchain: IDXGISwapChain,
}

pub fn init(title: &str) -> System {
    let event_loop = EventLoop::new();
    
    let target_monitor = event_loop.primary_monitor()
        .or_else(|| event_loop.available_monitors().next())
        .unwrap();

    let window = WindowBuilder::new()
        .with_title(title.to_owned())
        .with_transparent(true)
        //.with_skip_taskbar(true)
        .with_always_on_top(true)
        .with_decorations(false)
        .with_inner_size(target_monitor.size())
        .with_position(target_monitor.position())
        .build(&event_loop)
        .unwrap();

    let device = create_device().unwrap();
    let swapchain = unsafe { create_swapchain(&device, std::mem::transmute(window.hwnd())).unwrap() };
    let device_ctx = unsafe { device.GetImmediateContext().unwrap() };

    let mut target = create_render_target(&swapchain, &device).unwrap();

    let mut imgui = Context::create();
    let mut platform = WinitPlatform::init(&mut imgui);
    imgui.set_ini_filename(None);
    platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);

    // Fixed font size. Note imgui_winit_support uses "logical
    // pixels", which are physical pixels scaled by the devices
    // scaling factor. Meaning, 13.0 pixels should look the same size
    // on two different screens, and thus we do not need to scale this
    // value (as the scaling is handled by winit)
    let font_size = 13.0;
    {
        imgui.fonts().add_font(&[
            FontSource::TtfData {
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
            }
        ]);
    }

    let mut renderer = unsafe { Renderer::new(&mut imgui, &device).unwrap() };
    
    System {
        event_loop,
        imgui,
        platform,
        renderer,
        font_size,
        
        window,
        device_ctx,
        target,
        swapchain,
    }
}

impl System {
    pub fn main_loop<F: FnMut(&mut bool, &mut Ui) + 'static>(self, mut run_ui: F) {
        let System {
            event_loop,
            mut imgui,
            mut platform,
            mut renderer,
            mut window,
            mut device_ctx,
            mut target,
            mut swapchain,
            ..
        } = self;
        let mut last_frame = Instant::now();

        event_loop.run(move |event, _, control_flow| match event {
            Event::NewEvents(_) => {
                let now = Instant::now();
                imgui.io_mut().update_delta_time(now - last_frame);
                last_frame = now;
            }
            Event::MainEventsCleared => {
                let io = imgui.io_mut();
                platform.prepare_frame(io, &window).expect("Failed to start frame");
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                unsafe {
                    device_ctx.OMSetRenderTargets(Some([Some(target.clone())].as_slice()), None);
                    device_ctx.ClearRenderTargetView(&target, &0.6);
                }
                let ui = imgui.new_frame();
                
                let mut run = true;
                run_ui(&mut run, ui);
                if !run {
                    *control_flow = ControlFlow::Exit;
                }
    
                
                platform.prepare_render(&ui, &window);
                renderer.render(imgui.render()).unwrap();
                unsafe {
                    swapchain.Present(1, 0).unwrap();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            event => platform.handle_event(imgui.io_mut(), &window, &event),
        })
    }
}