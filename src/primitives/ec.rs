use lazy_static::lazy_static;
use num_bigint::BigInt;
use num_traits::{One, Zero};
use std::ops::{Add, Mul, Neg};

pub struct EC {
    pub a: BigInt,
    pub b: BigInt,
    pub p: BigInt,
    pub n: BigInt,
}

lazy_static! {
    pub static ref DEFAULTEC: EC = EC {
        a: BigInt::from(203298074u64),
        b: BigInt::from(2030070442u64),
        p: BigInt::from(2756527723u64),
        n: BigInt::from(2756629331u64),
    };
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Point {
    pub x: Option<BigInt>,
    pub y: Option<BigInt>,
}

impl Default for Point {
    fn default() -> Self {
        let x = BigInt::from(1668671046u64);
        let y = BigInt::from(372808598u64);
        Self {
            x: Some(x),
            y: Some(y),
        }
    }
}

impl Point {
    pub const fn is_none(&self) -> bool {
        self.x.is_none()
    }

    pub const fn infinity() -> Self {
        Self { x: None, y: None }
    }

    pub fn double(&self) -> Self {
        if self.is_none() {
            return self.clone();
        }

        let p_x = self.x.clone().unwrap();
        let p_y = self.y.clone().unwrap();

        if p_y.is_zero() {
            return Self::infinity();
        }

        // (3 * pow(p[0], 2, self.p) + self.a) * gmpy2.invert(2 * p[1], self.p) % self.p
        let l_1 = BigInt::from(3) * p_x.modpow(&BigInt::from(2), &DEFAULTEC.p) + &DEFAULTEC.a;
        let l_2 = ((BigInt::from(2) * &p_y).modinv(&DEFAULTEC.p).unwrap())
            .modpow(&BigInt::one(), &DEFAULTEC.p);
        let l = l_1 * l_2;

        // x3 = (pow(l, 2, self.p) - 2 * p[0]) % self.p
        // y3 = (l * (p[0] - x3) - p[1]) % self.p
        let x3 = (l.modpow(&BigInt::from(2), &DEFAULTEC.p) - (BigInt::from(2) * &p_x))
            .modpow(&BigInt::one(), &DEFAULTEC.p);
        let y3 = (l * (p_x - &x3) - p_y).modpow(&BigInt::one(), &DEFAULTEC.p);

        Self {
            x: Some(x3),
            y: Some(y3),
        }
    }
}

impl Neg for Point {
    type Output = Self;

    fn neg(self) -> Self::Output {
        if self.is_none() {
            return Self::infinity();
        }

        Self {
            x: self.x.clone(),
            y: self.y.map(|y| y.neg().modpow(&BigInt::one(), &DEFAULTEC.p)),
        }
    }
}

impl Add for &Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        if self.eq(rhs) {
            return self.double();
        }

        if self.is_none() {
            return rhs.clone();
        }
        if rhs.is_none() {
            return self.clone();
        }
        if *rhs == -self.clone() {
            return Point::infinity();
        }

        let p1_x = self.x.clone().unwrap();
        let p1_y = self.y.clone().unwrap();
        let p2_x = rhs.x.clone().unwrap();
        let p2_y = rhs.y.clone().unwrap();

        // l = (p2[1] - p1[1]) * gmpy2.invert(p2[0] - p1[0], self.p) % self.p
        let l = ((p2_y - &p1_y) * (&p2_x - &p1_x).modinv(&DEFAULTEC.p).unwrap())
            .modpow(&BigInt::one(), &DEFAULTEC.p);

        // x3 = (pow(l, 2, self.p) - p1[0] - p2[0]) % self.p
        let x3 = (l.modpow(&BigInt::from(2), &DEFAULTEC.p) - &p1_x - p2_x)
            .modpow(&BigInt::one(), &DEFAULTEC.p);

        // y3 = (l * (p1[0] - x3) - p1[1]) % self.p
        let y3 = (l * (p1_x - &x3) - p1_y).modpow(&BigInt::one(), &DEFAULTEC.p);

        Point {
            x: Some(x3),
            y: Some(y3),
        }
    }
}

impl Mul<&BigInt> for Point {
    type Output = Self;

    fn mul(self, rhs: &BigInt) -> Self::Output {
        if self.is_none() {
            return self;
        }

        let mut res = Self::infinity();
        let mut temp = self;

        let mut scalar = rhs.clone();

        while scalar > BigInt::zero() {
            let bit = &scalar % 2;
            scalar >>= 1;
            if bit == BigInt::one() {
                res = &res + &temp;
            }
            temp = temp.double();
        }

        res
    }
}
