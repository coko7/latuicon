use std::{env, fs, path::PathBuf};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let dst = manifest.join("src/icon_picker/glyphnames.json");

    if !dst.exists() {
        // Try to copy from the sibling late-sh repo on first build.
        let src = manifest.join("../late-sh/late-ssh/src/app/icon_picker/glyphnames.json");
        if src.exists() {
            fs::copy(&src, &dst).expect("failed to copy glyphnames.json from late-sh");
            println!("cargo:warning=Copied glyphnames.json from late-sh repo.");
        } else {
            panic!(
                "glyphnames.json not found at {} — copy it manually to src/icon_picker/",
                src.display()
            );
        }
    }

    // Re-run only if the JSON changes.
    println!("cargo:rerun-if-changed=src/icon_picker/glyphnames.json");
}
