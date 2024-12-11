use std::{
    collections::BTreeMap,
    env,
    ffi::{
        CStr,
        CString,
    },
};

use ash::{
    vk,
    Entry,
};
use imgui_winit_support::winit::window::Window;
use winit::raw_window_handle::HasDisplayHandle;

use crate::{
    vulkan::debug,
    VulkanError,
};

struct ExtensionBuilder {
    supported_extensions: BTreeMap<String, u32>,
    supported_layers: BTreeMap<String, u32>,

    requested_extensions: Vec<CString>,
    requested_layers: Vec<CString>,
}

impl ExtensionBuilder {
    pub fn new(entry: &Entry) -> Result<Self, VulkanError> {
        let supported_extensions = unsafe { entry.enumerate_instance_extension_properties(None)? }
            .into_iter()
            .map(|ext| {
                (
                    unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) }
                        .to_string_lossy()
                        .to_string(),
                    ext.spec_version,
                )
            })
            .collect::<BTreeMap<_, _>>();

        let supported_layers = unsafe { entry.enumerate_instance_layer_properties()? }
            .into_iter()
            .map(|layer| {
                (
                    unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) }
                        .to_string_lossy()
                        .to_string(),
                    layer.spec_version,
                )
            })
            .collect::<BTreeMap<_, _>>();

        Ok(Self {
            supported_extensions,
            supported_layers,

            requested_extensions: Vec::new(),
            requested_layers: Vec::new(),
        })
    }

    pub fn add_extension(
        &mut self,
        name: impl Into<CString>,
        required: bool,
    ) -> Result<(), VulkanError> {
        let cname: CString = name.into();

        let name = cname.to_string_lossy().to_string();
        let is_supported = self.supported_extensions.contains_key(&name);

        if !is_supported {
            if required {
                log::warn!("Required Vulkan extension {name} requested but Vulkan does not support this extension. This might causes errors");
                //return Err(OverlayError::VulkanRequiredExtensionUnsupported(name));
            } else {
                log::debug!(
                    "Skipping registering vulkan extension {} as it's not supported.",
                    name
                );
                return Ok(());
            }
        }

        self.requested_extensions.push(cname);
        Ok(())
    }

    pub fn add_layer(
        &mut self,
        name: impl Into<CString>,
        required: bool,
    ) -> Result<(), VulkanError> {
        let cname: CString = name.into();

        let name = cname.to_string_lossy().to_string();
        let is_supported = self.supported_layers.contains_key(&name);

        if !is_supported {
            if required {
                log::warn!("Required Vulkan layer extension {name} requested but Vulkan does not support this extension. This might causes errors");
                //return Err(OverlayError::VulkanRequiredLayerUnsupported(name));
            } else {
                log::debug!(
                    "Skipping registering vulkan layer {} as it's not supported.",
                    name
                );
                return Ok(());
            }
        }

        self.requested_layers.push(cname);
        Ok(())
    }

    fn enabled_extension_names(&self) -> Vec<*const i8> {
        self.requested_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>()
    }

    fn enabled_layer_names(&self) -> Vec<*const i8> {
        self.requested_layers
            .iter()
            .map(|layer| layer.as_ptr())
            .collect::<Vec<_>>()
    }
}

pub fn create_vulkan_instance(
    entry: &Entry,
    window: &Window,
) -> Result<ash::Instance, VulkanError> {
    {
        let instance_version = match unsafe { entry.try_enumerate_instance_version()? } {
            Some(version) => version,
            None => vk::make_api_version(0, 1, 0, 0),
        };
        log::debug!(
            "Detected vulkan version {}.{}.{}",
            vk::api_version_major(instance_version),
            vk::api_version_minor(instance_version),
            vk::api_version_patch(instance_version)
        );
    }

    let ext_builder = {
        let mut ext_builder = ExtensionBuilder::new(entry)?;

        log::trace!("  Available extensions:");
        for (extension, version) in &ext_builder.supported_extensions {
            log::trace!(
                "  - {} (v{}.{}.{})",
                extension,
                vk::api_version_major(*version),
                vk::api_version_minor(*version),
                vk::api_version_patch(*version)
            );
        }

        log::trace!("  Available layers:");
        for (layer, version) in &ext_builder.supported_layers {
            log::trace!(
                "  - {} (v{}.{}.{})",
                layer,
                vk::api_version_major(*version),
                vk::api_version_minor(*version),
                vk::api_version_patch(*version)
            );
        }

        ext_builder.add_extension(debug::extension_name(), true)?;
        for extension in
            ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw())?
        {
            ext_builder.add_extension(unsafe { CStr::from_ptr(*extension) }, true)?;
        }

        if env::var("VTOL_KHRONOS_VALIDATION").map_or(false, |var| var == "1") {
            ext_builder.add_layer(c"VK_LAYER_KHRONOS_validation", true)?;
        }

        ext_builder
    };

    let app_info = vk::ApplicationInfo::default()
        .application_name(c"No Title")
        .application_version(vk::make_api_version(0, 1, 0, 0))
        .engine_name(c"No Engine")
        .engine_version(vk::make_api_version(0, 1, 0, 0))
        .api_version(vk::make_api_version(0, 1, 1, 0));

    let mut debug_messanger_ext = debug::create_extension_info();
    let enabled_extension_names = ext_builder.enabled_extension_names();
    let enabled_layer_names = ext_builder.enabled_layer_names();
    let instance_create_info = vk::InstanceCreateInfo::default()
        .application_info(&app_info)
        .enabled_extension_names(&enabled_extension_names)
        .enabled_layer_names(&enabled_layer_names)
        .push_next(&mut debug_messanger_ext);

    log::debug!("Creating Vulkan instance");
    {
        log::trace!(" Extensions:");
        for ext in &ext_builder.requested_extensions {
            log::debug!("  - {}", ext.to_string_lossy());
        }
        log::trace!(" Layers:");
        for layer in &ext_builder.requested_layers {
            log::debug!("  - {}", layer.to_string_lossy());
        }
    }

    let instance = unsafe {
        entry
            .create_instance(&instance_create_info, None)
            .map_err(VulkanError::InstanceCreationFailed)?
    };

    Ok(instance)
}
