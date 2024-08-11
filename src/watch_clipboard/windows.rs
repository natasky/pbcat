pub fn get_update_count() -> u64 {
    unsafe { windows::Win32::System::DataExchange::GetClipboardSequenceNumber() as u64 }
}
