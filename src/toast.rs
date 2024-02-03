use notify_rust::Notification;

pub fn toast(message: &str) {
    Notification::new()
        .summary("Error in hyprswitch")
        .body(message)
        .appname("hyprswitch")
        .timeout(0)
        .show().expect("Failed to send notification");
}