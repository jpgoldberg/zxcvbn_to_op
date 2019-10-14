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
        let op = op_score_from_zxcvbn(ZxScore(*z), CONTROL_POINTS)
            .expect(&format!("expected score for {}", z));
        println!("f({}) = {}", z, op);
    }
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
            line_function: first.line_from_points(second)?,
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

        // we are not going to get the expected conversion,
        // so we just print out results

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
