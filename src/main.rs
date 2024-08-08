use std::{
    io::{BufRead as _, Write as _},
    time::Duration,
};

use clap::Parser;

const POLL_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Parser)]
struct Options {
    // Write to clipboard instead of reading from it.
    #[arg(long)]
    write: bool,
}

fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt::Subscriber::builder()
            .with_writer(std::io::stderr)
            .with_env_filter(tracing_subscriber::EnvFilter::from_env("PBCAT_LOG"))
            .finish(),
    )?;

    let options = Options::parse();

    if options.write {
        write_forever()?;
    } else {
        read_forever()?;
    }

    Ok(())
}

#[tracing::instrument(level = "info")]
fn read_forever() -> Result<(), anyhow::Error> {
    let mut clipboard = arboard::Clipboard::new()?;
    let mut last_text: Option<String> = None;
    let mut last_update_count = get_update_count();
    tracing::info!("Started");

    loop {
        std::thread::sleep(POLL_INTERVAL);
        let current_update_count = get_update_count();
        tracing::trace!(current_update_count);

        if current_update_count == last_update_count {
            tracing::trace!("No update");
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

#[tracing::instrument(level = "info")]
fn write_forever() -> Result<(), anyhow::Error> {
    let mut clipboard = arboard::Clipboard::new()?;
    let mut buffer = Vec::<u8>::new();
    tracing::info!("Started");

    loop {
        buffer.clear();
        let mut size = std::io::stdin().lock().read_until(b'\0', &mut buffer)?;

        if size == 0 {
            tracing::info!("End of input");
            return Ok(());
        }

        if buffer[size - 1] == b'\0' {
            size -= 1;
        }

        let text = String::from_utf8_lossy(&buffer[..size]);
        let current_text = clipboard.get_text().ok();

        if Some(text.as_ref()) != current_text.as_ref().map(|s| s.as_str()) {
            tracing::debug!(?text, "Writing to clipboard");
            clipboard.set_text(text)?;
        } else {
            tracing::debug!("Skipping identical input");
        }
    }
}

#[cfg(target_os = "windows")]
fn get_update_count() -> u64 {
    unsafe { windows::Win32::System::DataExchange::GetClipboardSequenceNumber() as u64 }
}

#[cfg(target_os = "macos")]
fn get_update_count() -> u64 {
    unsafe { objc2_app_kit::NSPasteboard::generalPasteboard().changeCount() as u64 }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn get_update_count() -> u64 {
    // Fallback just checks contents every second.

    use std::{cell::LazyCell, sync::Mutex, time::Instant};

    static FIRST_CHECK_INSTANT: Mutex<LazyCell<Instant>> =
        Mutex::new(LazyCell::new(|| Instant::now()));

    FIRST_CHECK_INSTANT.lock().unwrap().elapsed().as_secs()
}
