use crate::primitives::ec::{Point, DEFAULTEC};
use anyhow::anyhow;
use num_bigint::BigInt;
use num_traits::{One, ToPrimitive, Zero};
use rand::{thread_rng, Rng};

#[derive(Debug, Clone, Eq, PartialEq)]
struct PollardRhoSequence {
    p1: Point,
    p2: Point,
    a1: BigInt,
    b1: BigInt,
    x1: Point,
    a2: BigInt,
    b2: BigInt,
    x2: Point,
    a: BigInt,
    b: BigInt,
    x: Point,
}

impl PollardRhoSequence {
    fn build(p1: Point, p2: Point) -> Self {
        let n = DEFAULTEC.n.to_u64().unwrap();

        let a1: BigInt = BigInt::from(thread_rng().gen_range(1..=n));
        let b1: BigInt = BigInt::from(thread_rng().gen_range(1..=n));
        let x1 = &(p1.clone() * &a1) + &(p2.clone() * &b1);

        let a2: BigInt = BigInt::from(thread_rng().gen_range(1..=n));
        let b2: BigInt = BigInt::from(thread_rng().gen_range(1..=n));
        let x2 = &(p1.clone() * &a2) + &(p2.clone() * &b2);

        Self {
            p1,
            p2,
            a1,
            b1,
            x1,
            a2,
            b2,
            x2,
            a: BigInt::zero(),
            b: BigInt::zero(),
            x: Point::infinity(),
        }
    }
}

impl Iterator for PollardRhoSequence {
    type Item = (Point, BigInt, BigInt);

    fn next(&mut self) -> Option<Self::Item> {
        let i = self
            .x
            .clone()
            .x
            .map_or_else(BigInt::zero, |x| x / (&DEFAULTEC.p / 3 + 1));

        if i == BigInt::zero() {
            self.a += &self.a1;
            self.b += &self.b1;
            self.x = &self.x + &self.x1;
        } else if i == BigInt::one() {
            self.a *= BigInt::from(2);
            self.b *= BigInt::from(2);
            self.x = self.x.double();
        } else if i == BigInt::from(2) {
            self.a += &self.a2;
            self.b += &self.b2;
            self.x = &self.x + &self.x2;
        } else {
            panic!("Iterator error.")
        }

        self.a = self.a.modpow(&BigInt::one(), &DEFAULTEC.n);
        self.b = self.b.modpow(&BigInt::one(), &DEFAULTEC.n);

        Some((self.x.clone(), self.a.clone(), self.b.clone()))
    }
}

pub fn pollards_rho(g: &Point, p: &Point) -> anyhow::Result<BigInt> {
    for _ in 0..3 {
        let sequence = PollardRhoSequence::build(g.clone(), p.clone());

        let mut tortoise = sequence.clone();
        let mut hare = sequence;

        let n = DEFAULTEC.n.to_u64().unwrap();

        for _ in 0..n {
            let (x1, a1, b1) = tortoise.next().unwrap();
            let _ = hare.next();
            let (x2, a2, b2) = hare.next().unwrap();

            if x1 == x2 {
                if b1 == b2 {
                    break;
                }
                let x = (a1 - a2) * (b2 - b1).modinv(&DEFAULTEC.n).unwrap();
                return Ok(x.modpow(&BigInt::one(), &DEFAULTEC.n));
            }
        }
    }

    Err(anyhow!("pollards_rho error."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_phollard_rho() {
        let w = BigInt::from(5);
        let g_w = Point::default() * &w;

        let now = Instant::now();
        let value = pollards_rho(&Point::default(), &g_w).unwrap();
        println!("Phollar_rho time: {:.2?}", now.elapsed());
        assert_eq!(value, w)
    }
}
