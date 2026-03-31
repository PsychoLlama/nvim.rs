use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // The generated enum header is produced by CMake before cargo runs.
    // Accept an env var override; fall back to the default relative path.
    let enum_header = env::var("NVIM_EX_CMDS_ENUM_HEADER").unwrap_or_else(|_| {
        // Relative to workspace root (two levels up from crate).
        "build/include/ex_cmds_enum.generated.h".to_string()
    });

    // Try to find the file relative to the manifest dir or workspace root.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // crate is at src/nvim-rs/ex_docmd, workspace root is ../../..
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("could not find workspace root");

    let header_path = workspace_root.join(&enum_header);

    // Tell cargo to rerun if the header changes.
    println!("cargo:rerun-if-changed={}", header_path.display());
    println!("cargo:rerun-if-env-changed=NVIM_EX_CMDS_ENUM_HEADER");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_file = out_dir.join("cmd_enum.rs");

    let content = match fs::read_to_string(&header_path) {
        Ok(c) => c,
        Err(e) => {
            // If the header doesn't exist yet (clean build before cmake runs),
            // emit an empty file so the crate still compiles.
            eprintln!(
                "cargo:warning=Could not read {}: {}. CMD_ constants will be empty.",
                header_path.display(),
                e
            );
            fs::write(&out_file, b"// CMD_ constants not yet generated\n").unwrap();
            return;
        }
    };

    let mut out = fs::File::create(&out_file).unwrap();

    // Parse lines of the form "  CMD_xxx," or "  CMD_xxx = N," from the C enum.
    // The enum starts after "typedef enum CMD_index {" and ends at "} cmdidx_T;".
    let mut in_enum = false;
    let mut index: i32 = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.contains("typedef enum CMD_index") || trimmed == "typedef enum CMD_index {" {
            in_enum = true;
            continue;
        }

        if !in_enum {
            continue;
        }

        // End of enum
        if trimmed.starts_with('}') {
            break;
        }

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }

        // Parse enum member: "CMD_xxx," or "CMD_xxx = N,"
        let entry = trimmed.trim_end_matches(',');
        if let Some(eq_pos) = entry.find('=') {
            let name = entry[..eq_pos].trim();
            let val_str = entry[eq_pos + 1..].trim();
            if let Ok(val) = val_str.parse::<i32>() {
                index = val;
                writeln!(out, "pub const {}: i32 = {};", name, index).unwrap();
                index += 1;
            }
        } else if entry.starts_with("CMD_") || entry.starts_with("  CMD_") {
            let name = entry.trim();
            writeln!(out, "pub const {}: i32 = {};", name, index).unwrap();
            index += 1;
        }
    }
}
