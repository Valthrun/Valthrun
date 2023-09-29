use std::ffi::NulError;

use ash::vk;
use imgui_rs_vulkan_renderer::RendererError;
use thiserror::Error;
use imgui_winit_support::winit::error::OsError;

pub type Result<T> = std::result::Result<T, OverlayError>;

#[derive(Error, Debug)]
pub enum OverlayError {
    #[error("no monitor available")]
    NoMonitorAvailable,

    #[error("invalid window name ({0})")]
    WindowInvalidName(NulError),

    #[error("the target window could not be found")]
    WindowNotFound,

    #[error("failed to create overlay window")]
    WindowCreateFailed(#[from] OsError),
    
    // #[error("{0}")]
    // DisplayError(#[from] DisplayCreationError),

    // #[error("{0}")]
    // RenderError(#[from] RendererError),

    #[error("{0}")]
    WindowsError(#[from] windows::core::Error),

    // #[error("generic error from vulkan: {0}")]
    // GenericError(#[from] Box<dyn std::error::Error>),

    #[error("vulkan: {0}")]
    VulkanError(#[from] vk::Result),

    #[error("render error: {0}")]
    RenderError(#[from] RendererError),
}
