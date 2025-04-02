pub fn show_critical_error(message: &str) {
    for line in message.lines() {
        log::error!("{}", line);
    }
}
