fn main() {
    set_windows_exe_options();
}

fn set_windows_exe_options() {
    static MANIFEST: &str = "../../pkg/windows/manifest.xml";

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let profile = std::env::var("PROFILE").unwrap_or_default();

    if target_os == "windows" && profile == "release" {
        let mut res = winres::WindowsResource::new();
        res.set_manifest_file(MANIFEST);

        if let Err(e) = res.compile() {
            eprintln!("Error: Failed to compile Windows resources: {e}");
            std::process::exit(1);
        }
    }
}
