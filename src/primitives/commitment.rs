use crate::primitives::ec::{Point, EC};
use num_bigint::BigInt;

#[derive(Debug, Clone, Eq, PartialEq)]
struct Commitment {
    pub c: Point,
}

impl Commitment {
    fn commit(ec: &EC, w: &BigInt, r: &BigInt) -> Commitment {
        let g_w = ec.mul(EC::generator(), w);
        let h_r = ec.mul(EC::generator(), r);
        Commitment {
            c: ec.add(g_w, h_r),
        }
    }

    fn add(self, ec: &EC, commitment: &Commitment) -> Commitment {
        Commitment {
            c: ec.add(self.c, commitment.c.clone()),
        }
    }

    fn open(&self, ec: &EC, w: &BigInt, r: &BigInt) -> bool {
        let commitment = Self::commit(ec, w, r);
        self.c == commitment.c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commitment() {
        let ec = EC::default();
        let w = BigInt::from(5);
        let r = BigInt::from(7);
        let commitment = Commitment::commit(&ec, &w, &r);
        assert!(commitment.open(&ec, &w, &r));
    }

    #[test]
    fn test_invalid_commitment() {
        let ec = EC::default();
        let w = BigInt::from(5);
        let r = BigInt::from(7);
        let commitment = Commitment::commit(&ec, &w, &r);
        assert_eq!(commitment.open(&ec, &BigInt::from(10), &r), false);
    }

    #[test]
    fn test_add_commitment() {
        let ec = EC::default();
        let w1 = BigInt::from(5);
        let r1 = BigInt::from(7);
        let c1 = Commitment::commit(&ec, &w1, &r1);

        let w2 = BigInt::from(123);
        let r2 = BigInt::from(321);
        let c2 = Commitment::commit(&ec, &w2, &r2);

        let sum = c1.add(&ec, &c2);

        assert!(sum.open(&ec, &(w1 + w2), &(r1 + r2)));
    }

    #[test]
    fn test_invalid_add_commitment() {
        let ec = EC::default();
        let w1 = BigInt::from(5);
        let r1 = BigInt::from(7);
        let c1 = Commitment::commit(&ec, &w1, &r1);

        let w2 = BigInt::from(123);
        let r2 = BigInt::from(321);
        let c2 = Commitment::commit(&ec, &w2, &r2);

        let sum = c1.add(&ec, &c2);

        assert_eq!(sum.open(&ec, &(w1 - w2), &(r1 + r2)), false);
    }
}
