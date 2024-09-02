use crate::types::Transformer;
use std::convert::From;
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, Default)]
pub struct Point2 {
    x: f64,
    y: f64,
}

impl Point2 {
    pub fn random() -> Self {
        Self {
            x: rand::random::<f64>(),
            y: rand::random::<f64>(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Point3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Point3 {
    pub fn random() -> Self {
        Self {
            x: rand::random::<f64>(),
            y: rand::random::<f64>(),
            z: rand::random::<f64>(),
        }
    }
}

impl From<(Hsl, bool)> for Point3 {
    fn from((hsl, inverted): (Hsl, bool)) -> Self {
        let Hsl { h, s, l } = hsl;

        let cx = 0.5f64;
        let cy = 0.5f64;

        let radians = h / (180f64 / PI);

        let dist = (if inverted { 1.0 - l } else { l }) * cx;

        let x = cx + dist * radians.cos();
        let y = cy + dist * radians.sin();

        let z = s;

        Point3 { x, y, z }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Hsl {
    pub h: f64,
    pub s: f64,
    pub l: f64,
}

impl From<(Point3, bool)> for Hsl {
    fn from((point, inverted): (Point3, bool)) -> Self {
        let Point3 { x, y, z } = point;

        let cx = 0.5f64;
        let cy = 0.5f64;

        let radians = (y - cy).atan2(x - cx);

        let mut deg = radians * (180f64 / PI);
        deg = (360f64 + deg) % 360f64;

        let dist = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
        let l = dist / cx;
        let s = z;

        Hsl {
            h: deg,
            s,
            l: if inverted { 1.0 - l } else { l },
        }
    }
}

pub struct HslPairInit {
    start_hue: f64,
    saturation: Point2,
    lightness: Point2,
}

impl Default for HslPairInit {
    fn default() -> Self {
        Self {
            start_hue: rand::random::<f64>() * 360f64,
            saturation: Point2::random(),
            lightness: Point2 {
                x: 0.75 + rand::random::<f64>() * 0.2,
                y: 0.75 + rand::random::<f64>() * 0.2,
            },
        }
    }
}

pub struct HslTripleInit {
    start_hue: f64,
    saturation: Point3,
    lightness: Point3,
}

impl Default for HslTripleInit {
    fn default() -> Self {
        Self {
            start_hue: rand::random::<f64>() * 360f64,
            saturation: Point3::random(),
            lightness: Point3 {
                x: 0.75 + rand::random::<f64>() * 0.2,
                y: rand::random::<f64>() * 0.2,
                z: 0.75 + rand::random::<f64>() * 0.2,
            },
        }
    }
}

impl Hsl {
    pub fn random_pair(init: HslPairInit) -> [Self; 2] {
        let HslPairInit {
            start_hue: h,
            saturation: s,
            lightness: l,
        } = init;
        [
            Hsl { h, s: s.x, l: l.x },
            Hsl {
                h: (h + 60f64 + rand::random::<f64>() * 180f64) % 360f64,
                s: s.y,
                l: l.y,
            },
        ]
    }

    pub fn random_triple(init: HslTripleInit) -> [Self; 3] {
        let HslTripleInit {
            start_hue: h,
            saturation: s,
            lightness: l,
        } = init;

        [
            Hsl { h, s: s.x, l: l.x },
            Hsl {
                h: (h + 60f64 + rand::random::<f64>() * 180f64) % 360f64,
                s: s.y,
                l: l.y,
            },
            Hsl {
                h: (h + 60f64 + rand::random::<f64>() * 180f64) % 360f64,
                s: s.z,
                l: l.z,
            },
        ]
    }
}

fn vector_on_line(
    t: f64,
    p1: &Point3,
    p2: &Point3,
    inverted: bool,
    fx: Transformer,
    fy: Transformer,
    fz: Transformer,
) -> Point3 {
    let t_modified_x = fx(t, inverted);
    let t_modified_y = fy(t, inverted);
    let t_modified_z = fz(t, inverted);

    let x = (1f64 - t_modified_x) * p1.x + t_modified_x * p2.x;
    let y = (1f64 - t_modified_y) * p1.y + t_modified_y * p2.y;
    let z = (1f64 - t_modified_z) * p1.z + t_modified_z * p2.z;

    Point3 { x, y, z }
}

pub fn vectors_on_line(
    p1: &Point3,
    p2: &Point3,
    num_points: i32,
    inverted: bool,
    fx: Transformer,
    fy: Transformer,
    fz: Transformer,
) -> Vec<Point3> {
    (0..num_points)
        .map(move |i| {
            let t: f64 = i as f64 / (num_points - 1) as f64;
            vector_on_line(t, p1, p2, inverted, fx, fy, fz)
        })
        .collect()
}

pub struct PartialPoint3(Option<f64>, Option<f64>, Option<f64>);

impl PartialPoint3 {
    pub fn distance(&self, other: &PartialPoint3, hue_mode: Option<bool>) -> f64 {
        let a1 = self.0;
        let a2 = other.0;

        let a = match (hue_mode.unwrap_or(false), a1, a2) {
            (true, Some(a), Some(b)) => (a - b).abs().min(360f64 - (a - b).abs()) / 360f64,
            (false, Some(a), Some(b)) => a - b,
            _ => 0f64,
        };

        let b = match (self.1, other.1) {
            (Some(a), Some(b)) => b - a,
            _ => 0f64,
        };

        let c = match (self.2, other.2) {
            (Some(a), Some(b)) => b - a,
            _ => 0f64,
        };

        (a * a + b * b + c * c).sqrt()
    }
}

impl From<&Hsl> for PartialPoint3 {
    fn from(value: &Hsl) -> Self {
        Self(Some(value.h), Some(value.s), Some(value.l))
    }
}

impl From<&Point3> for PartialPoint3 {
    fn from(value: &Point3) -> Self {
        Self(Some(value.x), Some(value.y), Some(value.z))
    }
}

pub enum PointOrHsl {
    Point(Point3),
    Hsl(Hsl),
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct ColorPoint {
    pub color: Hsl,
    pub point: Point3,
    pub inverted: bool,
}

impl ColorPoint {
    pub fn set_inverted(&mut self, inverted: bool) {
        self.inverted = inverted;
    }

    pub fn set_postion(&mut self, point: Point3) {
        let new_color = Hsl::from((point, self.inverted));
        self.color = new_color;
        self.point = point;
    }

    pub fn set_hsl(&mut self, hsl: Hsl) {
        let new_point = Point3::from((hsl, self.inverted));
        self.color = hsl;
        self.point = new_point;
    }

    pub fn css_string(&self) -> String {
        format!(
            "hsl({:06.2}, {}%, {}%)",
            self.color.h,
            self.color.s * 100f64,
            self.color.l * 100f64
        )
    }

    pub fn shift_hue(&mut self, angle: f64) {
        self.color.h = (360f64 + (self.color.h + angle)) % 360f64;
        self.point = Point3::from((self.color, self.inverted));
    }
}

impl From<(Hsl, bool)> for ColorPoint {
    fn from((hsl, inverted): (Hsl, bool)) -> Self {
        ColorPoint {
            point: Point3::from((hsl, inverted)),
            color: hsl,
            inverted,
        }
    }
}

impl From<(Point3, bool)> for ColorPoint {
    fn from((point, inverted): (Point3, bool)) -> Self {
        ColorPoint {
            color: Hsl::from((point, inverted)),
            point,
            inverted,
        }
    }
}
