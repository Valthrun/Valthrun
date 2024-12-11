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

    #[error("vulkan: {0}")]
    Vulkan(#[from] VulkanError),

    #[error("invalid window name ({0})")]
    WindowInvalidName(NulError),

    #[error("the target window could not be found")]
    WindowNotFound,

    #[error("failed to create overlay window")]
    WindowCreateFailed(#[from] OsError),

    #[error("{0}")]
    WindowsError(#[from] windows::core::Error),

    #[error("a parameter contains the null character")]
    ParameterContainsNull(#[from] NulError),

    #[error("current exe path is invalid: {0}")]
    ExePathInvalid(std::io::Error),

    #[error("the exe must be located within a directory")]
    ExePathMissingParentDirectory,

    #[error("target font is not a true type font")]
    FontUnsupported,

    #[error("failed to parse font face: {0}")]
    FontFaceParsingError(#[from] ttf_parser::FaceParsingError),

    #[error("desktop window manager has composition disabled")]
    DwmCompositionDisabled,
}

#[derive(Error, Debug)]
pub enum VulkanError {
    #[error("vulkan-1.dll could not be found ({0})")]
    DllNotFound(#[from] LoadingError),

    #[error("vulkan: {0}")]
    VulkanError(#[from] VkResult),

    #[error("failed to create a instance: {0}")]
    InstanceCreationFailed(VkResult),

    #[error("failed to create a surface: {0}")]
    SurfaceCreationFailed(VkResult),

    #[error("composite alpha is unsupported")]
    CompositeAlphaUnsupported,

    #[error("missing required extension: {0}")]
    RequiredExtensionUnsupported(String),

    #[error("missing required layer: {0}")]
    RequiredLayerUnsupported(String),

    #[error("render error: {0}")]
    RenderError(#[from] RendererError),
}
