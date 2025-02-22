fn main() {
    #[cfg(windows)]
    let _ = embed_resource::compile("tray.rc", embed_resource::NONE);
}
