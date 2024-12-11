use std::ffi::CStr;

use ash::{
    khr::{
        self,
        surface::Instance as Surface,
        swapchain::Device as SwapchainDevice,
    },
    vk::{
        self,
        CompositeAlphaFlagsKHR,
    },
    Device,
    Entry,
    Instance,
};
use imgui::DrawData;
use imgui_rs_vulkan_renderer::Renderer;
use imgui_winit_support::winit::window::Window;
use winit::raw_window_handle::{
    HasDisplayHandle,
    HasWindowHandle,
};

use super::{
    driver::get_vulkan_entry,
    instance::create_vulkan_instance,
};
use crate::VulkanError;

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;

pub struct VulkanContext {
    _entry: Entry,
    pub instance: Instance,

    surface: Surface,
    surface_khr: vk::SurfaceKHR,

    pub physical_device: vk::PhysicalDevice,
    pub graphics_q_index: u32,
    pub present_q_index: u32,

    pub device: Device,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
}

impl VulkanContext {
    pub fn new(window: &Window) -> Result<Self, VulkanError> {
        // Vulkan instance
        let entry = get_vulkan_entry()?;
        let instance = create_vulkan_instance(&entry, window)?;

        // Vulkan surface
        let surface = Surface::new(&entry, &instance);
        let surface_khr = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None,
            )
            .map_err(VulkanError::SurfaceCreationFailed)?
        };

        // Vulkan physical device and queue families indices (graphics and present)
        let (physical_device, graphics_q_index, present_q_index) =
            create_vulkan_physical_device_and_get_graphics_and_present_qs_indices(
                &instance,
                &surface,
                surface_khr,
            )?;

        // Vulkan logical device and queues
        let (device, graphics_queue, present_queue) =
            create_vulkan_device_and_graphics_and_present_qs(
                &instance,
                physical_device,
                graphics_q_index,
                present_q_index,
            )?;

        Ok(Self {
            _entry: entry,
            instance,
            surface,
            surface_khr,
            physical_device,
            graphics_q_index,
            present_q_index,
            device,
            graphics_queue,
            present_queue,
        })
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        log::debug!("Destroying Vulkan Context");
        unsafe {
            self.device.destroy_device(None);
            self.surface.destroy_surface(self.surface_khr, None);
            self.instance.destroy_instance(None);
        }
    }
}

pub struct Swapchain {
    device: ash::Device,
    pub loader: SwapchainDevice,
    pub extent: vk::Extent2D,
    pub khr: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,
}

impl Swapchain {
    pub fn new(vulkan_context: &VulkanContext) -> Result<Self, VulkanError> {
        // Swapchain
        let (loader, khr, extent, format, images, image_views) =
            create_vulkan_swapchain(&vulkan_context)?;

        // Renderpass
        let render_pass = create_vulkan_render_pass(&vulkan_context.device, format)?;

        // Framebuffers
        let framebuffers =
            create_vulkan_framebuffers(&vulkan_context.device, render_pass, extent, &image_views)?;

        Ok(Self {
            device: vulkan_context.device.clone(),

            loader,
            extent,
            khr,
            images,
            image_views,
            render_pass,
            framebuffers,
        })
    }

    pub fn recreate(&mut self, vulkan_context: &VulkanContext) -> Result<(), VulkanError> {
        log::debug!("Recreating the swapchain");

        unsafe { vulkan_context.device.device_wait_idle()? };

        self.destroy();

        // Swapchain
        let (loader, khr, extent, format, images, image_views) =
            create_vulkan_swapchain(vulkan_context)?;

        // Renderpass
        let render_pass = create_vulkan_render_pass(&vulkan_context.device, format)?;

        // Framebuffers
        let framebuffers =
            create_vulkan_framebuffers(&vulkan_context.device, render_pass, extent, &image_views)?;

        self.loader = loader;
        self.extent = extent;
        self.khr = khr;
        self.images = images;
        self.image_views = image_views;
        self.render_pass = render_pass;
        self.framebuffers = framebuffers;

        Ok(())
    }

