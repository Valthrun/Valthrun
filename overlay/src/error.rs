use std::ffi::NulError;

use glium::backend::glutin::DisplayCreationError;
use imgui_glium_renderer::RendererError;
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

    #[error("{0}")]
    DisplayError(#[from] DisplayCreationError),

    #[error("{0}")]
    RenderError(#[from] RendererError),

    #[error("{0}")]
    WindowsError(#[from] windows::core::Error)
}