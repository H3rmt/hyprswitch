use notify_rust::Notification;

pub fn toast(message: &str) {
    Notification::new()
        .summary("Error in window_switcher")
        .body(message)
        .appname("window_switcher")
        .timeout(0)
        .show().expect("Failed to send notification");
}