    fn destroy(&mut self) {
        log::debug!("Destroying Swapchain");
        unsafe {
            self.framebuffers
                .iter()
                .for_each(|fb| self.device.destroy_framebuffer(*fb, None));
            self.framebuffers.clear();
            self.device.destroy_render_pass(self.render_pass, None);
            self.image_views
                .iter()
                .for_each(|v| self.device.destroy_image_view(*v, None));
            self.image_views.clear();
            self.loader.destroy_swapchain(self.khr, None);
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        log::debug!("Dropping Swapchain");
        self.destroy();
    }
}

fn create_vulkan_physical_device_and_get_graphics_and_present_qs_indices(
    instance: &Instance,
    surface: &Surface,
    surface_khr: vk::SurfaceKHR,
) -> Result<(vk::PhysicalDevice, u32, u32), VulkanError> {
    log::debug!("Creating vulkan physical device");
    let devices = unsafe { instance.enumerate_physical_devices()? };
    let mut graphics = None;
    let mut present = None;

    log::debug!("Available devices:");
    for device in &devices {
        unsafe {
            let props = instance.get_physical_device_properties(*device);
            let device_name = CStr::from_ptr(props.device_name.as_ptr());

            log::debug!("- #{} {device_name:?}", props.device_id);
        }
    }

    let device = devices
        .into_iter()
        .find(|device| {
            let device = *device;

            // Does device supports graphics and present queues
            let props = unsafe { instance.get_physical_device_queue_family_properties(device) };
            for (index, family) in props.iter().filter(|f| f.queue_count > 0).enumerate() {
                let index = index as u32;
                graphics = None;
                present = None;

                if family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                    && family.queue_flags.contains(vk::QueueFlags::COMPUTE)
                    && graphics.is_none()
                {
                    graphics = Some(index);
                }

                let present_support = unsafe {
                    surface
                        .get_physical_device_surface_support(device, index, surface_khr)
                        .expect("Failed to get surface support")
                };
                if present_support && present.is_none() {
                    present = Some(index);
                }

                if graphics.is_some() && present.is_some() {
                    break;
                }
            }

            // Does device support desired extensions
            let extension_props = unsafe {
                instance
                    .enumerate_device_extension_properties(device)
                    .expect("Failed to get device ext properties")
            };
            let extention_support = extension_props.iter().any(|ext| {
                let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
                khr::swapchain::NAME == name
            });

            // Does the device have available formats for the given surface
            let formats = unsafe {
                surface
                    .get_physical_device_surface_formats(device, surface_khr)
                    .expect("Failed to get physical device surface formats")
            };

            // Does the device have available present modes for the given surface
            let present_modes = unsafe {
                surface
                    .get_physical_device_surface_present_modes(device, surface_khr)
                    .expect("Failed to get physical device surface present modes")
            };

            graphics.is_some()
                && present.is_some()
                && extention_support
                && !formats.is_empty()
                && !present_modes.is_empty()
        })
        .expect("Could not find a suitable device");

    unsafe {
        let props = instance.get_physical_device_properties(device);
        let device_name = CStr::from_ptr(props.device_name.as_ptr());
        log::debug!(
            "Selected physical device: #{} {device_name:?}",
            props.device_id
        );
    }

    Ok((device, graphics.unwrap(), present.unwrap()))
}

fn create_vulkan_device_and_graphics_and_present_qs(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    graphics_q_index: u32,
    present_q_index: u32,
) -> Result<(Device, vk::Queue, vk::Queue), VulkanError> {
    log::debug!("Creating vulkan device and graphics and present queues");
    let queue_priorities = [1.0f32];
    let queue_create_infos = {
        let mut indices = vec![graphics_q_index, present_q_index];
        indices.dedup();

        indices
            .iter()
            .map(|index| {
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(*index)
                    .queue_priorities(&queue_priorities)
            })
            .collect::<Vec<_>>()
    };

    let device_extensions_ptrs = [khr::swapchain::NAME.as_ptr()];

    let device_create_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(&queue_create_infos)
        .enabled_extension_names(&device_extensions_ptrs);

    let device = unsafe { instance.create_device(physical_device, &device_create_info, None)? };
    let graphics_queue = unsafe { device.get_device_queue(graphics_q_index, 0) };
    let present_queue = unsafe { device.get_device_queue(present_q_index, 0) };

    Ok((device, graphics_queue, present_queue))
}

fn create_vulkan_swapchain(
    vulkan_context: &VulkanContext,
) -> Result<
    (
        SwapchainDevice,
        vk::SwapchainKHR,
        vk::Extent2D,
        vk::Format,
        Vec<vk::Image>,
        Vec<vk::ImageView>,
    ),
    VulkanError,
> {
    log::debug!("Creating vulkan swapchain");
    // Swapchain format
    let format = {
        let formats = unsafe {
            vulkan_context.surface.get_physical_device_surface_formats(
                vulkan_context.physical_device,
                vulkan_context.surface_khr,
            )?
        };
        if formats.len() == 1 && formats[0].format == vk::Format::UNDEFINED {
            vk::SurfaceFormatKHR {
                format: vk::Format::B8G8R8A8_UNORM,
                color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
            }
        } else {
            *formats
                .iter()
                .find(|format| {
                    format.format == vk::Format::B8G8R8A8_UNORM
                        && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                })
                .unwrap_or(&formats[0])
        }
    };
    log::debug!("Swapchain format: {format:?}");

    // Swapchain present mode
    let present_mode = {
        let present_modes = unsafe {
            vulkan_context
                .surface
                .get_physical_device_surface_present_modes(
                    vulkan_context.physical_device,
                    vulkan_context.surface_khr,
                )?
        };
        if present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            vk::PresentModeKHR::MAILBOX
        } else if present_modes.contains(&vk::PresentModeKHR::IMMEDIATE) {
            vk::PresentModeKHR::IMMEDIATE
        } else {
            vk::PresentModeKHR::FIFO
        }
    };
    log::debug!("Swapchain present mode: {present_mode:?}");

