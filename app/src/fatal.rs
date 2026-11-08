//! Uygulama başlayamadığında kullanıcıya hata gösterip çıkar.

pub fn fatal(msg: &str) -> ! {
    log::error!("{msg}");
    show(msg);
    std::process::exit(1)
}

#[cfg(windows)]
fn show(msg: &str) {
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
    let wide = |s: &str| s.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
    let (text, title) = (wide(msg), wide("WhatsappRS"));
    unsafe {
        MessageBoxW(None, PCWSTR(text.as_ptr()), PCWSTR(title.as_ptr()), MB_OK | MB_ICONERROR);
    }
}

#[cfg(not(windows))]
fn show(msg: &str) {
    eprintln!("WhatsappRS: {msg}");
}
