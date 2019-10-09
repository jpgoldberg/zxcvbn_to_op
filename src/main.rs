extern crate zxcvbn;
extern crate float_cmp;

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
    let points = vec![
        Point { zx: 0.0, op: 1.0 },
        Point { zx: 4.0, op: 8.0 },
        Point { zx: 5.0, op: 22.0 },
        Point { zx: 7.5, op: 40.0 },
        Point { zx: 12.0, op: 57.0 },
        Point { zx: 15.0, op: 65.0 },
        Point { zx: 17.0, op: 88.0 },
        Point { zx: 20.0, op: 100.0 },
    ];
    let _foo = lines_from_points(points);
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

    Some(Box::new(move |x| x*m + b))
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
        let m = (line(first.zx) - line(second.zx))/ (first.zx - second.zx);

        println!("\nFor points {} and {}", first, second);
        println!("\ty = {}x + {}", m, b);
    }

    Some(out)
}
