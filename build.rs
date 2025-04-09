use html_minifier::minify;
use std::{fs, path::Path};

fn main() {
    let dir = Path::new("static_src");
    let out_dir = Path::new("static");

    fs::create_dir_all(out_dir).unwrap();

    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("html") {
            let html = fs::read_to_string(&path).unwrap();
            let minified = minify(&html).unwrap();
            let output_path = out_dir.join(path.file_name().unwrap());
            fs::write(output_path, minified).unwrap();
        }
    }

    println!("cargo:rerun-if-changed=templates_src");
}
