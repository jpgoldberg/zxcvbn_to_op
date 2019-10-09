extern crate float_cmp;
extern crate zxcvbn;

use zxcvbn::{zxcvbn, ZxcvbnError};

use float_cmp::ApproxEqRatio;

use std::fmt;
use std::io;
use std::io::prelude::*;

fn old_main() {
    let mut pwd = String::new();
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        pwd = line.unwrap();
        let g = zxcvbn_bits(pwd.clone()).unwrap();
        println!("{}\t{}", pwd, g);
    }
}

fn main() {
    // control points determined by eyeballing OP vs ZXCVBN scatter plot
    let points = vec![
        Point { zx: 0.0, op: 1.0 },
        Point { zx: 4.0, op: 8.0 },
        Point { zx: 5.0, op: 22.0 },
        Point { zx: 7.5, op: 40.0 },
        Point { zx: 12.0, op: 57.0 },
        Point { zx: 15.0, op: 65.0 },
        Point { zx: 17.0, op: 88.0 },
        Point {
            zx: 20.0,
            op: 100.0,
        },
    ];
    let foo = mapping_from_control_points(points).unwrap();

    // should really be doing this in test functions
    for z in &[-1.0, 0.0, 0.3, 2.0, 3.5, 4.2, 5.5, 6.2, 8.4, 9.0,
                11.0, 12.0, 13.6, 15.0, 16.0, 17.0, 18.1, 19.0, 19.9, 21.0] {
        println!("f({}) = {}", z, foo(*z as f32));
    }
}

#[derive(Debug)]
struct GuessEnt(u64, f64, u32);

impl fmt::Display for GuessEnt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // write!(f, "Guesses: {}, Bits: {}", self.0, self.2)
        write!(f, "{}", self.1)
    }
}

fn zxcvbn_bits(input: String) -> Result<GuessEnt, ZxcvbnError> {
    let estimate = zxcvbn(&input, &[])?;
    let guessing_bits = (estimate.guesses as f32).log(2.0) as u32 + 1;

    Ok(GuessEnt(
        estimate.guesses,
        estimate.guesses_log10,
        guessing_bits,
    ))
}

// just need to create the mapping function from points
struct Point {
    zx: f32,
    op: f32,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.zx, self.op)
    }
}

/// returns a function the the strait line that goes through points p1 and p2
/// Technically this returns a closure, but the closure is boxed with fixed
/// values moved into it. So it's easier to think of it as a function.
fn line_from_points(p1: &Point, p2: &Point) -> Option<Box<dyn Fn(f32) -> f32>> {
    // just some convenience naming
    let x1 = p1.zx;
    let y1 = p1.op;
    let x2 = p2.zx;
    let y2 = p2.op;

    if x1.approx_eq_ratio(&x2, 0.001) {
        return None;
    }

    // use American notation of y = mx + b
    let m = (y2 - y1) / (x2 - x1);
    let b = y1 - (m * x1);

    Some(Box::new(move |x| x * m + b))
}

/// This assumes that the input is already sorted properly. Might add sorting later
fn mapping_from_control_points(points: Vec<Point>) -> Option<Box<dyn Fn(f32) -> f32>> {
    // We need to create a sequence of linear functions based on pairs of points

    if points.len() < 2 {
        return None;
    }

    // Internally, we need to keep a list of range endpoints and the function
    // we have for that range

    struct Segment {
        lower: f32,
        upper: f32,
        line_function: Box<dyn Fn(f32) -> f32>,
    };

    let mut segments: Vec<Segment> = Vec::new();
    segments.reserve_exact(points.len() - 1);

    // this looping could be done more nicely with iter and nth(). But
    // I'm old and don't know these new fangled things that kids use today.
    //
    // Deliberately starts at 1, as we look at element and the _previous_ element
    for i in 1..points.len() {
        let first = &points[i - 1];
        let second = &points[i];
        segments.push(Segment {
            lower: first.zx,
            upper: second.zx,
            line_function: line_from_points(first, second)?,
        });
    }

    // segments has all of the information I need to build the actual function that we return
    // if creating it manually, and knowing how many segments there were, we'd use a match
    // construction. There probably is a clever way to do that, but let's not be so clever

    Some(Box::new(move |x| {
        let mut ret = 100.0;
        for s in &segments {
            if x < s.upper {
               ret = (s.line_function)(x);
            }
        }
        ret
    }))
}

fn lines_from_points(points: Vec<Point>) -> Option<String> {
    // assumes that points are already sorted
    if points.len() < 2 {
        return None;
    }

    let out = String::from("string to keep IDE happy while I'm working");

    // we loop through from the second member, and we need an index.
    // Probably could use nth() but that requires reading docs
    for i in 1..points.len() {
        let first = &points[i - 1];
        let second = &points[i];
        let line = line_from_points(first, second).unwrap();

        // just for debugging, I want a local m and b computed from line

        assert!(line(first.zx).approx_eq_ratio(&first.op, 0.0001));
        assert!(line(second.zx).approx_eq_ratio(&second.op, 0.0001));

        let b = line(0.0);
        let m = (line(first.zx) - line(second.zx)) / (first.zx - second.zx);

        println!("\nFor points {} and {}", first, second);
        println!("\ty = {}x + {}", m, b);
    }

    Some(out)
}
