pub fn get_update_count() -> u64 {
    unsafe { objc2_app_kit::NSPasteboard::generalPasteboard().changeCount() as u64 }
}
