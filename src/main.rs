use zxcvbn_to_op::convert_zxcvbn_guesses_log10_to_op_strength;

fn main() {
    // should really be doing this in test functions
    for z in [
        0.0, 0.3, 2.0, 3.5, 4.2, 5.5, 6.2, 8.4, 9.0, 11.0, 12.0, 13.6, 15.0, 16.0, 17.0, 18.1,
        19.0, 19.9, 21.0, 24.0, 120.0,
    ]
    .iter()
    {
        let op = convert_zxcvbn_guesses_log10_to_op_strength(*z)
            .expect(&format!("expected score for {}", z));
        println!("f({}) = {}", z, op);
    }
}
