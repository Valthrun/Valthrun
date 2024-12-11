use std::mem;

use ash::{
    vk,
    Device,
};
use imgui_rs_vulkan_renderer::RendererResult;

#[allow(dead_code)]
pub fn create_and_fill_buffer<T: Copy>(
    data: &[T],
    device: &Device,
    usage: vk::BufferUsageFlags,
    mem_properties: vk::PhysicalDeviceMemoryProperties,
) -> RendererResult<(vk::Buffer, vk::DeviceMemory)> {
    let size = data.len() * mem::size_of::<T>();
    let (buffer, memory) = create_buffer(size, device, usage, mem_properties)?;
    update_buffer_content(device, memory, data)?;
    Ok((buffer, memory))
}

pub fn create_buffer(
    size: usize,
    device: &Device,
    usage: vk::BufferUsageFlags,
    mem_properties: vk::PhysicalDeviceMemoryProperties,
) -> RendererResult<(vk::Buffer, vk::DeviceMemory)> {
    let buffer_info = vk::BufferCreateInfo::default()
        .size(size as _)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = unsafe { device.create_buffer(&buffer_info, None)? };

    let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
    let mem_type = find_memory_type(
        mem_requirements,
        mem_properties,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );

    let alloc_info = vk::MemoryAllocateInfo::default()
        .allocation_size(mem_requirements.size)
        .memory_type_index(mem_type);
    let memory = unsafe { device.allocate_memory(&alloc_info, None)? };
    unsafe { device.bind_buffer_memory(buffer, memory, 0)? };

    Ok((buffer, memory))
}

pub fn update_buffer_content<T: Copy>(
    device: &Device,
    buffer_memory: vk::DeviceMemory,
    data: &[T],
) -> RendererResult<()> {
    unsafe {
        let size = (data.len() * mem::size_of::<T>()) as _;

        let data_ptr = device.map_memory(buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;
        let mut align = ash::util::Align::new(data_ptr, mem::align_of::<T>() as _, size);
        align.copy_from_slice(&data);
        device.unmap_memory(buffer_memory);
    };
    Ok(())
}

pub fn find_memory_type(
    requirements: vk::MemoryRequirements,
    mem_properties: vk::PhysicalDeviceMemoryProperties,
    required_properties: vk::MemoryPropertyFlags,
) -> u32 {
    for i in 0..mem_properties.memory_type_count {
        if requirements.memory_type_bits & (1 << i) != 0
            && mem_properties.memory_types[i as usize]
                .property_flags
                .contains(required_properties)
        {
            return i;
        }
    }
    panic!("Failed to find suitable memory type.")
}
