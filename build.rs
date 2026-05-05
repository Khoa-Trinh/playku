use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn main() {
    // 1. Setup the library path
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("Failed to get CARGO_MANIFEST_DIR");
    let lib_path = PathBuf::from(&manifest_dir).join("mpv_lib");

    // 2. Tell Cargo where to find the library for linking
    println!("cargo:rustc-link-search=native={}", lib_path.display());
    println!("cargo:rustc-link-lib=mpv");
    println!("cargo:rerun-if-changed=mpv_lib");
    println!("cargo:rerun-if-changed=mpv_config");

    // --- AUTOMATION: COPY DLL AND CONFIG TO TARGET DIRECTORY ---
    
    // Cargo gives us an OUT_DIR deep inside the target folder (e.g., target/debug/build/playku-xxxx/out)
    if let Ok(out_dir) = env::var("OUT_DIR") {
        let out_path = PathBuf::from(out_dir);
        
        // Navigate up 3 levels to reach the actual `target/debug/` or `target/release/` folder
        if let (Some(_build_dir), Some(target_dir)) = (
            out_path.parent().and_then(|p| p.parent()), 
            out_path.parent().and_then(|p| p.parent().and_then(|p2| p2.parent()))
        ) {
            // 1. Copy DLLs
            if let Ok(entries) = fs::read_dir(&lib_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("dll") {
                        let file_name = path.file_name().unwrap();
                        let dest_path = target_dir.join(file_name);
                        let _ = fs::copy(&path, &dest_path);
                    }
                }
            }

            // 2. Copy mpv_config directory
            let src_config = PathBuf::from(&manifest_dir).join("mpv_config");
            let dst_config = target_dir.join("mpv_config");
            if src_config.exists() {
                let _ = copy_dir_all(&src_config, &dst_config);
            }
        }
    }
}