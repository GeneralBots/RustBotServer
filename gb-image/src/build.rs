use std::fs;
use std::path::Path;

fn main() {
    let assets_dir = Path::new("assets");
    if !assets_dir.exists() {
        fs::create_dir(assets_dir).expect("Failed to create assets directory");
    }

    let font_path = assets_dir.join("DejaVuSans.ttf");
    if !font_path.exists() {
        let font_url = "https://github.com/dejavu-fonts/dejavu-fonts/raw/master/ttf/DejaVuSans.ttf";
        let response = reqwest::blocking::get(font_url)
            .expect("Failed to download font")
            .bytes()
            .expect("Failed to get font bytes");
        
        fs::write(font_path, response)
            .expect("Failed to save font file");
    }
}