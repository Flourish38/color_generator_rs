mod code;

use code::make_document;
use regex::Regex;
use std::env;

const RADIUS: f64 = 300.0;
const DELTA: f64 = RADIUS * 0.02;

fn main() {
    // cargo run -p palette-visualizer -- #ff0000 #ffff00 #00ff00 #0000ff
    // cargo run -p palette-visualizer -- '#ff0000' '#ffff00' '#00ff00' '#0000ff'
    let regex = Regex::new(r"^#[0-9a-fA-F]{6}$").unwrap();
    let colors: Vec<_> = env::args()
        .map(|s| {
            if s.starts_with("#") {
                s
            } else {
                let mut out = "#".to_owned();
                out.push_str(s.as_str());
                out
            }
        })
        .filter(|s| regex.is_match(s))
        .collect();

    println!("{}", colors.len());

    let start_time = std::time::Instant::now();

    let document = make_document(colors, RADIUS, DELTA);

    println!("{:#?}", start_time.elapsed());

    svg::save("image.svg", &document).unwrap();
}