    let capabilities = unsafe {
        vulkan_context
            .surface
            .get_physical_device_surface_capabilities(
                vulkan_context.physical_device,
                vulkan_context.surface_khr,
            )?
    };
    log::debug!(
        "Supported composite alpha: {:b}",
        capabilities.supported_composite_alpha.as_raw()
    );
    let composite_alpha = if capabilities
        .supported_composite_alpha
        .contains(CompositeAlphaFlagsKHR::PRE_MULTIPLIED)
    {
        CompositeAlphaFlagsKHR::PRE_MULTIPLIED
    } else {
        CompositeAlphaFlagsKHR::INHERIT
    };
    log::debug!("  using mode {}", composite_alpha.as_raw());

    // Swapchain extent
    let extent = {
        if capabilities.current_extent.width != std::u32::MAX {
            capabilities.current_extent
        } else {
            let min = capabilities.min_image_extent;
            let max = capabilities.max_image_extent;
            let width = WIDTH.min(max.width).max(min.width);
            let height = HEIGHT.min(max.height).max(min.height);
            vk::Extent2D { width, height }
        }
    };
    log::debug!("Swapchain extent: {extent:?}");

    // Swapchain image count
    let image_count = 3.clamp(capabilities.min_image_count, capabilities.max_image_count);
    log::debug!(
        "Swapchain image count ([{}; {}]): {}",
        capabilities.min_image_count,
        capabilities.max_image_count,
        image_count
    );

    // Swapchain
    let families_indices = [
        vulkan_context.graphics_q_index,
        vulkan_context.present_q_index,
    ];
    let create_info = {
        let mut builder = vk::SwapchainCreateInfoKHR::default()
            .surface(vulkan_context.surface_khr)
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);

        builder = if vulkan_context.graphics_q_index != vulkan_context.present_q_index {
            builder
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&families_indices)
        } else {
            builder.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        };

        builder
            .pre_transform(capabilities.current_transform)
            .composite_alpha(composite_alpha)
            .present_mode(present_mode)
            .clipped(true)
    };

    let swapchain = SwapchainDevice::new(&vulkan_context.instance, &vulkan_context.device);
    let swapchain_khr = unsafe { swapchain.create_swapchain(&create_info, None)? };

    // Swapchain images and image views
    let images = unsafe { swapchain.get_swapchain_images(swapchain_khr)? };
    let views = images
        .iter()
        .map(|image| {
            let create_info = vk::ImageViewCreateInfo::default()
                .image(*image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format.format)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            unsafe { vulkan_context.device.create_image_view(&create_info, None) }
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok((
        swapchain,
        swapchain_khr,
        extent,
        format.format,
        images,
        views,
    ))
}

fn create_vulkan_render_pass(
    device: &Device,
    format: vk::Format,
) -> Result<vk::RenderPass, VulkanError> {
    log::debug!("Creating vulkan render pass");
    let attachment_descs = [vk::AttachmentDescription::default()
        .format(format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)];

    let color_attachment_refs = [vk::AttachmentReference::default()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];

    let subpass_descs = [vk::SubpassDescription::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&color_attachment_refs)];

    let subpass_deps = [vk::SubpassDependency::default()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(
            vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        )];

    let render_pass_info = vk::RenderPassCreateInfo::default()
        .attachments(&attachment_descs)
        .subpasses(&subpass_descs)
        .dependencies(&subpass_deps);

    Ok(unsafe { device.create_render_pass(&render_pass_info, None)? })
}

fn create_vulkan_framebuffers(
    device: &Device,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
    image_views: &[vk::ImageView],
) -> Result<Vec<vk::Framebuffer>, VulkanError> {
    log::debug!("Creating {} vulkan framebuffers", image_views.len());
    Ok(image_views
        .iter()
        .map(|view| [*view])
        .map(|attachments| {
            let framebuffer_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(extent.width)
                .height(extent.height)
                .layers(1);
            unsafe { device.create_framebuffer(&framebuffer_info, None) }
        })
        .collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn record_command_buffers(
    device: &Device,
    _command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
    framebuffer: vk::Framebuffer,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
    renderer: &mut Renderer,
    draw_data: &DrawData,
) -> Result<(), VulkanError> {
    unsafe { device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())? };

    let command_buffer_begin_info =
        vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe { device.begin_command_buffer(command_buffer, &command_buffer_begin_info)? };

    let render_pass_begin_info = vk::RenderPassBeginInfo::default()
        .render_pass(render_pass)
        .framebuffer(framebuffer)
        .render_area(vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        })
        .clear_values(&[vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 0.0],
            },
        }]);

    unsafe {
        device.cmd_begin_render_pass(
            command_buffer,
            &render_pass_begin_info,
            vk::SubpassContents::INLINE,
        )
    };

    renderer.cmd_draw(command_buffer, draw_data)?;

    unsafe { device.cmd_end_render_pass(command_buffer) };

    unsafe { device.end_command_buffer(command_buffer)? };

    Ok(())
}
