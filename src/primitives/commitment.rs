use crate::primitives::ec::Point;
use num_bigint::BigInt;
use std::ops::Add;

#[derive(Debug, Clone, Eq, PartialEq)]
struct Commitment {
    pub c: Point,
}

impl Commitment {
    fn commit(w: &BigInt, r: &BigInt) -> Commitment {
        let g_w = Point::default() * w;
        let h_r = Point::default() * r;
        Commitment { c: g_w + h_r }
    }

    fn open(&self, w: &BigInt, r: &BigInt) -> bool {
        self.c == Self::commit(w, r).c
    }
}

impl Add for Commitment {
    type Output = Commitment;

    fn add(self, rhs: Self) -> Self::Output {
        Commitment { c: self.c + rhs.c }
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
        assert!(commitment.open(&w, &r));
    }

    #[test]
    fn test_invalid_commitment() {
        let w = BigInt::from(5);
        let r = BigInt::from(7);
        let commitment = Commitment::commit(&w, &r);
        assert_eq!(commitment.open(&BigInt::from(10), &r), false);
    }

    #[test]
    fn test_add_commitment() {
        let w1 = BigInt::from(5);
        let r1 = BigInt::from(7);
        let c1 = Commitment::commit(&w1, &r1);

        let w2 = BigInt::from(123);
        let r2 = BigInt::from(321);
        let c2 = Commitment::commit(&w2, &r2);

        let sum = c1 + c2;

        assert!(sum.open(&(w1 + w2), &(r1 + r2)));
    }

    #[test]
    fn test_invalid_add_commitment() {
        let w1 = BigInt::from(5);
        let r1 = BigInt::from(7);
        let c1 = Commitment::commit(&w1, &r1);

        let w2 = BigInt::from(123);
        let r2 = BigInt::from(321);
        let c2 = Commitment::commit(&w2, &r2);

        let sum = c1 + c2;

        assert_eq!(sum.open(&(w1 - w2), &(r1 + r2)), false);
    }
}
