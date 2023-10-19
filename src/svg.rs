use random_color::RandomColor;
use svg::node::element::{Group, Rectangle, Text, SVG};

use crate::SortableClient;

pub fn create_svg(rectangles: Vec<(i16, i16, i16, i16, String)>, filename: &str) {
    let mut svg = SVG::new()
        .set("width", "100%")
        .set("height", "100%")
        .set("viewBox", "0 0 100 100");

    for (x, y, width, height, iden) in rectangles {
        let color = RandomColor::new().to_hsl_string();

        let group = Group::new()
            .add(
                Rectangle::new()
                    .set("x", x)
                    .set("y", y)
                    .set("width", width)
                    .set("height", height)
                    .set("stroke", color)
                    .set("stroke-width", "1"),
            )
            .add(
                Text::new()
                    .set("x", x + width / 3)
                    .set("y", y + height / 2 + 1)
                    .set("font-size", "3")
                    .set("fill", "white")
                    .add(svg::node::Text::new(iden)),
            );

        svg = svg.add(group);
    }

    // save to file
    svg::save(filename, &svg).unwrap();
}

pub fn create_svg_from_client<SC>(rectangles: &[SC], filename: &str)
where
    SC: SortableClient,
{
    create_svg(
        rectangles
            .iter()
            .map(|v| (v.x() * 2, v.y() * 2, v.w() * 2, v.h() * 2, v.iden()))
            .collect::<Vec<(i16, i16, i16, i16, String)>>(),
        filename,
    );
}
