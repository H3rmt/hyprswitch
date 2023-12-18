use random_color::RandomColor;
use svg::node::element::{Group, Rectangle, SVG, Text};

pub fn create_svg(rectangles: Vec<(usize, u16, u16, u16, u16, String)>, filename: String, x: u16, y: u16, width: u16, height: u16, stroke_width: u16) {
    let mut svg = SVG::new()
        .set("width", "100%")
        .set("height", "100%")
        .set("viewBox", format!("{} {} {} {}", x, y, width, height));

    // draw line around the edge of the svg
    // svg = svg.add(
    //     Rectangle::new()
    //         .set("x", 0)
    //         .set("y", 0)
    //         .set("width", width)
    //         .set("height", height)
    //         .set("stroke", "red")
    //         .set("stroke-width", stroke_width)
    //         .set("fill", "none"),
    // );

    for (i, x, y, width, height, identifier) in rectangles {
        let color = RandomColor::new().to_hsl_array();
        let group = Group::new()
            .add(
                Rectangle::new()
                    .set("x", x)
                    .set("y", y)
                    .set("width", width)
                    .set("height", height)
                    .set("stroke", format!("hsl({},{}%,{}%)", color[0], color[1], color[2]))
                    .set("stroke-width", stroke_width)
                    .set("fill", "none"),
            )
            .add(
                Text::new()
                    .set("x", (x + width / 2) as i16 - ((identifier.len() as u16 * (stroke_width * 4)) / 2) as i16)
                    .set("y", (y + height / 2) as i16 + (((((stroke_width) as f32 * color[0] as f32) / 90.0) as i16) - stroke_width as i16))
                    .set("font-size", stroke_width * 4)
                    .set("fill", "white")
                    .add(svg::node::Text::new(format!("{i}-{identifier}"))),
            );

        svg = svg.add(group);
    }

    svg::save(filename.clone(), &svg).unwrap_or_else(|_| panic!("unable to save svg {filename}"));
}