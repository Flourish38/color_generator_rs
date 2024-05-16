mod code;

pub use code::make_document;
use lib::color::sRGB;

const RADIUS: f64 = 300.0;
const DELTA: f64 = RADIUS * 0.02;

pub fn save_svg<T>(path: T, colors: Vec<sRGB>) -> Result<(), std::io::Error>
where
    T: std::convert::AsRef<std::path::Path>,
{
    let document = make_document(colors, RADIUS, DELTA);

    svg::save(path, &document)
}
