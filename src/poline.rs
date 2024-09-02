use std::{convert::From, iter::Peekable};

use crate::{
    error::PolineError,
    fns::PositionFn,
    point::{vectors_on_line, ColorPoint, Hsl, HslPairInit, PartialPoint3, PointOrHsl},
    types::Transformer,
};

pub struct Poline {
    anchor_points: Vec<ColorPoint>,
    num_points: i32,
    pos_fn_x: fn(f64, bool) -> f64,
    pos_fn_y: fn(f64, bool) -> f64,
    pos_fn_z: fn(f64, bool) -> f64,
    inverted: bool,
    points: Vec<Vec<ColorPoint>>,
    anchor_pairs: Vec<Vec<ColorPoint>>,
    closed_loop: bool,
}

pub struct PolineBuilder {
    anchor_colors: Vec<Hsl>,
    num_points: i32,
    position_fn_x: Option<fn(f64, bool) -> f64>,
    position_fn_y: Option<fn(f64, bool) -> f64>,
    position_fn_z: Option<fn(f64, bool) -> f64>,
    closed_loop: bool,
    inverted: bool,
}

impl PolineBuilder {
    pub fn num_points(mut self, points: i32) -> Self {
        self.num_points = points;
        self
    }

    pub fn anchor_points(mut self, points: Vec<Hsl>) -> Self {
        self.anchor_colors = points;
        self
    }

    pub fn closed_loop(mut self, closed_loop: bool) -> Self {
        self.closed_loop = closed_loop;
        self
    }

    pub fn set_x_fn(mut self, f: fn(f64, bool) -> f64) -> Self {
        self.position_fn_x = Some(f);
        self
    }

    pub fn set_y_fn(mut self, f: fn(f64, bool) -> f64) -> Self {
        self.position_fn_y = Some(f);
        self
    }

    pub fn set_z_fn(mut self, f: fn(f64, bool) -> f64) -> Self {
        self.position_fn_z = Some(f);
        self
    }

    pub fn invert_lightness(mut self, invert: bool) -> Self {
        self.inverted = invert;
        self
    }

    pub fn build(self) -> Result<Poline, PolineError> {
        if self.anchor_colors.len() < 2 {
            return Err(PolineError::InvalidAnchorColorCount);
        }

        let anchor_points: Vec<ColorPoint> = self
            .anchor_colors
            .into_iter()
            .map(|p| ColorPoint::from((p, self.inverted)))
            .collect();

        let num_points = self.num_points + 2; // add 2 for the anchor points
        let pos_fn_x = self
            .position_fn_x
            .unwrap_or(PositionFn::Sinusoidal.get_fn());
        let pos_fn_y = self
            .position_fn_y
            .unwrap_or(PositionFn::Sinusoidal.get_fn());
        let pos_fn_z = self
            .position_fn_z
            .unwrap_or(PositionFn::Sinusoidal.get_fn());

        let mut out = Poline {
            num_points,
            anchor_points,
            points: Vec::new(),
            anchor_pairs: Vec::new(),
            pos_fn_x,
            pos_fn_y,
            pos_fn_z,
            inverted: self.inverted,
            closed_loop: self.closed_loop,
        };
        out.update_anchor_pairs();
        Ok(out)
    }
}

impl Default for PolineBuilder {
    fn default() -> Self {
        Self {
            anchor_colors: Hsl::random_pair(HslPairInit::default())
                .into_iter()
                .collect(),
            num_points: 4,
            position_fn_x: None,
            position_fn_y: None,
            position_fn_z: None,
            closed_loop: false,
            inverted: false,
        }
    }
}

struct MaybeLast<I>
where
    I: Iterator,
{
    dont_take_last: bool,
    iter: Peekable<I>,
}

impl<I> Iterator for MaybeLast<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let this_val = self.iter.next();
        if !self.dont_take_last {
            return this_val;
        }
        match self.iter.peek() {
            Some(_) => this_val,
            None => None,
        }
    }
}

impl Poline {
    pub fn builder() -> PolineBuilder {
        PolineBuilder::default()
    }

    fn update_anchor_pairs(&mut self) {
        let mut anchor_pairs: Vec<Vec<ColorPoint>> = Vec::new();

        let anchor_points_length = if self.closed_loop {
            self.anchor_points.len()
        } else {
            self.anchor_points.len() - 1
        };

        for i in 0..anchor_points_length {
            let pair = vec![
                self.anchor_points[i],
                self.anchor_points[(i + 1) % self.anchor_points.len()],
            ];
            anchor_pairs.push(pair);
        }

        let points = anchor_pairs
            .iter()
            .enumerate()
            .map(|(i, pair)| -> Vec<ColorPoint> {
                let p1 = pair.first().map(|p| p.point).unwrap_or_default();
                let p2 = pair.get(1).map(|p| p.point).unwrap_or_default();

                vectors_on_line(
                    &p1,
                    &p2,
                    self.num_points,
                    (i % 2) == 0,
                    self.pos_fn_x,
                    self.pos_fn_y,
                    self.pos_fn_z,
                )
                .into_iter()
                .map(|p| ColorPoint::from((p, self.inverted)))
                .collect()
            })
            .collect();

        self.points = points;
        self.anchor_pairs = anchor_pairs;
    }

