mod code;
mod color_sorting;

use code::make_document;
use color_lib::sRGB;
use regex::Regex;
use std::env;

const RADIUS: f64 = 300.0;
const DELTA: f64 = RADIUS * 0.02;

#[allow(non_snake_case)]
fn parse_to_sRGB(c: String) -> Option<sRGB> {
    let regex = Regex::new(r"^[0-9a-fA-F]{6}$").unwrap();
    let s = if c.starts_with('#') { &c[1..] } else { &c };
    if !regex.is_match(s) {
        None
    } else {
        Some([
            u8::from_str_radix(&s[0..2], 16).unwrap(),
            u8::from_str_radix(&s[2..4], 16).unwrap(),
            u8::from_str_radix(&s[4..6], 16).unwrap(),
        ])
    }
}

fn main() {
    // cargo run -p palette-visualizer -- #ff0000 #ffff00 #00ff00 #0000ff
    // cargo run -p palette-visualizer -- '#ff0000' '#ffff00' '#00ff00' '#0000ff'

    let colors: Vec<_> = env::args()
        .map(parse_to_sRGB)
        .filter(Option::is_some)
        .map(Option::unwrap)
        .collect();

    println!("{}", colors.len());

    let start_time = std::time::Instant::now();

    let document = make_document(colors, RADIUS, DELTA);

    println!("{:#?}", start_time.elapsed());

    svg::save("image.svg", &document).unwrap();
}
