//! `data\logs\` altına dönüşümlü log (en fazla ~5 MB).

use flexi_logger::{Cleanup, Criterion, FileSpec, Logger, LoggerHandle, Naming};
use std::path::Path;
use std::sync::OnceLock;

static HANDLE: OnceLock<LoggerHandle> = OnceLock::new();

pub fn init(logs_dir: &Path) {
    let result = Logger::try_with_str("info").and_then(|l| {
        l.log_to_file(FileSpec::default().directory(logs_dir).basename("whatsapprs").suppress_timestamp())
            .rotate(Criterion::Size(1_000_000), Naming::Numbers, Cleanup::KeepLogFiles(4))
            .start()
    });
    match result {
        Ok(h) => {
            let _ = HANDLE.set(h);
        }
        Err(e) => eprintln!("log başlatılamadı: {e}"),
    }
}
