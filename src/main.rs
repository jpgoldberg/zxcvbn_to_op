extern crate float_cmp;

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
        let op = ZxScore(*z)
            .to_op_score(CONTROL_POINTS)
            .expect(&format!("expected score for {}", z));
        println!("f({}) = {}", z, op);
    }
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
        let line = first.line_from_points(second)?;

        let b = line(ZxScore(0.0));
        let m = (line(first.zx) - line(second.zx)).to_f32() / (first.zx - second.zx).to_f32();

        messages.push_str(&format!("\nFor points {} and {}:", first, second));
        messages.push_str(&format!("\ty = {}x + {}", m, b));
    }

    Some(messages)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    // run with `cargo test -- --nocapture --ignored`
    fn test_b5book_entropy_scale() {
        struct TestData {
            top_of: String,
            bits: f32,
            op: f32,
        }

        let tests = &[
            TestData {
                top_of: "Terrible".to_string(),
                bits: 20.0,
                op: 26.0,
            },
            TestData {
                top_of: "Weak".to_string(),
                bits: 33.0,
                op: 44.0,
            },
            TestData {
                top_of: "Fair".to_string(),
                bits: 40.0,
                op: 53.0,
            },
            TestData {
                top_of: "Good".to_string(),
                bits: 45.0,
                op: 60.0,
            },
            TestData {
                top_of: "Very Good".to_string(),
                bits: 55.0,
                op: 73.0,
            },
            TestData {
                top_of: "Excellent".to_string(),
                bits: 64.0,
                op: 85.0,
            },
            TestData {
                top_of: "Fantastic".to_string(),
                bits: 75.0,
                op: 85.0,
            },
        ];

        for t in tests {
            let op = ZxScore::from_bits(t.bits)
                .to_op_score(CONTROL_POINTS)
                .unwrap();

            // run with nocapture to see results
            println!("For top of {}", t.top_of);
            println!("\t{} bits is {} zscore", t.bits, ZxScore::from_bits(t.bits));
            println!("\top is {}, Target: {}", op, t.op);
        }
    }
}
