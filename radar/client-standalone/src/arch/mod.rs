#[cfg(windows)]
mod windows;

#[cfg(unix)]
mod linux;

pub fn show_critical_error(message: &str) {
    #[cfg(windows)]
    windows::show_critical_error(message);

    #[cfg(unix)]
    linux::show_critical_error(message);
}
