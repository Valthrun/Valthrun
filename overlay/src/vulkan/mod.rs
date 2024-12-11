use ash::vk;
use frame::FrameData;
use imgui_rs_vulkan_renderer::{
    Options,
    Renderer,
};
use imgui_winit_support::winit::{
    dpi::PhysicalSize,
    window::Window,
};
use render::{
    record_command_buffers,
    Swapchain,
    VulkanContext,
};

use crate::{
    PerfTracker,
    RenderBackend,
    VulkanError,
};

mod buffer;
mod debug;
mod driver;
mod frame;
mod instance;
mod render;
mod texture;

pub struct VulkanRenderBackend {
    vulkan_context: VulkanContext,
    swapchain: Swapchain,

    frame_data: Vec<FrameData>,
    frame_data_index: usize,

    imgui_renderer: Renderer,
    dirty_swapchain: bool,
}

impl VulkanRenderBackend {
    pub fn new(window: &Window, imgui: &mut imgui::Context) -> Result<Self, VulkanError> {
        let vulkan_context = VulkanContext::new(&window)?;
        let frame_data = vec![
            FrameData::new(&vulkan_context)?,
            // FrameData::new(&vulkan_context)?,
        ];
        let swapchain = Swapchain::new(&vulkan_context)?;

        let imgui_renderer = Renderer::with_default_allocator(
            &vulkan_context.instance,
            vulkan_context.physical_device,
            vulkan_context.device.clone(),
            vulkan_context.graphics_queue,
            frame_data[0].command_pool, // Just any pool will do. Only one time thing
            swapchain.render_pass,
            imgui,
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

        Ok(Self {
            vulkan_context,
            swapchain,

            frame_data,
            frame_data_index: 0,

            imgui_renderer,

            dirty_swapchain: true,
        })
    }
}

impl RenderBackend for VulkanRenderBackend {
    fn update_fonts_texture(&mut self, imgui: &mut imgui::Context) {
        self.frame_data_index = self.frame_data_index.wrapping_add(1);
        let frame_data = &self.frame_data[self.frame_data_index % self.frame_data.len()];

        if let Err(err) = self.imgui_renderer.update_fonts_texture(
            self.vulkan_context.graphics_queue,
            frame_data.command_pool,
            imgui,
        ) {
            log::warn!("Failed to update fonts texture: {}", err);
        } else {
            log::debug!("Updated font texture successfully");
        }
    }

    fn render_frame(
        &mut self,
        perf: &mut PerfTracker,
        window: &Window,
        draw_data: &imgui::DrawData,
    ) {
        self.frame_data_index = self.frame_data_index.wrapping_add(1);
        let frame_data = &self.frame_data[self.frame_data_index % self.frame_data.len()];

        // If swapchain must be recreated wait for windows to not be minimized anymore
        if self.dirty_swapchain {
            let PhysicalSize { width, height } = window.inner_size();
            if width > 0 && height > 0 {
                log::debug!("Recreate swapchain");
                self.swapchain
                    .recreate(&self.vulkan_context)
                    .expect("Failed to recreate swapchain");
                self.imgui_renderer
                    .set_render_pass(self.swapchain.render_pass)
                    .expect("Failed to rebuild renderer pipeline");
                self.dirty_swapchain = false;
            } else {
                /* No need to render a frame when the window size is zero */
                return;
            }
        }

        unsafe {
            self.vulkan_context
                .device
                .wait_for_fences(&[frame_data.render_fence], true, u64::MAX)
                .expect("failed to wait for render fence");
        };

        perf.mark("fence");
        let next_image_result = unsafe {
            self.swapchain.loader.acquire_next_image(
                self.swapchain.khr,
                std::u64::MAX,
                frame_data.semaphore_image_available,
                vk::Fence::null(),
            )
        };
        let image_index = match next_image_result {
            Ok((image_index, _)) => image_index,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.dirty_swapchain = true;
                return;
            }
            Err(error) => {
                panic!("Error while acquiring next image. Cause: {}", error)
            }
        };
        unsafe {
            self.vulkan_context
                .device
                .reset_fences(&[frame_data.render_fence])
                .expect("failed to reset fences");
        };

        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let wait_semaphores = [frame_data.semaphore_image_available];
        let signal_semaphores = [frame_data.semaphore_render_finished];

        // Re-record commands to draw geometry
        record_command_buffers(
            &self.vulkan_context.device,
            frame_data.command_pool,
            frame_data.command_buffer,
            self.swapchain.framebuffers[image_index as usize],
            self.swapchain.render_pass,
            self.swapchain.extent,
            &mut self.imgui_renderer,
            &draw_data,
        )
        .expect("Failed to record command buffer");

        let command_buffers = [frame_data.command_buffer];
        let submit_info = [vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)];

        perf.mark("before submit");
        unsafe {
            self.vulkan_context
                .device
                .queue_submit(
                    self.vulkan_context.graphics_queue,
                    &submit_info,
                    frame_data.render_fence,
                )
                .expect("Failed to submit work to gpu.")
        };
        perf.mark("after submit");

        let swapchains = [self.swapchain.khr];
        let images_indices = [image_index];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&images_indices);

        let present_result = unsafe {
            self.swapchain
                .loader
                .queue_present(self.vulkan_context.present_queue, &present_info)
        };
        match present_result {
            Ok(is_suboptimal) if is_suboptimal => {
                self.dirty_swapchain = true;
            }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.dirty_swapchain = true;
            }
            Err(error) => panic!("Failed to present queue. Cause: {}", error),
            _ => {}
        }
        perf.mark("present");
    }
}

impl Drop for VulkanRenderBackend {
    fn drop(&mut self) {
        if let Err(err) = unsafe { self.vulkan_context.device.device_wait_idle() } {
            log::warn!("Failed to wait for device idle: {}", err);
        };
    }
}
