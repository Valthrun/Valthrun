use ash::vk;

use super::render::VulkanContext;
use crate::VulkanError;

pub struct FrameData {
    device: ash::Device,

    pub command_pool: vk::CommandPool,
    pub command_buffer: vk::CommandBuffer,

    pub semaphore_image_available: vk::Semaphore,
    pub semaphore_render_finished: vk::Semaphore,

    pub render_fence: vk::Fence,
}

impl FrameData {
    pub fn new(instance: &VulkanContext) -> Result<Self, VulkanError> {
        let device = instance.device.clone();

        let command_pool = {
            let command_pool_info = vk::CommandPoolCreateInfo::default()
                .queue_family_index(instance.graphics_q_index)
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
            unsafe { device.create_command_pool(&command_pool_info, None)? }
        };

        let command_buffer = {
            let allocate_info = vk::CommandBufferAllocateInfo::default()
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);

            unsafe { device.allocate_command_buffers(&allocate_info)?[0] }
        };

        let semaphore_image_available = {
            let semaphore_info = vk::SemaphoreCreateInfo::default();
            unsafe { device.create_semaphore(&semaphore_info, None)? }
        };

        let semaphore_render_finished = {
            let semaphore_info = vk::SemaphoreCreateInfo::default();
            unsafe { device.create_semaphore(&semaphore_info, None)? }
        };

        let render_fence = {
            let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
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
