mod watch_clipboard;

use anyhow::{anyhow, Result};
use std::{
    io::{BufRead as _, Write as _},
    sync::mpsc,
    time::Duration,
};

const POLL_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Clone, PartialEq, Eq, Debug)]
enum Message {
    ClipboardTextChanged { text: String },
    ReceivedText { text: String },
    Exit,
}

impl Message {
    fn text(&self) -> Option<&str> {
        match self {
            Message::ClipboardTextChanged { text } => Some(&text),
            Message::ReceivedText { text } => Some(&text),
            Message::Exit => None,
        }
    }

    fn into_text(self) -> Option<String> {
        match self {
            Message::ClipboardTextChanged { text } => Some(text),
            Message::ReceivedText { text } => Some(text),
            Message::Exit => None,
        }
    }
}

fn main() -> Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt::Subscriber::builder()
            .with_writer(std::io::stderr)
            .with_env_filter(tracing_subscriber::EnvFilter::from_env("PBCAT_LOG"))
            .finish(),
    )?;

    let (sender, receiver) = mpsc::channel::<Message>();

    let threads = [
        std::thread::spawn({
            let sender = sender.clone();
            move || watch_input(sender)
        }),
        std::thread::spawn(move || watch_clipboard_forever(sender)),
        std::thread::spawn(move || handle_messages(receiver)),
    ];

    for thread in threads {
        thread
            .join()
            .map_err(|err| anyhow!("thread panicked: {:?}", err))??;
    }

    Ok(())
}

fn watch_input(output: mpsc::Sender<Message>) -> Result<()> {
    let mut buffer = Vec::<u8>::new();

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

        let text = String::from_utf8_lossy(&buffer[..size]).into_owned();

        if let Ok(_) = output.send(Message::ReceivedText { text }) {
            // Channel closed.
            break;
        };
    }

    let _ = output.send(Message::Exit);

    Ok(())
}

fn watch_clipboard_forever(output: mpsc::Sender<Message>) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new()?;
    let mut last_update_count = watch_clipboard::get_update_count();

    loop {
        std::thread::sleep(POLL_INTERVAL);

        let current_update_count = watch_clipboard::get_update_count();
        tracing::trace!(current_update_count);

        if current_update_count == last_update_count {
            tracing::trace!("No update");
            continue;
        }

        last_update_count = current_update_count;
        let Ok(text) = clipboard.get_text() else {
            // E.g. no text content.
            tracing::debug!("No text");
            continue;
        };

        let Ok(_) = output.send(Message::ClipboardTextChanged { text }) else {
            // Channel closed.
            break;
        };
    }

    Ok(())
}

fn handle_messages(input: mpsc::Receiver<Message>) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new()?;
    let mut last_text: Option<String> = None;

    while let Ok(message) = input.recv() {
        if last_text.as_ref().map(|s| s.as_str()) == message.text() {
            tracing::debug!("No change");
            continue;
        }

        match &message {
            Message::ClipboardTextChanged { text } => {
                print!("{}\0", text);
                std::io::stdout().flush()?;
            }
            Message::ReceivedText { text } => {
                tracing::debug!(?text, "Writing to clipboard");
                clipboard.set_text(text)?;
            }
            Message::Exit => break,
        }

        last_text = message.into_text();
    }

    Ok(())
}
