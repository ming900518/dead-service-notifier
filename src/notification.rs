#[cfg(target_os = "linux")]
pub fn notification(name: &str, body: &str) {
    use notify_rust::{Hint, Notification, Timeout};

    Notification::new()
        .summary(name)
        .body(body)
        .appname("Dead Service Notifier")
        .hint(Hint::Resident(true))
        .timeout(Timeout::Never)
        .show()
        .unwrap();
}

#[cfg(target_os = "macos")]
pub fn notification(name: &str, body: &str) {
    use cacao::user_notifications::{Notification, NotificationAuthOption, NotificationCenter};

    NotificationCenter::request_authorization(&[
        NotificationAuthOption::Alert,
        NotificationAuthOption::Sound,
        NotificationAuthOption::Badge,
    ]);
    NotificationCenter::notify(Notification::new(name, body));
}

#[cfg(target_os = "windows")]
pub fn notification(name: &str, body: &str) {
    use winrt_notification::{Duration, Sound, Toast};

    Toast::new(Toast::POWERSHELL_APP_ID)
        .title("Dead Service Notifier")
        .text1(name)
        .text2(body)
        .sound(Some(Sound::SMS))
        .duration(Duration::Short)
        .show()
        .unwrap();
}
