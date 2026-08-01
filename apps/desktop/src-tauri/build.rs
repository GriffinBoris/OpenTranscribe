fn main() {
    #[cfg(target_os = "macos")]
    add_swift_link_search();

    tauri_build::build()
}

#[cfg(target_os = "macos")]
fn add_swift_link_search() {
    let output = std::process::Command::new("xcrun")
        .args(["--find", "swiftc"])
        .output()
        .expect("xcrun must locate swiftc for ScreenCaptureKit");
    assert!(
        output.status.success(),
        "xcrun must locate swiftc for ScreenCaptureKit"
    );
    let swiftc = std::path::Path::new(
        std::str::from_utf8(&output.stdout)
            .expect("swiftc path must be UTF-8")
            .trim(),
    );
    let swift_libraries = swiftc
        .parent()
        .and_then(std::path::Path::parent)
        .expect("swiftc must live under a toolchain usr/bin directory")
        .join("lib/swift/macosx");

    println!(
        "cargo:rustc-link-search=native={}",
        swift_libraries.display()
    );
}
