use std::ffi::NulError;

use imgui_rs_vulkan_renderer::RendererError;
use imgui_winit_support::winit::error::OsError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, OverlayError>;
pub use ash::{
    vk::Result as VkResult,
    LoadingError,
};

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
    VulkanError(#[from] VkResult),

    #[error("render error: {0}")]
    RenderError(#[from] RendererError),

    #[error("a parameter contains the null character")]
    ParameterContainsNull(#[from] NulError),

    #[error("current exe path is invalid: {0}")]
    ExePathInvalid(std::io::Error),

    #[error("the exe must be located within a directory")]
    ExePathMissingParentDirectory,

    #[error("failed to write the vulkan dll")]
    VulkanDllError(std::io::Error),

    #[error("failed to create a vulkan instance: {0}")]
    VulkanInstanceCreationFailed(VkResult),

    #[error("failed to create a vulkan surface: {0}")]
    VulkanSurfaceCreationFailed(VkResult),
}
