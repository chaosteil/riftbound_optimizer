use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("cards.json");
    
    // We fetch the JSON dataset during build time
    let url = "https://gist.githubusercontent.com/OwenMelbz/e04dadf641cc9b81cb882b4612343112/raw/fadb466cfd014c6a47da41fc291ef5c436d4cf4f/riftbound.json";
    
    if let Ok(response) = ureq::get(url).call() {
        if let Ok(text) = response.into_string() {
            fs::write(&dest_path, text).expect("Failed to write cards.json");
        } else {
            // Write empty array on failure so it still compiles
            fs::write(&dest_path, "[]").unwrap();
        }
    } else {
        fs::write(&dest_path, "[]").unwrap();
    }
    
    println!("cargo:rerun-if-changed=build.rs");
}