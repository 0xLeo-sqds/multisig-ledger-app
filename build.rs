fn main() {
    println!("cargo:rerun-if-changed=icons/");
    // TODO: Generate glyphs/home_nano_nbgl.png from icons when Squads branding is ready.
    // For now, we use placeholder icons from the boilerplate.
    // See app-boilerplate-rust/build.rs for the image processing pattern.
}
