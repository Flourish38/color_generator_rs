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
    // cargo run -p palette-visualizer -- ff0000 ffff00 00ff00 0000ff
    // cargo run -p palette-visualizer -- 000000 ff0000 00ff00 0000ff ffff00 ff00ff 00ffff ffffff ff8800
    // cargo run -p palette-visualizer --release -- FA35EC 9449FF 3BDB78 BF18BE 5FCE8D F7C4FF 9A9800 71017C AFA488 3D6C01 84048F 579C80 0B01B4 F4B294 8520DF BF285C 9228FD FF3990 CE4189 E35D53 C716DC 8DAA34 524163 14A99C 5137FC 897A3B 7DAFE3 0153AC 2D605F 513D3E 6F089B 1666D3 F2E5DB 64895E 5C0C7A 8F44C1 364A13 9F6170 D1D5A6 0378E9 1175A8 6E5B3D 4B0296 31EBC2 742779 B5EE50 AB42A5 F2ECFD A339D5 3918F9 71012C 809F6E D2F09E 748C9E 4617AA 9907BF E4B7FF 320CC3 002FC5 8B91FF 874A90 711ED3 F7F50A A51578 66497A 0DACCA 0110FF A4C7E8 65ED79 A9B276 BFC101 2A65A3 42735D D65334 4298BA AA3F42 FB6BFD 502CCA 75F4BF 9BE04F 408689 8875A2 22348C A0CF53 00DEC9 87B8A1 5605C0 85183A 407785 D84DD5 D77DCD 408503 F77502 00B7FE 3104AA 2B5E04 FFD3FB 671EB9 0E58F0 50BFC3 33CDB8 163AFF 9F74E6 C3AF25 D417FE 5E0FF0 3920DD 1710E4 B851C1 E5A3AE 493165 87FC7C C1FEBD 5979FE 0005CD F584EA 9B184F 3041E2 F6C901 09FFDC 9FC05A 7D3A90 2938A4 C39E59 FADC00 8B0BA8 6B1450 E56CB4

    let colors: Vec<_> = env::args()
        .map(parse_to_sRGB)
        .filter(Option::is_some)
        .map(Option::unwrap)
        .collect();

    if colors.len() == 0 {
        panic!("Got no colors!")
    }

    let start_time = std::time::Instant::now();

    let document = make_document(colors, RADIUS, DELTA);

    println!("{:#?}", start_time.elapsed());

    svg::save("image.svg", &document).unwrap();
}
