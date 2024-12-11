use ash::{
    vk,
    Device,
};
use imgui_rs_vulkan_renderer::RendererResult;

use super::buffer::*;

/// Helper struct representing a sampled texture.
#[allow(dead_code)]
pub struct Texture {
    pub image: vk::Image,
    image_mem: vk::DeviceMemory,
    pub image_view: vk::ImageView,
    pub sampler: vk::Sampler,
}

impl Texture {
    /// Create a texture from an `u8` array containing an rgba image.
    ///
    /// The image data is device local and it's format is R8G8B8A8_UNORM.
    ///     
    /// # Arguments
    ///
    /// * `device` - The Vulkan logical device.
    /// * `transfer_queue` - The queue with transfer capabilities to execute commands.
    /// * `command_pool` - The command pool used to create a command buffer used to record commands.
    /// * `mem_properties` - The memory properties of the Vulkan physical device.
    /// * `width` - The width of the image.
    /// * `height` - The height of the image.
    /// * `data` - The image data.
    #[allow(dead_code)]
    pub fn from_rgba8(
        device: &Device,
        transfer_queue: vk::Queue,
        command_pool: vk::CommandPool,
        mem_properties: vk::PhysicalDeviceMemoryProperties,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> RendererResult<Self> {
        let (texture, staging_buff, staging_mem) =
            execute_one_time_commands(device, transfer_queue, command_pool, |buffer| {
                Self::cmd_from_rgba(device, buffer, mem_properties, width, height, data)
            })??;

        unsafe {
            device.destroy_buffer(staging_buff, None);
            device.free_memory(staging_mem, None);
        }

        Ok(texture)
    }

    fn cmd_from_rgba(
        device: &Device,
        command_buffer: vk::CommandBuffer,
        mem_properties: vk::PhysicalDeviceMemoryProperties,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> RendererResult<(Self, vk::Buffer, vk::DeviceMemory)> {
        let (buffer, buffer_mem) = create_and_fill_buffer(
            data,
            device,
            vk::BufferUsageFlags::TRANSFER_SRC,
            mem_properties,
        )?;

        let (image, image_mem) = {
            let extent = vk::Extent3D {
                width,
                height,
                depth: 1,
            };

            let image_info = vk::ImageCreateInfo::default()
                .image_type(vk::ImageType::TYPE_2D)
                .extent(extent)
                .mip_levels(1)
                .array_layers(1)
                .format(vk::Format::R8G8B8A8_UNORM)
                .tiling(vk::ImageTiling::OPTIMAL)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .samples(vk::SampleCountFlags::TYPE_1)
                .flags(vk::ImageCreateFlags::empty());

            let image = unsafe { device.create_image(&image_info, None)? };
            let mem_requirements = unsafe { device.get_image_memory_requirements(image) };
            let mem_type_index = find_memory_type(
                mem_requirements,
                mem_properties,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            );

            let alloc_info = vk::MemoryAllocateInfo::default()
                .allocation_size(mem_requirements.size)
                .memory_type_index(mem_type_index);
            let memory = unsafe {
                let mem = device.allocate_memory(&alloc_info, None)?;
                device.bind_image_memory(image, mem, 0)?;
                mem
            };

            (image, memory)
        };

        // Transition the image layout and copy the buffer into the image
        // and transition the layout again to be readable from fragment shader.
        {
            let mut barrier = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);

            unsafe {
                device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[barrier],
                )
            };

            let region = vk::BufferImageCopy::default()
                .buffer_offset(0)
                .buffer_row_length(0)
                .buffer_image_height(0)
                .image_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                .image_extent(vk::Extent3D {
                    width,
                    height,
                    depth: 1,
                });
            unsafe {
                device.cmd_copy_buffer_to_image(
                    command_buffer,
                    buffer,
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[region],
                )
            }

            barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
            barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
            barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

            unsafe {
                device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[barrier],
                )
            };
        }

        let image_view = {
            let create_info = vk::ImageViewCreateInfo::default()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::R8G8B8A8_UNORM)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            unsafe { device.create_image_view(&create_info, None)? }
        };

        let sampler = {
            let sampler_info = vk::SamplerCreateInfo::default()
                .mag_filter(vk::Filter::LINEAR)
                .min_filter(vk::Filter::LINEAR)
                .address_mode_u(vk::SamplerAddressMode::REPEAT)
                .address_mode_v(vk::SamplerAddressMode::REPEAT)
                .address_mode_w(vk::SamplerAddressMode::REPEAT)
                .anisotropy_enable(false)
                .max_anisotropy(1.0)
                .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
                .unnormalized_coordinates(false)
                .compare_enable(false)
                .compare_op(vk::CompareOp::ALWAYS)
                .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
                .mip_lod_bias(0.0)
                .min_lod(0.0)
                .max_lod(1.0);
            unsafe { device.create_sampler(&sampler_info, None)? }
        };

        let texture = Self {
            image,
            image_mem,
            image_view,
            sampler,
        };

        Ok((texture, buffer, buffer_mem))
    }

    /// Free texture's resources.
    #[allow(dead_code)]
    pub fn destroy(&mut self, device: &Device) {
        unsafe {
            device.destroy_sampler(self.sampler, None);
            device.destroy_image_view(self.image_view, None);
            device.destroy_image(self.image, None);
            device.free_memory(self.image_mem, None);
        }
    }
}

fn execute_one_time_commands<R, F: FnOnce(vk::CommandBuffer) -> R>(
    device: &Device,
    queue: vk::Queue,
    pool: vk::CommandPool,
    executor: F,
) -> RendererResult<R> {
    let command_buffer = {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(pool)
            .command_buffer_count(1);

        unsafe { device.allocate_command_buffers(&alloc_info)?[0] }
    };
    let command_buffers = [command_buffer];

    // Begin recording
    {
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe { device.begin_command_buffer(command_buffer, &begin_info)? };
    }

    // Execute user function
    let executor_result = executor(command_buffer);

    // End recording
    unsafe { device.end_command_buffer(command_buffer)? };

    // Submit and wait
    {
        let submit_info = vk::SubmitInfo::default().command_buffers(&command_buffers);
        let submit_infos = [submit_info];
        unsafe {
            device.queue_submit(queue, &submit_infos, vk::Fence::null())?;
            device.queue_wait_idle(queue)?;
        };
    }

    // Free
    unsafe { device.free_command_buffers(pool, &command_buffers) };

    Ok(executor_result)
}
