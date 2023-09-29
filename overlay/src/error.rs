use std::ffi::NulError;

use ash::{vk, LoadingError};
use imgui_rs_vulkan_renderer::RendererError;
use imgui_winit_support::winit::error::OsError;
use thiserror::Error;

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

    #[error("vulkan-1.dll could not be found ({0})")]
    VulkanDllNotFound(#[from] LoadingError),

    #[error("{0}")]
    WindowsError(#[from] windows::core::Error),

    #[error("vulkan: {0}")]
    VulkanError(#[from] vk::Result),

    #[error("render error: {0}")]
    RenderError(#[from] RendererError),

    #[error("a parameter contains the null character")]
    ParameterContainsNull(#[from] NulError),
}
