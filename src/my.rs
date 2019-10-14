use float_cmp::ApproxEqRatio;
/// This will need a better name than `my`, but I'm trying to follow the docs
/// and I can't think of anything better anyway.
use std::fmt;
use std::ops::{Div, Sub};

use crate::MAX_OP_STRENGTH_SCORE;

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

    pub fn to_op_score(&self, points: &'static [&'static Point]) -> Option<OpScore> {
        // We need at least two points to create at least one line segment
        let zx = self.value();

        if zx < 0.0 {
            return None;
        }

        let g_pts: &Vec<GenericPoint> = &points
            .iter()
            .map(|p| GenericPoint {
                x: p.zx.value() as f64,
                y: p.op.value() as f64,
            })
            .collect();
        let max_out = MAX_OP_STRENGTH_SCORE.value() as f64;
        let ret = connected_lines_at(g_pts.to_vec(), zx as f64, max_out)?;
        let ret = ret as f32;
        Some(OpScore(ret))
    }
}

#[derive(Clone, Copy)]
struct GenericPoint {
    x: f64,
    y: f64,
}

#[allow(dead_code)]
impl OpScore {
    pub fn to_zx_score(&self, points: &'static [&'static Point]) -> Option<ZxScore> {
        // We need at least two points to create at least one line segment
        let op = self.value();

        if op < 0.0 || op > 100.0 {
            return None;
        }

        // Swaps x and y while converting to generic points
        let generic_points: &Vec<GenericPoint> = &points
            .iter()
            .map(|p| GenericPoint {
                x: p.op.value() as f64,
                y: p.zx.value() as f64,
            })
            .collect();

        let max_out = 40.0; // corresponds to roughly 128 bits
        let ret = connected_lines_at(generic_points.to_vec(), op as f64, max_out)?;
        let ret = ret as f32;
        Some(ZxScore(ret))
    }
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

/// This will return None if any of the following are true
/// - There aren't at least two points from which to create a line
/// - The x values of adjacent points are too close to each other
/// - The points are not ordered ascending in x value
/// - The input x is greater than the largest x value for a point
///
/// This is why this function is not public. Public callers should
/// make sure that input is sane or handle errors
fn connected_lines_at(points: Vec<GenericPoint>, x: f64, max: f64) -> Option<f64> {
    if points.len() < 2 {
        return None;
    }

    struct Segment {
        upper: f64,
        line_function: Box<dyn Fn(f64) -> f64>,
    };
    let mut segments: Vec<Segment> = Vec::new();
    segments.reserve_exact(points.len());

    for pair in points.windows(2) {
        let first = &pair[0];
        let second = &pair[1];

        if !(second.x > first.x) {
            return None;
        }

        segments.push(Segment {
            upper: second.x,
            line_function: first.line_from_points(second)?,
        })
    }

    // segments now has all of the information need to compute the op score.
    Some((segments
        .into_iter()
        .find(|s| s.upper > x)
        .unwrap_or(Segment {
            upper: 0.0,
            line_function: Box::new(move |_| max),
        })
        .line_function)(x))
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::ApproxEqRatio;

    const TEST_POINTS: &'static [&'static Point] = &[
        &Point {
            zx: ZxScore(0.0),
            op: OpScore(0.0),
        }, // y = (1/2)x + 0
        &Point {
            zx: ZxScore(40.0),
            op: OpScore(20.0),
        }, // y = 4x + 20
        &Point {
            zx: ZxScore(60.0),
            op: OpScore(100.0),
        },
    ];

    struct TestVector {
        zx: f32,
        expected: f32,
    }

    #[test]
    fn test_interpolation() {
        // These tests are build around TEST_POINTS, so do not depend on our actual
        // control points.
        let tests = &[
            TestVector {
                zx: 0.0,
                expected: 0.0,
            },
            TestVector {
                zx: 6.0,
                expected: 3.0,
            },
            TestVector {
                zx: 12.0,
                expected: 6.0,
            },
            TestVector {
                zx: 18.0,
                expected: 9.0,
            },
            TestVector {
                zx: 24.0,
                expected: 12.0,
            },
            TestVector {
                zx: 30.0,
                expected: 15.0,
            },
            TestVector {
                zx: 39.0,
                expected: 19.5,
            },
            TestVector {
                zx: 40.0,
                expected: 20.0,
            },
            TestVector {
                zx: 41.0,
                expected: 24.0,
            },
            TestVector {
                zx: 44.0,
                expected: 36.0,
            },
            TestVector {
                zx: 50.0,
                expected: 60.0,
            },
            TestVector {
                zx: 59.0,
                expected: 96.0,
            },
            TestVector {
                zx: 60.0,
                expected: 100.0,
            },
            TestVector {
                zx: 61.0,
                expected: MAX_OP_STRENGTH_SCORE.value(),
            },
        ];

        for t in tests {
            let z = ZxScore(t.zx);
            // let op = op_score_from_zxcvbn(z, TEST_POINTS).unwrap().value();
            let op = z.to_op_score(TEST_POINTS).unwrap();
            assert!(
                op.to_f32().approx_eq_ratio(&t.expected, 0.01),
                "f({}) should be {}. Got {}",
                t.zx,
                t.expected,
                op
            );
        }
    }

    #[test]
    fn test_inverse() {
        // can only test for values for which the function and its
        // inverse are well defined.
        for i in 1..8 {
            let t = ZxScore(i as f32 * 2.5);
            let op = t.to_op_score(&crate::CONTROL_POINTS).unwrap();
            let zx = op.to_zx_score(&crate::CONTROL_POINTS).unwrap();
            assert!(
                t.value().approx_eq_ratio(&zx.value(), 0.0001),
                "t ({}) != zx ({})",
                t,
                op
            );
        }
    }
}
