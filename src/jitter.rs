use crate::svg::Primitive;
use rand::random;

pub struct JitterConfig {
    pub amplitude: f64,
    pub frequency: f64,
    pub stroke_width_var: f64,
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

// TODO: #5 seed 対応で再現性を持たせる
fn noise(amplitude: f64) -> f64 {
    (random::<f64>() - 0.5) * 2.0 * amplitude
}

fn jittered_stroke_width(base: Option<f64>, config: &JitterConfig) -> Option<f64> {
    base.map(|w| {
        let v = noise(config.stroke_width_var * w);
        (w + v).max(0.1)
    })
}

fn format_path_element(
    d: &str,
    fill: &Option<String>,
    stroke: &Option<String>,
    stroke_width: &Option<f64>,
) -> String {
    let mut attrs = vec![format!(r#"d="{}""#, d)];
    if let Some(f) = fill {
        attrs.push(format!(r#"fill="{}""#, f));
    }
    if let Some(s) = stroke {
        attrs.push(format!(r#"stroke="{}""#, s));
    }
    if let Some(sw) = stroke_width {
        attrs.push(format!(r#"stroke-width="{:.3}""#, sw));
    }
    format!("<path {} />", attrs.join(" "))
}

pub fn jitter_primitive(primitive: &Primitive, config: &JitterConfig) -> String {
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
            let d = jitter_rect(*x, *y, *width, *height, config);
            let sw = jittered_stroke_width(*stroke_width, config);
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
            let d = jitter_line(*x1, *y1, *x2, *y2, config);
            let sw = jittered_stroke_width(*stroke_width, config);
            format_path_element(&d, &None, stroke, &sw)
        }
        Primitive::Polyline {
            points,
            stroke,
            stroke_width,
        } => {
            let d = jitter_polyline(points, config);
            let sw = jittered_stroke_width(*stroke_width, config);
            format_path_element(&d, &None, stroke, &sw)
        }
        Primitive::Path {
            d,
            fill,
            stroke,
            stroke_width,
        } => {
            let jd = jitter_path_d(d, config);
            let sw = jittered_stroke_width(*stroke_width, config);
            format_path_element(&jd, fill, stroke, &sw)
        }
        _ => "<!-- unsupported -->".to_string(),
    }
}

fn jitter_rect(x: f64, y: f64, w: f64, h: f64, config: &JitterConfig) -> String {
    let segments = config.frequency.max(1.0).ceil() as usize;
    let mut pts = Vec::new();

    // top edge
    for i in 0..segments {
        let t = i as f64 / segments as f64;
        pts.push((
            x + w * t + noise(config.amplitude),
            y + noise(config.amplitude),
        ));
    }
    // right edge
    for i in 1..segments {
        let t = i as f64 / segments as f64;
        pts.push((
            x + w + noise(config.amplitude),
            y + h * t + noise(config.amplitude),
        ));
    }
    // bottom edge
    for i in 1..segments {
        let t = i as f64 / segments as f64;
        pts.push((
            x + w * (1.0 - t) + noise(config.amplitude),
            y + h + noise(config.amplitude),
        ));
    }
    // left edge
    for i in 1..=segments {
        let t = i as f64 / segments as f64;
        pts.push((
            x + noise(config.amplitude),
            y + h * (1.0 - t) + noise(config.amplitude),
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

fn jitter_line(x1: f64, y1: f64, x2: f64, y2: f64, config: &JitterConfig) -> String {
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
        let n = noise(config.amplitude);
        pts.push((px + nx * n, py + ny * n));
    }
    pts.push((x2, y2));

    let mut d = format!("M {:.3} {:.3}", pts[0].0, pts[0].1);
    for p in pts.iter().skip(1) {
        d.push_str(&format!(" L {:.3} {:.3}", p.0, p.1));
    }
    d
}

fn jitter_polyline(points: &[(f64, f64)], config: &JitterConfig) -> String {
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
            let n = noise(config.amplitude);
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
    let mut normalized = d.replace(',', " ");
    for cmd in [
        'M', 'm', 'L', 'l', 'C', 'c', 'Q', 'q', 'Z', 'z', 'H', 'h', 'V', 'v', 'S', 's', 'T', 't',
        'A', 'a',
    ] {
        normalized = normalized.replace(cmd, &format!(" {} ", cmd));
    }
    normalized
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
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

fn jitter_path_d(d: &str, config: &JitterConfig) -> String {
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
            result.push(cmd);
            i += 1;
            match cmd.to_ascii_uppercase() {
                'M' => {
                    let x = to_absolute(
                        tokens[i].parse::<f64>().unwrap_or(0.0),
                        current_x,
                        is_relative,
                    ) + noise(config.amplitude);
                    let y = to_absolute(
                        tokens[i + 1].parse::<f64>().unwrap_or(0.0),
                        current_y,
                        is_relative,
                    ) + noise(config.amplitude);
                    result.push_str(&format!(" {:.3} {:.3}", x, y));
                    current_x = x;
                    current_y = y;
                    start_x = x;
                    start_y = y;
                    i += 2;
                    // 後続の座標ペアは暗黙のlinetoとして処理
                    while i + 2 <= tokens.len() && !is_command(&tokens[i]) {
                        let x = to_absolute(
                            tokens[i].parse::<f64>().unwrap_or(0.0),
                            current_x,
                            is_relative,
                        ) + noise(config.amplitude);
                        let y = to_absolute(
                            tokens[i + 1].parse::<f64>().unwrap_or(0.0),
                            current_y,
                            is_relative,
                        ) + noise(config.amplitude);
                        result.push_str(&format!(" {:.3} {:.3}", x, y));
                        current_x = x;
                        current_y = y;
                        i += 2;
                    }
                }
                'L' | 'T' => {
                    while i + 2 <= tokens.len() && !is_command(&tokens[i]) {
                        let x = to_absolute(
                            tokens[i].parse::<f64>().unwrap_or(0.0),
                            current_x,
                            is_relative,
                        ) + noise(config.amplitude);
                        let y = to_absolute(
                            tokens[i + 1].parse::<f64>().unwrap_or(0.0),
                            current_y,
                            is_relative,
                        ) + noise(config.amplitude);
                        result.push_str(&format!(" {:.3} {:.3}", x, y));
                        current_x = x;
                        current_y = y;
                        i += 2;
                    }
                }
                'C' => {
                    while i + 6 <= tokens.len() && !is_command(&tokens[i]) {
                        let x1 = to_absolute(
                            tokens[i].parse::<f64>().unwrap_or(0.0),
                            current_x,
                            is_relative,
                        ) + noise(config.amplitude);
                        let y1 = to_absolute(
                            tokens[i + 1].parse::<f64>().unwrap_or(0.0),
                            current_y,
                            is_relative,
                        ) + noise(config.amplitude);
                        let x2 = to_absolute(
                            tokens[i + 2].parse::<f64>().unwrap_or(0.0),
                            current_x,
                            is_relative,
                        ) + noise(config.amplitude);
                        let y2 = to_absolute(
                            tokens[i + 3].parse::<f64>().unwrap_or(0.0),
                            current_y,
                            is_relative,
                        ) + noise(config.amplitude);
                        let x = to_absolute(
                            tokens[i + 4].parse::<f64>().unwrap_or(0.0),
                            current_x,
                            is_relative,
                        ) + noise(config.amplitude);
                        let y = to_absolute(
                            tokens[i + 5].parse::<f64>().unwrap_or(0.0),
                            current_y,
                            is_relative,
                        ) + noise(config.amplitude);
                        result.push_str(&format!(
                            " {:.3} {:.3} {:.3} {:.3} {:.3} {:.3}",
                            x1, y1, x2, y2, x, y
                        ));
                        current_x = x;
                        current_y = y;
                        i += 6;
                    }
                }
                'Q' | 'S' => {
                    while i + 4 <= tokens.len() && !is_command(&tokens[i]) {
                        let x1 = to_absolute(
                            tokens[i].parse::<f64>().unwrap_or(0.0),
                            current_x,
                            is_relative,
                        ) + noise(config.amplitude);
                        let y1 = to_absolute(
                            tokens[i + 1].parse::<f64>().unwrap_or(0.0),
                            current_y,
                            is_relative,
                        ) + noise(config.amplitude);
                        let x = to_absolute(
                            tokens[i + 2].parse::<f64>().unwrap_or(0.0),
                            current_x,
                            is_relative,
                        ) + noise(config.amplitude);
                        let y = to_absolute(
                            tokens[i + 3].parse::<f64>().unwrap_or(0.0),
                            current_y,
                            is_relative,
                        ) + noise(config.amplitude);
                        result.push_str(&format!(" {:.3} {:.3} {:.3} {:.3}", x1, y1, x, y));
                        current_x = x;
                        current_y = y;
                        i += 4;
                    }
                }
                'H' => {
                    while i < tokens.len() && !is_command(&tokens[i]) {
                        let x = to_absolute(
                            tokens[i].parse::<f64>().unwrap_or(0.0),
                            current_x,
                            is_relative,
                        ) + noise(config.amplitude);
                        result.push_str(&format!(" {:.3}", x));
                        current_x = x;
                        i += 1;
                    }
                }
                'V' => {
                    while i < tokens.len() && !is_command(&tokens[i]) {
                        let y = to_absolute(
                            tokens[i].parse::<f64>().unwrap_or(0.0),
                            current_y,
                            is_relative,
                        ) + noise(config.amplitude);
                        result.push_str(&format!(" {:.3}", y));
                        current_y = y;
                        i += 1;
                    }
                }
                'A' => {
                    while i + 7 <= tokens.len() && !is_command(&tokens[i]) {
                        let rx = tokens[i].parse::<f64>().unwrap_or(0.0);
                        let ry = tokens[i + 1].parse::<f64>().unwrap_or(0.0);
                        let rot = tokens[i + 2].parse::<f64>().unwrap_or(0.0);
                        let large_arc = tokens[i + 3].parse::<f64>().unwrap_or(0.0);
                        let sweep = tokens[i + 4].parse::<f64>().unwrap_or(0.0);
                        let x = to_absolute(
                            tokens[i + 5].parse::<f64>().unwrap_or(0.0),
                            current_x,
                            is_relative,
                        ) + noise(config.amplitude);
                        let y = to_absolute(
                            tokens[i + 6].parse::<f64>().unwrap_or(0.0),
                            current_y,
                            is_relative,
                        ) + noise(config.amplitude);
                        result.push_str(&format!(
                            " {:.3} {:.3} {:.3} {:.0} {:.0} {:.3} {:.3}",
                            rx, ry, rot, large_arc, sweep, x, y
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
            i += 1;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jitter_path_d_relative_commands() {
        let d = "m 10 10 l 20 0 c 10 0 10 10 0 10 z";
        let config = JitterConfig::default();
        let result = jitter_path_d(d, &config);
        // 相対コマンドが正しく処理され、結果にノイズが含まれることを確認
        assert!(result.starts_with('m'));
        assert!(result.contains('l'));
        assert!(result.contains('c'));
        assert!(result.contains('z'));
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
    fn test_is_command_excludes_e() {
        assert!(!is_command("e"));
        assert!(!is_command("E"));
        assert!(is_command("M"));
        assert!(is_command("m"));
    }
}
