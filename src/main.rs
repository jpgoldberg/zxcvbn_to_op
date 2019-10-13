extern crate float_cmp;

use float_cmp::ApproxEqRatio;

mod my;
use my::*;

const MAX_OP_STRENGTH_SCORE: OpScore = OpScore(100.0);

/// // control points determined by eyeballing OP vs ZXCVBN scatter plot
const CONTROL_POINTS: &'static [&'static Point] = &[
    &Point {
        zx: ZxScore(0.0),
        op: OpScore(1.0),
    },
    &Point {
        zx: ZxScore(4.0),
        op: OpScore(8.0),
    },
    &Point {
        zx: ZxScore(8.0),
        op: OpScore(45.0),
    },
    &Point {
        zx: ZxScore(14.0),
        op: OpScore(57.0),
    },
    &Point {
        zx: ZxScore(20.0),
        op: OpScore(100.0),
    },
];

fn main() {
    println!(
        "{}",
        equations_points(CONTROL_POINTS).expect("expected message")
    ); // just print out what our functions are

    // should really be doing this in test functions
    for z in &[
        0.0, 0.3, 2.0, 3.5, 4.2, 5.5, 6.2, 8.4, 9.0, 11.0, 12.0, 13.6, 15.0, 16.0, 17.0, 18.1,
        19.0, 19.9, 21.0, 24.0, 120.0,
    ] {
        let op =
            op_score_from_zxcvbn(ZxScore(*z), CONTROL_POINTS).expect(&format!("expected score for {}", z));
        println!("f({}) = {}", z, op);
    }
}

/// returns a function the the strait line that goes through points p1 and p2
/// Technically this returns a closure, but the closure is boxed with fixed
/// values moved into it. So it's easier to think of it as a function.
fn line_from_points(p1: &Point, p2: &Point) -> Option<Box<dyn Fn(ZxScore) -> OpScore>> {
    // just some convenience naming
    let x1 = p1.zx.to_f32();
    let y1 = p1.op.to_f32();
    let x2 = p2.zx.to_f32();
    let y2 = p2.op.to_f32();

    if x1.approx_eq_ratio(&x2, 0.001) {
        return None;
    }

    // use American notation of y = mx + b
    let m = (y2 - y1) / (x2 - x1);
    let b = y1 - (m * x1);

    Some(Box::new(move |x| OpScore(x.value() * m + b)))
}

fn op_score_from_zxcvbn(zx_score: ZxScore, points: &'static [&'static Point]) -> Option<OpScore> {
    // We need to create a sequence of linear functions based on pairs of points

    // We need at least two points to create at least one line segment
    if points.len() < 2 {
        return None;
    }

    if zx_score.to_f32() < 0.0 {
        return None; // weird input should signal some sort of error
    }

    // Internally, we need to keep a list of range endpoints and the function
    // we have for that range

    struct Segment {
        upper: ZxScore,
        line_function: Box<dyn Fn(ZxScore) -> OpScore>,
    };

    let mut segments: Vec<Segment> = Vec::new();
    segments.reserve_exact(points.len() - 1);

    // this looping could be done more nicely with iter and nth(). But
    // I'm old and don't know these new fangled things that kids use today.
    //
    // Deliberately starts at 1, as we look at element and the one _before_ it
    for i in 1..points.len() {
        let first = &points[i - 1];
        let second = &points[i];
        segments.push(Segment {
            upper: second.zx,
            line_function: line_from_points(first, second)?,
        });
    }

    // segments now has all of the information need to compute the op score.
    let ret = (segments
        .into_iter()
        .find(|s| s.upper > zx_score)
        // If we don't find anything, we've gone over the top of our defined range
        // and so return our maximum op_score
        .unwrap_or(Segment {
            upper: ZxScore(std::f32::MAX),
            line_function: Box::new(|_| MAX_OP_STRENGTH_SCORE),
        })
        .line_function)(zx_score);

    Some(ret)
}

// This is just for printing out information about what is
// computed from sets of points. It plays no role in actually converting
// anything
fn equations_points(points: &'static [&'static Point]) -> Option<String> {
    // assumes that points are already sorted
    if points.len() < 2 {
        return Some(format!(
            "Not enough points ({}) to create any equations",
            points.len()
        ));
    }

    let mut messages = String::new();

    for pair in points.windows(2) {
        let first = pair[0];
        let second = pair[1];
        let line = line_from_points(first, second)?;

        let b = line(ZxScore(0.0));
        let m = (line(first.zx) - line(second.zx)).to_f32() / (first.zx - second.zx).to_f32();

        messages.push_str(&format!("\nFor points {} and {}:", first, second));
        messages.push_str(&format!("\ty = {}x + {}", m, b));
    }

    Some(messages)
}
