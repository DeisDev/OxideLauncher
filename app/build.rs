fn main() {
    // Skip icon embedding for now
    tauri_build::try_build(tauri_build::Attributes::new()).expect("failed to run build script");
}
