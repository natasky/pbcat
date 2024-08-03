use std::{io::Write as _, time::Duration};

const POLL_INTERVAL: Duration = Duration::from_millis(500);

fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt::Subscriber::builder()
            .with_writer(std::io::stderr)
            .with_env_filter(tracing_subscriber::EnvFilter::from_env("PBCAT_LOG"))
            .finish(),
    )?;

    tracing::info!("Initializing");

    let mut clipboard = arboard::Clipboard::new()?;
    let mut last_text: Option<String> = None;
    let mut last_update_count = get_update_count();

    tracing::info!("Started");
    loop {
        std::thread::sleep(POLL_INTERVAL);
        let current_update_count = get_update_count();
        tracing::debug!(current_update_count);

        if current_update_count == last_update_count {
            tracing::debug!("No update");
            continue;
        }

        last_update_count = current_update_count;
        let Ok(current_text) = clipboard.get_text() else {
            // E.g. no text content.
            tracing::debug!("No text");
            continue;
        };

        if last_text.as_ref() == Some(&current_text) {
            tracing::debug!("No change");
            continue;
        }

        print!("{}\0", current_text);
        std::io::stdout().flush()?;

        last_text = Some(current_text)
    }
}

#[cfg(target_os = "windows")]
fn get_update_count() -> u64 {
    unsafe { windows::Win32::System::DataExchange::GetClipboardSequenceNumber() as u64 }
}

#[cfg(target_os = "macos")]
fn get_update_count() -> u64 {
    unsafe { NSPasteboard.generalPasteboard().changeCount() as u64 }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn get_update_count() -> u64 {
    // Fallback just checks contents every second.

    use std::{cell::LazyCell, sync::Mutex, time::Instant};

    static FIRST_CHECK_INSTANT: Mutex<LazyCell<Instant>> =
        Mutex::new(LazyCell::new(|| Instant::now()));

    FIRST_CHECK_INSTANT.lock().unwrap().elapsed().as_secs()
}
