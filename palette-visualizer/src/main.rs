use regex::Regex;
use std::env;
use std::f64::consts::{PI, TAU};
use std::f64::INFINITY;
use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::node::Value;
use svg::Document;

const RADIUS: f64 = 300.0;
const DELTA: f64 = RADIUS * 0.02;

fn get_position(radius: f64, angle: f64, delta: f64) -> (f64, f64) {
    let (sin, cos) = angle.sin_cos();
    (sin * radius - cos * delta, cos * radius + sin * delta)
}

fn make_slice<T: ?Sized>(
    (r_inner, r_outer): (f64, f64),
    start_angle: f64,
    n: usize,
    delta: f64,
    color: &T,
) -> Path
where
    Value: for<'a> From<&'a T>,
{
    // offset radii for spacing
    let inner_radius = if r_inner == 0.0 {
        // This correction ensures that the innermost ring comes to sharp points on the inside.
        let tan = (PI / 2.0 - PI / n as f64).tan();
        r_inner + tan * delta
    } else {
        r_inner + delta
    };
    let outer_radius = r_outer - delta;
    let end_angle = start_angle + TAU / n as f64;

    // didn't prove rigorously that these offsets work, but like. they do
    let c1 = get_position(inner_radius, start_angle, -delta);
    let c2 = get_position(inner_radius, end_angle, delta);
    let c3 = get_position(outer_radius, end_angle, delta);
    let c4 = get_position(outer_radius, start_angle, -delta);

    // correction to make sure the arcs are still centered at the origin
    let outer_radius_offset = (outer_radius * outer_radius + delta * delta).sqrt();
    let inner_radius_offset = (inner_radius * inner_radius + delta * delta).sqrt();

    let data = Data::new()
        .move_to(c1)
        .elliptical_arc_to((
            inner_radius_offset,
            inner_radius_offset,
            0,
            0,
            0,
            c2.0,
            c2.1,
        ))
        .line_to(c3)
        .elliptical_arc_to((
            outer_radius_offset,
            outer_radius_offset,
            0,
            0,
            1,
            c4.0,
            c4.1,
        ))
        .close();

    Path::new().set("fill", color).set("d", data)
}

fn make_center_circle(radius: f64, delta: f64, color: String) -> Path {
    let inner_radius = 0.0;
    let outer_radius = radius - delta;

    let p1 = get_position(inner_radius, 0.0, 0.0);
    let p2 = get_position(inner_radius, PI, 0.0);
    let p3 = get_position(outer_radius, 0.0, 0.0);
    let p4 = get_position(outer_radius, PI, 0.0);

    let data = Data::new()
        .move_to(p1)
        .elliptical_arc_to((inner_radius, inner_radius, 0, 0, 0, p2.0, p2.1))
        .elliptical_arc_to((inner_radius, inner_radius, 0, 0, 0, p1.0, p1.1))
        .line_to(p3)
        .elliptical_arc_to((outer_radius, outer_radius, 0, 0, 1, p4.0, p4.1))
        .elliptical_arc_to((outer_radius, outer_radius, 0, 0, 1, p3.0, p3.1))
        .close();

    Path::new().set("fill", color).set("d", data)
}

fn make_ring(radii: (f64, f64), start_angle: f64, delta: f64, colors: Vec<String>) -> Vec<Path> {
    let n = colors.len();
    if n == 1 {
        if radii.0 == 0.0 {
            return vec![make_center_circle(radii.1, delta, colors[0].clone())];
        }
        return make_ring(
            radii,
            start_angle,
            delta,
            vec![colors[0].clone(), colors[0].clone()],
        );
    }
    let angle_offset = TAU / n as f64;
    colors
        .iter()
        .enumerate()
        .map(|(i, color)| {
            let inner_start_angle = start_angle + i as f64 * angle_offset;
            make_slice(radii, inner_start_angle, n, delta, color)
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
    // convert the unitless radii fractions into actual radii with units
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

fn optimize_layers(n: usize) -> Vec<usize> {
    let baseline_squareness = squareness_objective((0.0, RADIUS), n);
    let mut best_result = vec![n];
    let mut best_score = baseline_squareness;
    let mut layers = 1;
    loop {
        let (list, score) = add_to_list(&vec![], n, layers);
        if score < best_score {
            best_result = list;
            best_score = score;
            layers += 1;
        } else {
            return best_result;
        }
    }
}

fn angle_offset(n: usize, m: usize) -> f64 {
    // This is utter nonsense,
    // but it does a pretty good job of scrambling the lines.
    if n == m {
        // edge case. :/
        return PI / n as f64;
    }
    let (gcd, lcm) = num_integer::gcd_lcm(n, m);
    if n % 2 == 0 || m % 2 == 0 {
        (PI / lcm as f64) + (PI / gcd as f64) + (TAU / n.max(m) as f64)
    } else {
        (TAU / lcm as f64) + (PI / gcd as f64) + (TAU / n.min(m) as f64)
    }
}

fn calculate_angles(rings: &Vec<usize>) -> Vec<f64> {
    let outer_angle = angle_offset(*rings.last().unwrap(), 4);
    let mut output = vec![outer_angle];
    let mut prev_angle = outer_angle;
    for i in (1..=(rings.len() - 1)).rev() {
        prev_angle += angle_offset(rings[i], rings[i - 1]);
        output.push(prev_angle);
    }
    output.reverse();
    output
}

fn make_rings(mut colors: Vec<String>) -> Vec<Vec<Path>> {
    let n = colors.len();

    let ring_sizes = optimize_layers(n);

    let num_rings = ring_sizes.len();

    println!("{:?}", ring_sizes);

    let radii = calculate_radii(&ring_sizes);

    let angles = calculate_angles(&ring_sizes);

    let mut rings = Vec::with_capacity(num_rings);

    for i in 0..ring_sizes.len() {
        let remaining_colors = colors.split_off(ring_sizes[i]);
        rings.push(make_ring(
            (radii[i], radii[i + 1]),
            angles[i],
            // lines between patches get thinner with more rings
            DELTA / ((num_rings + 1) as f64).log2(),
            colors,
        ));
        colors = remaining_colors;
    }

    rings
}

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

    let document = make_document(make_rings(colors));

    svg::save("image.svg", &document).unwrap();
}
