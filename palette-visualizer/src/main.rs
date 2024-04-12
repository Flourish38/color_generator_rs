use regex::Regex;
use std::env;
use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::node::Value;
use svg::Document;

const RADIUS: f64 = 50.0;

fn get_position(radius: f64, angle: f64) -> (f64, f64) {
    let (sin, cos) = angle.sin_cos();
    (sin * radius, cos * radius)
}

fn make_slice<T: ?Sized>((r1, r2): (f64, f64), (angle_1, angle_2): (f64, f64), color: &T) -> Path
where
    Value: for<'a> From<&'a T>,
{
    let inner_radius = r1.min(r2);
    let outer_radius = r1.max(r2);
    let start_angle = angle_1.min(angle_2);
    let end_angle = angle_1.max(angle_2);

    let c1 = get_position(inner_radius, start_angle);
    let c2 = get_position(inner_radius, end_angle);
    let c3 = get_position(outer_radius, end_angle);
    let c4 = get_position(outer_radius, start_angle);

    let data = Data::new()
        .move_to(c1)
        .elliptical_arc_to((inner_radius, inner_radius, 0, 0, 0, c2.0, c2.1))
        .line_to(c3)
        .elliptical_arc_to((outer_radius, outer_radius, 0, 0, 1, c4.0, c4.1))
        .close();

    Path::new().set("fill", color).set("d", data)
}

fn make_ring(radii: (f64, f64), start_angle: f64, colors: Vec<String>) -> Vec<Path> {
    let n = colors.len();
    if n == 1 {
        return make_ring(
            radii,
            start_angle,
            vec![colors[0].clone(), colors[0].clone()],
        );
    }
    let angle_offset = std::f64::consts::TAU / n as f64;
    colors
        .iter()
        .enumerate()
        .map(|(i, color)| {
            let angle_1 = start_angle + i as f64 * angle_offset;
            let angle_2 = start_angle + (i + 1) as f64 * angle_offset;
            make_slice(radii, (angle_1, angle_2), color)
        })
        .collect()
}

fn make_document(rings: Vec<Vec<Path>>) -> Document {
    rings
        .into_iter()
        .fold(Document::new(), |doc, paths| {
            paths.into_iter().fold(doc, |doc, path| doc.add(path))
        })
        .set("viewBox", (-RADIUS, -RADIUS, RADIUS * 2.0, RADIUS * 2.0))
}

fn calculate_radius_frac(inner_ring: usize, outer_ring: usize, inside_frac: f64) -> f64 {
    // derivation: https://www.wolframalpha.com/input?i2d=true&i=Integrate%5Br%2C%7Br%2Cf%2CSubscript%5Br%2C2%5D%7D%2C%7Bt%2C0%2CDivide%5B2%CF%80%2Cm%5D%7D%5D%3DIntegrate%5Br%2C%7Br%2C0%2C1%7D%2C%7Bt%2C0%2CDivide%5B2%CF%80%2Cn%5D%7D%5D
    // exploration: https://www.desmos.com/calculator/esepiyoguk
    let partial = inside_frac * inside_frac * inner_ring as f64;
    (partial + outer_ring as f64).sqrt() / partial.sqrt()
}

fn calculate_radii(per_ring: Vec<usize>) -> Vec<f64> {
    let mut acc = 1.0;
    let mut rad_fracs = Vec::new();
    for i in 1..per_ring.len() {
        let f = calculate_radius_frac(per_ring[0], per_ring[i], acc);
        rad_fracs.push(f);
        acc *= f;
    }
    let mut total = RADIUS;
    let mut outputs = vec![total];
    for frac in rad_fracs.iter().rev() {
        total /= frac;
        outputs.push(total);
    }
    outputs.reverse();
    outputs
}

fn make_rings(colors: Vec<String>) -> Vec<Vec<Path>> {
    let n = colors.len();
    todo!()
}

fn main() {
    // cargo run -p palette-visualizer -- #ff0000 #ffff00 #00ff00 #0000ff
    // cargo run -p palette-visualizer -- '#ff0000' '#ffff00' '#00ff00' '#0000ff'
    // let regex = Regex::new(r"^#[0-9a-fA-F]{6}$").unwrap();
    // let colors: Vec<_> = env::args().filter(|s| regex.is_match(s)).collect();

    // let document = make_document(make_rings(colors));

    // svg::save("image.svg", &document).unwrap();

    println!("{:?}", calculate_radii(vec![3, 9, 15]))
}
