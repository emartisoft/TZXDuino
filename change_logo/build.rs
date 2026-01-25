fn main() {
    println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icon.ico"); // ikon dosyanın yolu
    res.compile().unwrap();
}
