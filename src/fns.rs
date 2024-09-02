use std::f64::consts::PI;

use crate::types::Transformer;

pub enum PositionFn {
    Linear,
    Quadratic,
    Cubic,
    Quartic,
    Quintic,
    Sinusoidal,
    Asinusoidal,
    Arc,
    SmoothStep,
}

impl PositionFn {
    pub fn get_fn(&self) -> Transformer {
        match self {
            PositionFn::Linear => linear_fn,
            PositionFn::Quadratic => quadratic_fn,
            PositionFn::Cubic => cubic_fn,
            PositionFn::Quartic => quartic_fn,
            PositionFn::Quintic => quintic_fn,
            PositionFn::Sinusoidal => sinusoidal_fn,
            PositionFn::Asinusoidal => asinusoidal_fn,
            PositionFn::Arc => arc_fn,
            PositionFn::SmoothStep => smooth_step_fn,
        }
    }
}

fn linear_fn(t: f64, _inverted: bool) -> f64 {
    t
}

fn quadratic_fn(t: f64, inverted: bool) -> f64 {
    if inverted {
        1f64 - (1f64 - t).powi(2)
    } else {
        t.powi(2)
    }
}

fn cubic_fn(t: f64, inverted: bool) -> f64 {
    if inverted {
        1f64 - (1f64 - t).powi(3)
    } else {
        t.powi(3)
    }
}

fn quartic_fn(t: f64, inverted: bool) -> f64 {
    if inverted {
        1f64 - (1f64 - t).powi(4)
    } else {
        t.powi(4)
    }
}

fn quintic_fn(t: f64, inverted: bool) -> f64 {
    if inverted {
        1f64 - (1f64 - t).powi(5)
    } else {
        t.powi(5)
    }
}

fn sinusoidal_fn(t: f64, inverted: bool) -> f64 {
    if inverted {
        1f64 - ((1f64 - t) * PI / 2f64).sin()
    } else {
        (t * PI / 2f64).sin()
    }
}

fn asinusoidal_fn(t: f64, inverted: bool) -> f64 {
    if inverted {
        1f64 - (1f64 - t).asin() / (PI / 2f64)
    } else {
        t.asin() / (PI / 2f64)
    }
}

fn arc_fn(t: f64, inverted: bool) -> f64 {
    if inverted {
        (1f64 - (1f64 - t).powi(2)).sqrt()
    } else {
        1f64 - (1f64 - t).sqrt()
    }
}

fn smooth_step_fn(t: f64, _inverted: bool) -> f64 {
    t.powi(2) * (3f64 - 2f64 * t)
}
