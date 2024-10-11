use std::iter::Sum;
use crate::primitives::ec::Point;
use anyhow::anyhow;
use num_bigint::BigInt;
use std::ops::{Add, Deref};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Commitment {
    pub c: Point,
}

impl Default for Commitment {
    fn default() -> Self {
        Self {
            c: Point::infinity()
        }
    }
}

impl Commitment {
    pub fn new(c: Point) -> Self {
        Self { c }
    }
    
    pub fn commit(w: &BigInt, r: &BigInt) -> Commitment {
        let g_w = Point::default() * w;
        let h_r = Point::default() * r;
        Commitment { c: &g_w + &h_r }
    }

    pub fn open(&self, w: &BigInt, r: &BigInt) -> anyhow::Result<()> {
        if self.c != Self::commit(w, r).c {
            return Err(anyhow!("Open failed."));
        }
        Ok(())
    }
}

impl Add for &Commitment {
    type Output = Commitment;

    fn add(self, rhs: Self) -> Self::Output {
        Commitment { c: &self.c + &rhs.c }
    }
}

impl Sum for Commitment {
    fn sum<I: Iterator<Item=Self>>(iter: I) -> Self {
        let mut res = Point::infinity();
        for el in iter {
            res = &res + &el.c
        }
        Commitment::new(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commitment() {
        let w = BigInt::from(5);
        let r = BigInt::from(7);
        let commitment = Commitment::commit(&w, &r);
        assert!(commitment.open(&w, &r).is_ok());
    }

    #[test]
    fn test_invalid_commitment() {
        let w = BigInt::from(5);
        let r = BigInt::from(7);
        let commitment = Commitment::commit(&w, &r);
        assert!(commitment.open(&BigInt::from(10), &r).is_err());
    }

    #[test]
    fn test_add_commitment() {
        let w1 = BigInt::from(5);
        let r1 = BigInt::from(7);
        let c1 = Commitment::commit(&w1, &r1);

        let w2 = BigInt::from(123);
        let r2 = BigInt::from(321);
        let c2 = Commitment::commit(&w2, &r2);

        let sum = &c1 + &c2;

        assert!(sum.open(&(w1 + w2), &(r1 + r2)).is_ok());
    }

    #[test]
    fn test_invalid_add_commitment() {
        let w1 = BigInt::from(5);
        let r1 = BigInt::from(7);
        let c1 = Commitment::commit(&w1, &r1);

        let w2 = BigInt::from(123);
        let r2 = BigInt::from(321);
        let c2 = Commitment::commit(&w2, &r2);

        let sum = &c1 + &c2;

        assert!(sum.open(&(w1 - w2), &(r1 + r2)).is_err());
    }
}
