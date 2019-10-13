/// This will need a better name than `my`, but I'm trying to follow the docs
/// and I can't think of anything better anyway.

use std::fmt;
use std::ops::{Sub,Div};

/// ZxScores are what we get from zxcvbn `guesses_log10`
#[derive(PartialEq,PartialOrd,Clone,Copy)]
pub struct ZxScore(pub f32);

/// OpScores are the 1Password strength scores from 1 through 100
// We don't do as much math with these, so we don't need to derive as much
#[derive(PartialEq,Clone,Copy)]
pub struct OpScore(pub f32);

#[derive(Clone,Copy)]
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
        ZxScore(self.value()/rhs.value())
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

use std::f32::consts::{LN_2,LN_10};
#[allow(dead_code)]
impl ZxScore {
    /// ZxScores are very rough estimations. Converting them to bits doesn't change that
    pub fn to_bits(&self) -> f32 {
        self.value() * (LN_10/LN_2)
    }
    pub fn from_bits(bits: f32) -> ZxScore {
        ZxScore(bits * (LN_2/LN_10))
    }
}