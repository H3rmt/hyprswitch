pub fn toast(message: &str) {
    // only toast if feature is enabled, else log to stderr
    #[cfg(feature = "toast")] {
        use notify_rust::Notification;
        Notification::new()
            .summary("Error in window_switcher")
            .body(message)
            .appname("window_switcher")
            .timeout(0)
            .show().expect("Failed to send notification");
    }

    eprintln!("Error: {}", message);
}