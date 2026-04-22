#[derive(Debug, PartialEq)]
pub enum Primitive {
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        fill: Option<String>,
        stroke: Option<String>,
        stroke_width: Option<f64>,
    },
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        stroke: Option<String>,
        stroke_width: Option<f64>,
    },
    Polyline {
        points: Vec<(f64, f64)>,
        stroke: Option<String>,
        stroke_width: Option<f64>,
    },
    Path {
        d: String,
        fill: Option<String>,
        stroke: Option<String>,
        stroke_width: Option<f64>,
    },
    Circle {
        cx: f64,
        cy: f64,
        r: f64,
        fill: Option<String>,
        stroke: Option<String>,
        stroke_width: Option<f64>,
    },
    Ellipse {
        cx: f64,
        cy: f64,
        rx: f64,
        ry: f64,
        fill: Option<String>,
        stroke: Option<String>,
        stroke_width: Option<f64>,
    },
    Polygon {
        points: Vec<(f64, f64)>,
        fill: Option<String>,
        stroke: Option<String>,
        stroke_width: Option<f64>,
    },
    Text {
        x: f64,
        y: f64,
        content: String,
        font_family: Option<String>,
        font_size: Option<f64>,
        fill: Option<String>,
    },
    Group {
        children: Vec<Primitive>,
    },
    Unknown {
        tag: String,
        attrs: Vec<(String, String)>,
    },
}
