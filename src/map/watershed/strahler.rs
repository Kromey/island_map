use std::ops::{Add, AddAssign};

/// Strahler Number
/// 
/// https://en.wikipedia.org/wiki/Strahler_number#River_networks
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Strahler(u32);

impl Add for Strahler {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        if self == other {
            Self(self.0 + 1)
        } else {
            Self(self.0.max(other.0))
        }
    }
}

impl AddAssign for Strahler {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Default for Strahler {
    fn default() -> Self {
        Strahler(0)
    }
}

impl From<u32> for Strahler {
    fn from(from: u32) -> Strahler {
        Strahler(from)
    }
}

impl From<Strahler> for u32 {
    fn from(from: Strahler) -> u32 {
        from.0
    }
}
