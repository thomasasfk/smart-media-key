fn main() {
    #[cfg(windows)]
    let _ = embed_resource::compile("resources/tray.rc", embed_resource::NONE);
}
