use crate::svg::Primitive;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

#[derive(Debug, Clone, PartialEq)]
pub struct JitterConfig {
    pub amplitude: f64,
    pub frequency: f64,
    pub stroke_width_var: f64,
}

pub struct JitteredPath {
    pub d: String,
    pub stroke_width: Option<f64>,
}

impl Default for JitterConfig {
    fn default() -> Self {
        Self {
            amplitude: 2.0,
            frequency: 5.0,
            stroke_width_var: 0.2,
        }
    }
}

fn next_seed(seed_state: &mut Option<u64>) -> Option<u64> {
    let seed = *seed_state;
    if let Some(seed) = seed {
        *seed_state = Some(seed.wrapping_add(1));
    }
    seed
}

fn noise_with_rng<R: Rng + ?Sized>(rng: &mut R, amplitude: f64) -> f64 {
    (rng.gen::<f64>() - 0.5) * 2.0 * amplitude
}

fn jittered_stroke_width<R: Rng + ?Sized>(
    base: Option<f64>,
    config: &JitterConfig,
    rng: &mut R,
) -> Option<f64> {
    base.map(|w| {
        let v = noise_with_rng(rng, config.stroke_width_var * w);
        (w + v).max(0.1)
    })
}

fn format_path_element(
    d: &str,
    fill: &Option<String>,
    stroke: &Option<String>,
    stroke_width: &Option<f64>,
) -> String {
    let mut attrs = vec![format!(r#"d="{}""#, escape_attr(d))];
    if let Some(f) = fill {
        attrs.push(format!(r#"fill="{}""#, escape_attr(f)));
    }
    if let Some(s) = stroke {
        attrs.push(format!(r#"stroke="{}""#, escape_attr(s)));
    }
    if let Some(sw) = stroke_width {
        attrs.push(format!(r#"stroke-width="{sw:.3}""#));
    }
    format!("<path {} />", attrs.join(" "))
}

fn escape_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn jitter_primitive(primitive: &Primitive, config: &JitterConfig) -> String {
    jitter_primitive_with_seed(primitive, config, &mut None)
}

pub fn jitter_primitive_with_seed(
    primitive: &Primitive,
    config: &JitterConfig,
    seed_state: &mut Option<u64>,
) -> String {
    let seed = next_seed(seed_state);
    let mut rng = seed
        .map(StdRng::seed_from_u64)
        .unwrap_or_else(StdRng::from_entropy);
    jitter_primitive_with_rng(primitive, config, &mut rng)
}

pub fn jitter_primitive_path_with_seed(
    primitive: &Primitive,
    config: &JitterConfig,
    seed_state: &mut Option<u64>,
) -> Option<JitteredPath> {
    let seed = next_seed(seed_state);
    let mut rng = seed
        .map(StdRng::seed_from_u64)
        .unwrap_or_else(StdRng::from_entropy);
    jitter_primitive_path_with_rng(primitive, config, &mut rng)
}

fn jitter_primitive_path_with_rng<R: Rng + ?Sized>(
    primitive: &Primitive,
    config: &JitterConfig,
    rng: &mut R,
) -> Option<JitteredPath> {
    match primitive {
        Primitive::Rect {
            x,
            y,
            width,
            height,
            stroke_width,
            ..
        } => Some(JitteredPath {
            d: jitter_rect(*x, *y, *width, *height, config, rng),
            stroke_width: jittered_stroke_width(*stroke_width, config, rng),
        }),
        Primitive::Line {
            x1,
            y1,
            x2,
            y2,
            stroke_width,
            ..
        } => Some(JitteredPath {
            d: jitter_line(*x1, *y1, *x2, *y2, config, rng),
            stroke_width: jittered_stroke_width(*stroke_width, config, rng),
        }),
        Primitive::Polyline {
            points,
            stroke_width,
            ..
        } => {
            let d = jitter_polyline(points, config, rng);
            (!d.is_empty()).then_some(JitteredPath {
                d,
                stroke_width: jittered_stroke_width(*stroke_width, config, rng),
            })
        }
        Primitive::Path {
            d, stroke_width, ..
        } => {
            let d = jitter_path_d(d, config, rng)?;
            Some(JitteredPath {
                d,
                stroke_width: jittered_stroke_width(*stroke_width, config, rng),
            })
        }
        Primitive::Circle {
            cx,
            cy,
            r,
            stroke_width,
            ..
        } => {
            let path_d = circle_to_path(*cx, *cy, *r);
            let d = jitter_path_d(&path_d, config, rng)?;
            Some(JitteredPath {
                d,
                stroke_width: jittered_stroke_width(*stroke_width, config, rng),
            })
        }
        Primitive::Ellipse {
            cx,
            cy,
            rx,
            ry,
            stroke_width,
            ..
        } => {
            let path_d = ellipse_to_path(*cx, *cy, *rx, *ry);
            let d = jitter_path_d(&path_d, config, rng)?;
            Some(JitteredPath {
                d,
                stroke_width: jittered_stroke_width(*stroke_width, config, rng),
            })
        }
        Primitive::Polygon {
            points,
            stroke_width,
            ..
        } => {
            let path_d = polygon_to_path(points);
            let d = jitter_path_d(&path_d, config, rng)?;
            Some(JitteredPath {
                d,
                stroke_width: jittered_stroke_width(*stroke_width, config, rng),
            })
        }
        _ => None,
    }
}

fn jitter_primitive_with_rng<R: Rng + ?Sized>(
    primitive: &Primitive,
    config: &JitterConfig,
    rng: &mut R,
) -> String {
    match primitive {
        Primitive::Rect {
            x,
            y,
            width,
            height,
            fill,
            stroke,
            stroke_width,
        } => {
            let d = jitter_rect(*x, *y, *width, *height, config, rng);
            let sw = jittered_stroke_width(*stroke_width, config, rng);
            format_path_element(&d, fill, stroke, &sw)
        }
        Primitive::Line {
            x1,
            y1,
            x2,
            y2,
            stroke,
            stroke_width,
        } => {
            let d = jitter_line(*x1, *y1, *x2, *y2, config, rng);
            let sw = jittered_stroke_width(*stroke_width, config, rng);
            format_path_element(&d, &None, stroke, &sw)
        }
        Primitive::Polyline {
            points,
            stroke,
            stroke_width,
        } => {
            let d = jitter_polyline(points, config, rng);
            let sw = jittered_stroke_width(*stroke_width, config, rng);
            format_path_element(&d, &None, stroke, &sw)
        }
        Primitive::Path {
            d,
            fill,
            stroke,
            stroke_width,
        } => {
            let jd = jitter_path_d(d, config, rng).unwrap_or_else(|| d.to_string());
            let sw = jittered_stroke_width(*stroke_width, config, rng);
            format_path_element(&jd, fill, stroke, &sw)
        }
        _ => "<!-- unsupported -->".to_string(),
    }
}

fn jitter_rect<R: Rng + ?Sized>(
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    config: &JitterConfig,
    rng: &mut R,
) -> String {
    let segments = config.frequency.max(1.0).ceil() as usize;
    let mut pts = Vec::new();

    // top edge
    for i in 0..segments {
        let t = i as f64 / segments as f64;
        pts.push((
            x + w * t + noise_with_rng(rng, config.amplitude),
            y + noise_with_rng(rng, config.amplitude),
        ));
    }
    // right edge
    for i in 1..segments {
        let t = i as f64 / segments as f64;
        pts.push((
            x + w + noise_with_rng(rng, config.amplitude),
            y + h * t + noise_with_rng(rng, config.amplitude),
        ));
    }
    // bottom edge
    for i in 1..segments {
        let t = i as f64 / segments as f64;
        pts.push((
            x + w * (1.0 - t) + noise_with_rng(rng, config.amplitude),
            y + h + noise_with_rng(rng, config.amplitude),
        ));
    }
    // left edge
    for i in 1..=segments {
        let t = i as f64 / segments as f64;
        pts.push((
            x + noise_with_rng(rng, config.amplitude),
            y + h * (1.0 - t) + noise_with_rng(rng, config.amplitude),
        ));
    }

    if pts.is_empty() {
        return String::new();
    }
    let mut d = format!("M {:.3} {:.3}", pts[0].0, pts[0].1);
    for p in pts.iter().skip(1) {
        d.push_str(&format!(" L {:.3} {:.3}", p.0, p.1));
    }
    d.push_str(" Z");
    d
}

fn jitter_line<R: Rng + ?Sized>(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    config: &JitterConfig,
    rng: &mut R,
) -> String {
    let segments = config.frequency.max(1.0).ceil() as usize;
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();
    let (nx, ny) = if len > 0.0 {
        (-dy / len, dx / len)
    } else {
        (0.0, 0.0)
    };

    let mut pts = vec![(x1, y1)];
    for i in 1..segments {
        let t = i as f64 / segments as f64;
        let px = x1 + dx * t;
        let py = y1 + dy * t;
        let n = noise_with_rng(rng, config.amplitude);
        pts.push((px + nx * n, py + ny * n));
    }
    pts.push((x2, y2));

    let mut d = format!("M {:.3} {:.3}", pts[0].0, pts[0].1);
    for p in pts.iter().skip(1) {
        d.push_str(&format!(" L {:.3} {:.3}", p.0, p.1));
    }
    d
}

fn jitter_polyline<R: Rng + ?Sized>(
    points: &[(f64, f64)],
    config: &JitterConfig,
    rng: &mut R,
) -> String {
    if points.len() < 2 {
        return String::new();
    }
    let segments = config.frequency.max(1.0).ceil() as usize;
    let mut all_pts = Vec::new();
    all_pts.push(points[0]);

    for window in points.windows(2) {
        let (x1, y1) = window[0];
        let (x2, y2) = window[1];
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        let (nx, ny) = if len > 0.0 {
            (-dy / len, dx / len)
        } else {
            (0.0, 0.0)
        };

        for i in 1..segments {
            let t = i as f64 / segments as f64;
            let px = x1 + dx * t;
            let py = y1 + dy * t;
            let n = noise_with_rng(rng, config.amplitude);
            all_pts.push((px + nx * n, py + ny * n));
        }
        all_pts.push((x2, y2));
    }

    // 連続する重複点を除去
    let mut deduped = Vec::new();
    for &p in &all_pts {
        if deduped.is_empty() || deduped.last().unwrap() != &p {
            deduped.push(p);
        }
    }

    let mut d = format!("M {:.3} {:.3}", deduped[0].0, deduped[0].1);
    for p in deduped.iter().skip(1) {
        d.push_str(&format!(" L {:.3} {:.3}", p.0, p.1));
    }
    d
}

fn tokenize_d(d: &str) -> Vec<String> {
    let chars: Vec<char> = d.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() || c == ',' {
            i += 1;
            continue;
        }

        if is_command_char(c) {
            tokens.push(c.to_string());
            i += 1;
            continue;
        }

        let start = i;
        if matches!(chars[i], '+' | '-') {
            i += 1;
        }

        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }

        if i < chars.len() && chars[i] == '.' {
            i += 1;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
        }

        if i < chars.len() && matches!(chars[i], 'e' | 'E') {
            let exponent_start = i;
            i += 1;
            if i < chars.len() && matches!(chars[i], '+' | '-') {
                i += 1;
            }
            let digits_start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            if digits_start == i {
                i = exponent_start;
            }
        }

        if start == i {
            i += 1;
        } else {
            tokens.push(chars[start..i].iter().collect());
        }
    }

    tokens
}

fn is_command_char(c: char) -> bool {
    matches!(
        c,
        'M' | 'm'
            | 'L'
            | 'l'
            | 'C'
            | 'c'
            | 'Q'
            | 'q'
            | 'Z'
            | 'z'
            | 'H'
            | 'h'
            | 'V'
            | 'v'
            | 'S'
            | 's'
            | 'T'
            | 't'
            | 'A'
            | 'a'
    )
}

fn is_command(token: &str) -> bool {
    if token.len() != 1 {
        return false;
    }
    let c = token.chars().next().unwrap();
    matches!(
        c,
        'M' | 'm'
            | 'L'
            | 'l'
            | 'C'
            | 'c'
            | 'Q'
            | 'q'
            | 'Z'
            | 'z'
            | 'H'
            | 'h'
            | 'V'
            | 'v'
            | 'S'
            | 's'
            | 'T'
            | 't'
            | 'A'
            | 'a'
    )
}

/// 相対コマンドに対して現在の絶対座標を加算し、絶対座標値を返す
fn to_absolute(value: f64, current: f64, is_relative: bool) -> f64 {
    if is_relative {
        current + value
    } else {
        value
    }
}

fn jitter_path_d<R: Rng + ?Sized>(d: &str, config: &JitterConfig, rng: &mut R) -> Option<String> {
    let tokens = tokenize_d(d);
    let mut result = String::new();
    let mut i = 0;
    let mut current_x = 0.0;
    let mut current_y = 0.0;
    let mut start_x = 0.0;
    let mut start_y = 0.0;

    while i < tokens.len() {
        if is_command(&tokens[i]) {
            let cmd = tokens[i].chars().next().unwrap();
            let is_relative = cmd.is_ascii_lowercase();
            if !result.is_empty() {
                result.push(' ');
            }
            result.push(cmd.to_ascii_uppercase());
            i += 1;
            match cmd.to_ascii_uppercase() {
                'M' => {
                    let (x, y) = read_xy(&tokens, i, current_x, current_y, is_relative)?;
                    let out_x = x + noise_with_rng(rng, config.amplitude);
                    let out_y = y + noise_with_rng(rng, config.amplitude);
                    result.push_str(&format!(" {out_x:.3} {out_y:.3}"));
                    current_x = x;
                    current_y = y;
                    start_x = x;
                    start_y = y;
                    i += 2;
                    // 後続の座標ペアは暗黙のlinetoとして処理
                    while i < tokens.len() && !is_command(&tokens[i]) {
                        let (x, y) = read_xy(&tokens, i, current_x, current_y, is_relative)?;
                        let out_x = x + noise_with_rng(rng, config.amplitude);
                        let out_y = y + noise_with_rng(rng, config.amplitude);
                        result.push_str(&format!(" {out_x:.3} {out_y:.3}"));
                        current_x = x;
                        current_y = y;
                        i += 2;
                    }
                }
                'L' | 'T' => {
                    while i < tokens.len() && !is_command(&tokens[i]) {
                        let (x, y) = read_xy(&tokens, i, current_x, current_y, is_relative)?;
                        let out_x = x + noise_with_rng(rng, config.amplitude);
                        let out_y = y + noise_with_rng(rng, config.amplitude);
                        result.push_str(&format!(" {out_x:.3} {out_y:.3}"));
                        current_x = x;
                        current_y = y;
                        i += 2;
                    }
                }
                'C' => {
                    while i < tokens.len() && !is_command(&tokens[i]) {
                        let (x1, y1) = read_xy(&tokens, i, current_x, current_y, is_relative)?;
                        let (x2, y2) = read_xy(&tokens, i + 2, current_x, current_y, is_relative)?;
                        let (x, y) = read_xy(&tokens, i + 4, current_x, current_y, is_relative)?;
                        let out_x1 = x1 + noise_with_rng(rng, config.amplitude);
                        let out_y1 = y1 + noise_with_rng(rng, config.amplitude);
                        let out_x2 = x2 + noise_with_rng(rng, config.amplitude);
                        let out_y2 = y2 + noise_with_rng(rng, config.amplitude);
                        let out_x = x + noise_with_rng(rng, config.amplitude);
                        let out_y = y + noise_with_rng(rng, config.amplitude);
                        result.push_str(&format!(
                            " {out_x1:.3} {out_y1:.3} {out_x2:.3} {out_y2:.3} {out_x:.3} {out_y:.3}"
                        ));
                        current_x = x;
                        current_y = y;
                        i += 6;
                    }
                }
                'Q' | 'S' => {
                    while i < tokens.len() && !is_command(&tokens[i]) {
                        let (x1, y1) = read_xy(&tokens, i, current_x, current_y, is_relative)?;
                        let (x, y) = read_xy(&tokens, i + 2, current_x, current_y, is_relative)?;
                        let out_x1 = x1 + noise_with_rng(rng, config.amplitude);
                        let out_y1 = y1 + noise_with_rng(rng, config.amplitude);
                        let out_x = x + noise_with_rng(rng, config.amplitude);
                        let out_y = y + noise_with_rng(rng, config.amplitude);
                        result.push_str(&format!(" {out_x1:.3} {out_y1:.3} {out_x:.3} {out_y:.3}"));
                        current_x = x;
                        current_y = y;
                        i += 4;
                    }
                }
                'H' => {
                    while i < tokens.len() && !is_command(&tokens[i]) {
                        let x = read_number(&tokens, i)
                            .map(|v| to_absolute(v, current_x, is_relative))?;
                        let out_x = x + noise_with_rng(rng, config.amplitude);
                        result.push_str(&format!(" {out_x:.3}"));
                        current_x = x;
                        i += 1;
                    }
                }
                'V' => {
                    while i < tokens.len() && !is_command(&tokens[i]) {
                        let y = read_number(&tokens, i)
                            .map(|v| to_absolute(v, current_y, is_relative))?;
                        let out_y = y + noise_with_rng(rng, config.amplitude);
                        result.push_str(&format!(" {out_y:.3}"));
                        current_y = y;
                        i += 1;
                    }
                }
                'A' => {
                    while i < tokens.len() && !is_command(&tokens[i]) {
                        let rx = read_number(&tokens, i)?;
                        let ry = read_number(&tokens, i + 1)?;
                        let rot = read_number(&tokens, i + 2)?;
                        let large_arc = read_number(&tokens, i + 3)?;
                        let sweep = read_number(&tokens, i + 4)?;
                        let x = to_absolute(read_number(&tokens, i + 5)?, current_x, is_relative);
                        let y = to_absolute(read_number(&tokens, i + 6)?, current_y, is_relative);
                        let out_x = x + noise_with_rng(rng, config.amplitude);
                        let out_y = y + noise_with_rng(rng, config.amplitude);
                        result.push_str(&format!(
                            " {rx:.3} {ry:.3} {rot:.3} {large_arc:.0} {sweep:.0} {out_x:.3} {out_y:.3}"
                        ));
                        current_x = x;
                        current_y = y;
                        i += 7;
                    }
                }
                'Z' => {
                    current_x = start_x;
                    current_y = start_y;
                }
                _ => {}
            }
        } else {
            // コマンドなしに数字が来た場合はスキップ（正規化で起きないはず）
            return None;
        }
    }
    (!result.is_empty()).then_some(result)
}

fn read_xy(
    tokens: &[String],
    index: usize,
    current_x: f64,
    current_y: f64,
    is_relative: bool,
) -> Option<(f64, f64)> {
    if is_command(tokens.get(index)?) || is_command(tokens.get(index + 1)?) {
        return None;
    }
    Some((
        to_absolute(read_number(tokens, index)?, current_x, is_relative),
        to_absolute(read_number(tokens, index + 1)?, current_y, is_relative),
    ))
}

fn read_number(tokens: &[String], index: usize) -> Option<f64> {
    let token = tokens.get(index)?;
    if is_command(token) {
        return None;
    }
    token.parse::<f64>().ok()
}

/// Convert circle to path with 4 Bezier curves
/// k ≈ 0.55228 (4/3 * tan(π/8))
fn circle_to_path(cx: f64, cy: f64, r: f64) -> String {
    const K: f64 = 0.55228475;
    let kr = K * r;
    format!(
        "M {:.3} {:.3} C {:.3} {:.3} {:.3} {:.3} {:.3} {:.3} C {:.3} {:.3} {:.3} {:.3} {:.3} {:.3} C {:.3} {:.3} {:.3} {:.3} {:.3} {:.3} C {:.3} {:.3} {:.3} {:.3} {:.3} {:.3} Z",
        // Start at left point
        cx - r, cy,
        // Top-left curve
        cx - r, cy - kr, cx - kr, cy - r, cx, cy - r,
        // Top-right curve
        cx + kr, cy - r, cx + r, cy - kr, cx + r, cy,
        // Bottom-right curve
        cx + r, cy + kr, cx + kr, cy + r, cx, cy + r,
        // Bottom-left curve
        cx - kr, cy + r, cx - r, cy + kr, cx - r, cy,
    )
}

/// Convert ellipse to path with 4 Bezier curves
fn ellipse_to_path(cx: f64, cy: f64, rx: f64, ry: f64) -> String {
    const K: f64 = 0.55228475;
    let krx = K * rx;
    let kry = K * ry;
    format!(
        "M {:.3} {:.3} C {:.3} {:.3} {:.3} {:.3} {:.3} {:.3} C {:.3} {:.3} {:.3} {:.3} {:.3} {:.3} C {:.3} {:.3} {:.3} {:.3} {:.3} {:.3} C {:.3} {:.3} {:.3} {:.3} {:.3} {:.3} Z",
        // Start at left point
        cx - rx, cy,
        // Top-left curve
        cx - rx, cy - kry, cx - krx, cy - ry, cx, cy - ry,
        // Top-right curve
        cx + krx, cy - ry, cx + rx, cy - kry, cx + rx, cy,
        // Bottom-right curve
        cx + rx, cy + kry, cx + krx, cy + ry, cx, cy + ry,
        // Bottom-left curve
        cx - krx, cy + ry, cx - rx, cy + kry, cx - rx, cy,
    )
}

/// Convert polygon to path
fn polygon_to_path(points: &[(f64, f64)]) -> String {
    if points.is_empty() {
        return String::new();
    }
    let mut d = format!("M {:.3} {:.3}", points[0].0, points[0].1);
    for &(x, y) in &points[1..] {
        d.push_str(&format!(" L {:.3} {:.3}", x, y));
    }
    d.push_str(" Z");
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jitter_path_d_relative_commands() {
        let d = "m 10 10 l 20 0 c 10 0 10 10 0 10 z";
        let config = JitterConfig::default();
        let mut rng = StdRng::seed_from_u64(42);
        let result = jitter_path_d(d, &config, &mut rng).unwrap();
        // 相対コマンドが正しく処理され、結果にノイズが含まれることを確認
        assert!(result.starts_with('M'));
        assert!(result.contains('L'));
        assert!(result.contains('C'));
        assert!(result.contains('Z'));
    }

    #[test]
    fn test_jitter_path_d_does_not_panic_on_incomplete_path() {
        let config = JitterConfig::default();
        let mut rng = StdRng::seed_from_u64(42);
        assert_eq!(jitter_path_d("M0", &config, &mut rng), None);
        assert_eq!(jitter_path_d("M 0 0 L 1", &config, &mut rng), None);
    }

    #[test]
    fn test_tokenize_d_with_exponents() {
        let d = "M 1e-5 1e5 L 2.5e-2 3.0e+1";
        let tokens = tokenize_d(d);
        // 指数表記が正しくトークン化されることを確認
        assert!(tokens.contains(&"1e-5".to_string()));
        assert!(tokens.contains(&"1e5".to_string()));
        assert!(tokens.contains(&"2.5e-2".to_string()));
        assert!(tokens.contains(&"3.0e+1".to_string()));
    }

    #[test]
    fn test_tokenize_d_with_compact_numbers() {
        let tokens = tokenize_d("M0-1L2.5.5Z");
        assert_eq!(tokens, ["M", "0", "-1", "L", "2.5", ".5", "Z"]);
    }

    #[test]
    fn test_jitter_primitive_escapes_attributes() {
        let primitive = Primitive::Path {
            d: "M0 0L1 1".to_string(),
            fill: Some("url(#a&b)".to_string()),
            stroke: Some("red\"blue".to_string()),
            stroke_width: None,
        };
        let result = jitter_primitive_with_seed(&primitive, &JitterConfig::default(), &mut Some(1));
        assert!(result.contains(r#"fill="url(#a&amp;b)""#));
        assert!(result.contains(r#"stroke="red&quot;blue""#));
    }

    #[test]
    fn test_is_command_excludes_e() {
        assert!(!is_command("e"));
        assert!(!is_command("E"));
        assert!(is_command("M"));
        assert!(is_command("m"));
    }
}
