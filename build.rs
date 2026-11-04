fn main() {
    println!("cargo:rerun-if-changed=assets/brand/windows/podcast-monitor.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("assets/brand/windows/podcast-monitor.ico")
            .compile()
            .expect("embedding the Windows icon");
    }
}
