use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::node::Value;
use svg::Document;

const SIZE: f64 = 100.0;

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

fn make_ring<T: ?Sized>(radii: (f64, f64), start_angle: f64, colors: Vec<&T>) -> Vec<Path>
where
    Value: for<'a> From<&'a T>,
{
    let n = colors.len();
    if n == 1 {
        return make_ring(radii, start_angle, vec![colors[0], colors[0]]);
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
        .set("viewBox", (-SIZE / 2.0, -SIZE / 2.0, SIZE, SIZE))
}

fn main() {
    let document = make_document(vec![
        make_ring((15.0, 25.0), 25.0_f64.to_radians(), vec!["green", "red"]),
        make_ring((0.0, 15.0), -15.0_f64.to_radians(), vec!["blue", "yellow"]),
    ]);

    svg::save("image.svg", &document).unwrap();
}