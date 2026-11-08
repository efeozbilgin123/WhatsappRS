//! Güvenlik ve davranış kararları (saf fonksiyonlar).

pub const WHATSAPP_URL: &str = "https://web.whatsapp.com/";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermKind { Microphone, Camera, Notifications, Other }

/// Yalnızca `https://web.whatsapp.com` (varsayılan port) mı?
fn is_whatsapp(url: &str) -> bool {
    match url::Url::parse(url) {
        Ok(u) => u.scheme() == "https" && u.host_str() == Some("web.whatsapp.com") && u.port().is_none(),
        Err(_) => false,
    }
}

/// Kamera/mikrofon/bildirim izni yalnızca WhatsApp Web'e verilir.
pub fn allow_permission(origin: &str, kind: PermKind) -> bool {
    kind != PermKind::Other && is_whatsapp(origin)
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
        for k in [PermKind::Microphone, PermKind::Camera, PermKind::Notifications] {
            assert!(allow_permission("https://web.whatsapp.com", k));
            assert!(allow_permission("https://web.whatsapp.com/", k));
        }
    }
    #[test] fn perm_denies_other_kinds() { assert!(!allow_permission("https://web.whatsapp.com", PermKind::Other)); }
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
    #[test] fn second_instance_minimized() {
        assert!(!show_on_second_instance(&["WhatsappRS.exe".into(), "--minimized".into()]));
        assert!(show_on_second_instance(&["WhatsappRS.exe".into()]));
    }
}