    pub fn num_points(&self) -> i32 {
        self.num_points - 2
    }

    pub fn set_num_points(&mut self, num: i32) -> Result<(), PolineError> {
        if num < 1 {
            return Err(PolineError::InvalidAnchorColorCount);
        }

        self.num_points = num + 2;
        self.update_anchor_pairs();
        Ok(())
    }

    pub fn set_position_fn(&mut self, fns: [Transformer; 3]) {
        self.pos_fn_x = fns[0];
        self.pos_fn_y = fns[1];
        self.pos_fn_z = fns[2];
        self.update_anchor_pairs();
    }

    pub fn position_fn(&self) -> [Transformer; 3] {
        [self.pos_fn_x, self.pos_fn_y, self.pos_fn_z]
    }

    pub fn anchor_points(&self) -> &Vec<ColorPoint> {
        &self.anchor_points
    }

    pub fn set_anchor_points(&mut self, points: Vec<ColorPoint>) {
        self.anchor_points = points;
        self.update_anchor_pairs();
    }

    pub fn add_anchor_point(
        &mut self,
        color_point: ColorPoint,
        at_index: Option<usize>,
    ) -> ColorPoint {
        let mut point = color_point;
        point.set_inverted(self.inverted);

        if let Some(index) = at_index {
            self.anchor_points.insert(index, point);
        } else {
            self.anchor_points.push(point);
        }

        self.update_anchor_pairs();
        point
    }

    pub fn remove_anchor_point(&mut self, index: usize) -> Result<ColorPoint, PolineError> {
        if index >= self.anchor_points.len() {
            return Err(PolineError::PointIndexOutOfBounds);
        }

        Ok(self.anchor_points.remove(index))
    }

    pub fn update_anchor_point(
        &mut self,
        index: usize,
        update: PointOrHsl,
    ) -> Result<ColorPoint, PolineError> {
        if index >= self.anchor_points.len() {
            return Err(PolineError::PointIndexOutOfBounds);
        }

        let mut out = self.anchor_points.remove(index);
        if let PointOrHsl::Point(point) = update {
            out.set_postion(point)
        } else if let PointOrHsl::Hsl(hsl) = update {
            out.set_hsl(hsl)
        }

        self.anchor_points.insert(index, out);

        Ok(out)
    }

    pub fn get_closest_anchor_point(&self, point_or_hsl: PointOrHsl) -> Option<(ColorPoint, f64)> {
        let distances: Vec<f64> = match point_or_hsl {
            PointOrHsl::Point(point) => self
                .anchor_points
                .iter()
                .map(|p| {
                    let p = p.point;
                    let other = PartialPoint3::from(&point);
                    PartialPoint3::from(&p).distance(&other, None)
                })
                .collect(),
            PointOrHsl::Hsl(hsl) => self
                .anchor_points
                .iter()
                .map(|p| {
                    let p = p.color;
                    let other = PartialPoint3::from(&hsl);
                    PartialPoint3::from(&p).distance(&other, None)
                })
                .collect(),
        };

        let min = distances
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.total_cmp(b))
            .unwrap_or((self.anchor_points.len(), &0.0));

        self.anchor_points.get(min.0).map(|p| (*p, *min.1))
    }

    pub fn closed_loop(&self) -> bool {
        self.closed_loop
    }

    pub fn set_closed_loop(&mut self, closed_loop: bool) {
        self.closed_loop = closed_loop;
        self.update_anchor_pairs();
    }

    pub fn inverted(&self) -> bool {
        self.inverted
    }

    pub fn set_inverted(&mut self, inverted: bool) {
        self.inverted = inverted;
        self.update_anchor_pairs();
    }

    pub fn flattened_points(&self) -> impl Iterator<Item = ColorPoint> + '_ {
        self.points
            .iter()
            .flatten()
            .enumerate()
            .filter_map(|(i, p)| {
                if i == 0 {
                    return Some(*p);
                }
                if i % self.num_points as usize == 0 {
                    None
                } else {
                    Some(*p)
                }
            })
    }

    pub fn colors(&self) -> impl Iterator<Item = Hsl> + '_ {
        // if this is a closed loop dont return the last element
        MaybeLast {
            iter: self.flattened_points().map(|p| p.color).peekable(),
            dont_take_last: self.closed_loop,
        }
    }

    pub fn colors_css(&self) -> impl Iterator<Item = String> + '_ {
        MaybeLast {
            iter: self.flattened_points().map(|p| p.css_string()).peekable(),
            dont_take_last: self.closed_loop,
        }
    }

    pub fn shift_hue(&mut self, h_shift: Option<f64>) {
        let val = h_shift.unwrap_or(20f64);
        self.anchor_points.iter_mut().for_each(|p| p.shift_hue(val));
        self.update_anchor_pairs()
    }
}
