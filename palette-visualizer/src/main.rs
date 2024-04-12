use regex::Regex;
use std::env;
use std::f64::consts::{PI, TAU};
use std::f64::INFINITY;
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
    let angle_offset = TAU / n as f64;
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

fn calculate_radii(per_ring: &Vec<usize>) -> Vec<f64> {
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
    outputs.push(0.0);
    outputs.reverse();
    outputs
}

fn squareness_objective((inner_radius, outer_radius): (f64, f64), n: usize) -> f64 {
    // squareness is the square root of the area divided by the chord length.
    let squareness = (PI * (outer_radius * outer_radius - inner_radius * inner_radius) / n as f64)
        .sqrt()
        / (outer_radius - inner_radius);

    // pass this through a function that makes deviations from 1 punished
    // this function is chosen because f(1/x) = f(x) for x > 0
    squareness.ln().abs()
}

fn mean_squareness(list: &Vec<usize>) -> f64 {
    let radii = calculate_radii(&list);
    let mut total = 0.0;
    for i in 0..list.len() {
        total += list[i] as f64 * squareness_objective((radii[i], radii[i + 1]), list[i]);
    }
    total / (list.iter().sum::<usize>() as f64)
}

fn add_to_list(
    list: &Vec<usize>,
    n_remaining: usize,
    layers_remaining: usize,
) -> (Vec<usize>, f64) {
    if layers_remaining == 0 {
        let mut inner_list = list.clone();
        inner_list.push(n_remaining);
        let sq = mean_squareness(&inner_list);
        (inner_list, sq)
    } else {
        let biggest_size = *list.last().unwrap_or(&0);
        let mut best_list = vec![];
        let mut best_score = INFINITY;
        for i in biggest_size..((n_remaining + layers_remaining) / (layers_remaining + 1)) {
            let mut inner_list = list.clone();
            inner_list.push(i);
            let (new_list, score) = add_to_list(&inner_list, n_remaining - i, layers_remaining - 1);
            if score < best_score {
                best_list = new_list;
                best_score = score;
            }
        }
        (best_list, best_score)
    }
}

fn optimize_layers(n: usize) -> (Vec<usize>, f64) {
    let baseline_squareness = squareness_objective((0.0, RADIUS), n);
    let mut best_result = vec![n];
    let mut best_score = baseline_squareness;
    let mut layers = 1;
    loop {
        //println!("{}\t{}:\t{:?}", layers, best_score, best_result);
        let (list, score) = add_to_list(&vec![], n, layers);
        if score < best_score {
            best_result = list;
            best_score = score;
            layers += 1;
        } else {
            //println!("{}\t{}:\t{:?}", layers + 1, score, list);
            return (best_result, best_score);
        }
    }
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

    use std::time::Instant;
    let start_time = Instant::now();

    let mut prev_len = 0;
    let mut prev_score = INFINITY;
    let mut increasing = false;
    for i in 1..171 {
        let results = optimize_layers(i);
        let len = results.0.len();
        if len != prev_len {
            println!("{}:\t{}\t{}\t{:?}", i, len, results.1, results.0);
            prev_len = len;
        }
        if prev_score < results.1 && !increasing {
            println!(
                "{}:\t{}\t{}\t{:?}",
                i - 1,
                len,
                prev_score,
                optimize_layers(i - 1).0
            );
            increasing = true;
        }
        if prev_score > results.1 {
            increasing = false;
        }
        prev_score = results.1;
    }
}
