//! Güvenlik ve davranış kararları (saf fonksiyonlar).

pub const WHATSAPP_URL: &str = "https://web.whatsapp.com/";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermKind { Microphone, Camera, Notifications, MultipleDownloads, Other }

/// Yalnızca `https://web.whatsapp.com` (varsayılan port) mı?
fn is_whatsapp(url: &str) -> bool {
    match url::Url::parse(url) {
        Ok(u) => u.scheme() == "https" && u.host_str() == Some("web.whatsapp.com") && u.port().is_none(),
        Err(_) => false,
    }
}

/// Kamera/mikrofon izni yalnızca WhatsApp Web'e verilir. Bildirim izni verilmez:
/// bildirimler bridge.js → Rust yolundan gider (aksi halde WebView2 kendi bildirimini de gösterir).
pub fn allow_permission(origin: &str, kind: PermKind) -> bool {
    matches!(kind, PermKind::Microphone | PermKind::Camera | PermKind::MultipleDownloads) && is_whatsapp(origin)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewWindowAction {
    OpenInApp,
    OpenExternal,
    /// Adres henüz belli değil (about:blank): gizli bir yardımcı pencere açılır, ilk gerçek adres
    /// varsayılan tarayıcıya gönderilir ve pencere kapatılır.
    OpenViaHelper,
    Deny,
}

/// WhatsApp'ın açmak istediği yeni pencere (PDF/medya önizleme, dış bağlantı) için karar.
pub fn new_window_action(url: &str) -> NewWindowAction {
    if url.is_empty() || url == "about:blank" {
        return NewWindowAction::OpenViaHelper;
    }
    if let Some(inner) = url.strip_prefix("blob:") {
        return if is_whatsapp(inner) { NewWindowAction::OpenInApp } else { NewWindowAction::Deny };
    }
    if is_whatsapp(url) {
        return NewWindowAction::OpenInApp;
    }
    match url::Url::parse(url) {
        Ok(u) if u.scheme() == "https" || u.scheme() == "http" => NewWindowAction::OpenExternal,
        _ => NewWindowAction::Deny,
    }
}

/// Otomatik başlatma kayıt değeri: boşluklu yollar için tırnaklı.
pub fn autostart_command(exe_path: &str) -> String {
    format!("\"{exe_path}\" --minimized")
}

/// Webview içinde kalabilecek adres mi? Değilse varsayılan tarayıcıda açılır.
pub fn is_internal_url(url: &str) -> bool {
    url == "about:blank" || is_whatsapp(url)
}

/// Pencere hem görünür hem odaktaysa bildirim gösterilmez.
pub fn should_show_toast(window_visible: bool, window_focused: bool) -> bool {
    !(window_visible && window_focused)
}

/// İkinci kopya `--minimized` ile (otomatik başlatma) açıldıysa pencere gösterilmez.
pub fn show_on_second_instance(args: &[String]) -> bool {
    !args.iter().any(|a| a == "--minimized")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn perm_allows_whatsapp_media() {
        for k in [PermKind::Microphone, PermKind::Camera] {
            assert!(allow_permission("https://web.whatsapp.com", k));
            assert!(allow_permission("https://web.whatsapp.com/", k));
        }
    }
    #[test] fn perm_denies_other_kinds() {
        assert!(!allow_permission("https://web.whatsapp.com", PermKind::Other));
        // Bildirimler bridge.js üzerinden gider; WebView2'nin kendi bildirimleri odak kuralını atlar.
        assert!(!allow_permission("https://web.whatsapp.com", PermKind::Notifications));
    }
    #[test] fn perm_denies_lookalikes() {
        for o in ["http://web.whatsapp.com", "https://web.whatsapp.com.evil.com", "https://evil.com/web.whatsapp.com",
                  "https://web.whatsapp.com:8443", "https://xweb.whatsapp.com", "https://whatsapp.com", "", "nonsense"] {
            assert!(!allow_permission(o, PermKind::Camera), "{o}");
        }
    }
    #[test] fn internal_urls() {
        assert!(is_internal_url("https://web.whatsapp.com/"));
        assert!(is_internal_url("https://web.whatsapp.com/send?phone=1"));
        assert!(is_internal_url("about:blank"));
        assert!(!is_internal_url("https://example.com/"));
        assert!(!is_internal_url("http://web.whatsapp.com/"));
        assert!(!is_internal_url("https://web.whatsapp.com.evil.com/"));
    }
    #[test] fn toast_policy() {
        assert!(!should_show_toast(true, true));
        assert!(should_show_toast(true, false));
        assert!(should_show_toast(false, false));
        assert!(should_show_toast(false, true));
    }
    #[test] fn autostart_command_is_quoted() {
        assert_eq!(
            autostart_command(r"C:\Users\Ali Veli\AppData\Local\Programs\WhatsappRS\WhatsappRS.exe"),
            r#""C:\Users\Ali Veli\AppData\Local\Programs\WhatsappRS\WhatsappRS.exe" --minimized"#
        );
    }
    #[test] fn perm_allows_multiple_downloads() {
        assert!(allow_permission("https://web.whatsapp.com", PermKind::MultipleDownloads));
        assert!(!allow_permission("https://evil.com", PermKind::MultipleDownloads));
    }
    #[test] fn new_window_actions() {
        assert_eq!(new_window_action("blob:https://web.whatsapp.com/3f2a-11"), NewWindowAction::OpenInApp);
        assert_eq!(new_window_action("https://web.whatsapp.com/"), NewWindowAction::OpenInApp);
        assert_eq!(new_window_action("https://example.com/a"), NewWindowAction::OpenExternal);
        assert_eq!(new_window_action("http://example.com/a"), NewWindowAction::OpenExternal);
        assert_eq!(new_window_action("blob:https://evil.com/x"), NewWindowAction::Deny);
        // WhatsApp bağlantıyı önce boş pencere açıp sonra adres vererek açabilir
        assert_eq!(new_window_action("about:blank"), NewWindowAction::OpenViaHelper);
        assert_eq!(new_window_action(""), NewWindowAction::OpenViaHelper);
        assert_eq!(new_window_action("javascript:alert(1)"), NewWindowAction::Deny);
        assert_eq!(new_window_action("file:///C:/Windows/system32/calc.exe"), NewWindowAction::Deny);
        assert_eq!(new_window_action("ms-settings:"), NewWindowAction::Deny);
    }
    #[test] fn second_instance_minimized() {
        assert!(!show_on_second_instance(&["WhatsappRS.exe".into(), "--minimized".into()]));
        assert!(show_on_second_instance(&["WhatsappRS.exe".into()]));
    }
}
