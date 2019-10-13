use float_cmp::ApproxEqRatio;
/// This will need a better name than `my`, but I'm trying to follow the docs
/// and I can't think of anything better anyway.
use std::fmt;
use std::ops::{Div, Sub};

/// ZxScores are what we get from zxcvbn `guesses_log10`
#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct ZxScore(pub f32);

/// OpScores are the 1Password strength scores from 1 through 100
// We don't do as much math with these, so we don't need to derive as much
#[derive(PartialEq, Clone, Copy)]
pub struct OpScore(pub f32);

#[derive(Clone, Copy)]
pub struct Point {
    pub zx: ZxScore,
    pub op: OpScore,
}

impl ZxScore {
    pub fn to_f32(&self) -> f32 {
        self.0
    }
    pub fn value(&self) -> f32 {
        self.to_f32()
    }
}
impl OpScore {
    pub fn to_f32(&self) -> f32 {
        self.0
    }

    pub fn value(&self) -> f32 {
        self.to_f32()
    }
}

// Arithmetic for scores (we only need division and subtraction for now)
impl Div for ZxScore {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        ZxScore(self.value() / rhs.value())
    }
}

impl Sub for ZxScore {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        ZxScore(self.value() - other.value())
    }
}

impl Sub for OpScore {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        OpScore(self.value() - other.value())
    }
}

// The Display implementations
impl fmt::Display for ZxScore {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl fmt::Display for OpScore {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.zx, self.op)
    }
}

// And now for some computations

use std::f32::consts::{LN_10, LN_2};
#[allow(dead_code)]
impl ZxScore {
    /// ZxScores are very rough estimations. Converting them to bits doesn't change that
    pub fn to_bits(&self) -> f32 {
        self.value() * (LN_10 / LN_2)
    }
    pub fn from_bits(bits: f32) -> ZxScore {
        ZxScore(bits * (LN_2 / LN_10))
    }
}

#[allow(dead_code)]
impl OpScore {
    pub fn to_zx_score(&self, points: &'static [&'static Point]) -> Option<ZxScore> {
        // We need at least two points to create at least one line segment
        let op = self.value();
        if points.len() < 2 {
            return None;
        }
        if op < 0.0 || op > 100.0 {
            return None;
        }

        // Swaps x and y while converting to generic points
        let generic_points: &Vec<GenericPoint> = &points.iter().map(|p| GenericPoint {
            x: p.op.value() as f64,
            y: p.zx.value() as f64,
        }).collect();


        // The rest is really ugly, as I'm reusing code for the other direction,
        // but I typed things so strongly that I can't do without really confusing names
        struct Segment {
            upper: f64,
            line_function: Box<dyn Fn(f64) -> f64>,
        };

        let mut segments: Vec<Segment> = Vec::new();
        segments.reserve_exact(points.len() - 1);

        // this looping could be done more nicely with iter and nth(). But
        // I'm old and don't know these new fangled things that kids use today.
        //
        // Deliberately starts at 1, as we look at element and the one _before_ it
        for i in 1..generic_points.len() {
            let first = &generic_points[i - 1];
            let second = &generic_points[i];
            
            segments.push(Segment {
                upper: second.x,
                line_function: first.line_from_points(second)?,
            });
        }

        // segments now has all of the information need to compute the op score.
        let ret = (segments
            .into_iter()
            .find(|s| s.upper > op.into())
            // If we don't find anything, we've gone over the top of our defined range
            // and so return our maximum op_score
            .unwrap_or(Segment {
                upper: 101.0,
                line_function: Box::new(|_| 40.0),
            })
            .line_function)(op.into());

        Some(ZxScore(ret as f32))
    }
}

// returns a function the the strait line that goes through points p1 and p2
/// Technically this returns a closure, but the closure is boxed with fixed
/// values moved into it. So it's easier to think of it as a function.

#[derive(Clone, Copy)]
struct GenericPoint {
    x: f64,
    y: f64,
}

impl GenericPoint {
    fn line_from_points(&self, p2: &GenericPoint) -> Option<Box<dyn Fn(f64) -> f64>> {
        let p1 = self;

        if p1.x.approx_eq_ratio(&p2.x, 0.001) {
            return None;
        }

        // use American notation of y = mx + b
        let m = (p2.y - p1.y) / (p2.x - p1.x);
        let b = p1.y - (m * p1.x);

        Some(Box::new(move |x| m * x + b))
    }
    pub fn new_from_point(p: &Point) -> GenericPoint {
        GenericPoint {
            x: p.zx.value() as f64,
            y: p.op.value() as f64,
        }
    }
}

impl Point {
    #[allow(dead_code)]
    fn new_from_generic_point(p: &GenericPoint) -> Point {
        Point {
            zx: ZxScore(p.x as f32),
            op: OpScore(p.y as f32),
        }
    }
    pub fn line_from_points(&self, p2: &Point) -> Option<Box<dyn Fn(ZxScore) -> OpScore>> {
        let gp1 = &GenericPoint::new_from_point(self);
        let gp2 = &GenericPoint::new_from_point(p2);

        let generic_line = gp1.line_from_points(gp2)?;

        Some(Box::new(move |x| {
            OpScore(generic_line(x.value() as f64) as f32)
        }))
    }
}
