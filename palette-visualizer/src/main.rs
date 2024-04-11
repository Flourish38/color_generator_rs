use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::Document;

const CENTER: (f32, f32) = (50.0, 50.0);

fn get_position(radius: f32, angle: f32) -> (f32, f32) {
    let (sin, cos) = angle.sin_cos();
    (sin * radius + CENTER.0, cos * radius + CENTER.1)
}

fn make_slice<T: Into<svg::node::Value>>(
    (r1, r2): (f32, f32),
    (angle_1, angle_2): (f32, f32),
    color: T,
) -> Path {
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

fn main() {
    let slice = make_slice(
        (15.0, 25.0),
        (15.0_f32.to_radians(), 105.0_f32.to_radians()),
        "red",
    );
    let slice2 = make_slice(
        (15.0, 30.0),
        (105.0_f32.to_radians(), 195.0_f32.to_radians()),
        "yellow",
    );
    let slice3 = make_slice(
        (15.0, 35.0),
        (195.0_f32.to_radians(), 285.0_f32.to_radians()),
        "green",
    );
    let slice4 = make_slice(
        (15.0, 40.0),
        (285.0_f32.to_radians(), 375.0_f32.to_radians()),
        "blue",
    );

    let document = Document::new()
        .set("viewBox", (0, 0, 100, 100))
        .add(slice)
        .add(slice2)
        .add(slice3)
        .add(slice4);

    svg::save("image.svg", &document).unwrap();
}
