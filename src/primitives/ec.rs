use num_bigint::BigInt;
use num_traits::{One, Zero};
use std::ops::{Add, Neg};

pub struct EC {
    a: BigInt,
    b: BigInt,
    p: BigInt,
    n: BigInt,
}

impl Default for EC {
    fn default() -> Self {
        let a = BigInt::from(u64::from_str_radix("0c1e151a", 16).unwrap());
        let b = BigInt::from(u64::from_str_radix("79006aaa", 16).unwrap());
        let p = BigInt::from(u64::from_str_radix("a44d466b", 16).unwrap());
        let n = BigInt::from(u64::from_str_radix("a44ed353", 16).unwrap());

        EC { a, b, p, n }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Point {
    x: Option<BigInt>,
    y: Option<BigInt>,
}

impl Point {
    fn is_none(&self) -> bool {
        self.x.is_none()
    }

    fn infinity() -> Point {
        Point { x: None, y: None }
    }
}

impl EC {
    pub fn generator() -> Point {
        let x = BigInt::from(u64::from_str_radix("6375e646", 16).unwrap());
        let y = BigInt::from(u64::from_str_radix("16389b96", 16).unwrap());
        Point {
            x: Some(x),
            y: Some(y),
        }
    }

    pub fn neg(&self, p: &Point) -> Point {
        if p.is_none() {
            return Point::infinity();
        }

        Point {
            x: p.x.clone(),
            y: p.y.clone().map(|y| y.neg().modpow(&BigInt::one(), &self.p)),
        }
    }

    pub fn double(&self, p: Point) -> Point {
        if p.is_none() {
            return p;
        }

        let p_x = p.x.unwrap();
        let p_y = p.y.unwrap();

        if p_y.is_zero() {
            return Point::infinity();
        }

        // (3 * pow(p[0], 2, self.p) + self.a) * gmpy2.invert(2 * p[1], self.p) % self.p
        let l_1 = (BigInt::from(3) * p_x.modpow(&BigInt::from(2), &self.p)).add(&self.a);
        let l_2 = (BigInt::from(2) * &p_y).modinv(&self.p).unwrap();
        let l = l_1 * l_2;

        // x3 = (pow(l, 2, self.p) - 2 * p[0]) % self.p
        // y3 = (l * (p[0] - x3) - p[1]) % self.p

        let x3 = l.modpow(&BigInt::from(2), &self.p) - (BigInt::from(2) * &p_x);
        let y3 = (l * (p_x - &x3) - p_y).modpow(&BigInt::one(), &self.p);

        Point {
            x: Some(x3),
            y: Some(y3),
        }
    }

    pub fn add(&self, p1: Point, p2: Point) -> Point {
        if p1.eq(&p2) {
            return self.double(p1.clone());
        }

        if p1.is_none() {
            return p2.clone();
        }
        if p2.is_none() {
            return p1.clone();
        }
        if p2.eq(&self.neg(&p1)) {
            return Point::infinity();
        }

        let p1_x = p1.x.unwrap();
        let p1_y = p1.y.unwrap();
        let p2_x = p2.x.unwrap();
        let p2_y = p2.y.unwrap();

        // l = (p2[1] - p1[1]) * gmpy2.invert(p2[0] - p1[0], self.p) % self.p
        let l = ((p2_y - &p1_y) * (&p2_x - &p1_x).modinv(&self.p).unwrap())
            .modpow(&BigInt::one(), &self.p);

        // x3 = (pow(l, 2, self.p) - p1[0] - p2[0]) % self.p
        let x3 =
            (l.modpow(&BigInt::from(2), &self.p) - &p1_x - p2_x).modpow(&BigInt::one(), &self.p);

        // y3 = (l * (p1[0] - x3) - p1[1]) % self.p
        let y3 = (l * (p1_x - &x3) - p1_y).modpow(&BigInt::one(), &self.p);

        Point {
            x: Some(x3),
            y: Some(y3),
        }
    }

    pub fn mul(&self, p: Point, scalar: &BigInt) -> Point {
        if p.is_none() {
            return p;
        }

        let mut res = Point::infinity();
        let mut temp = p;

        let mut scalar = scalar.clone();

        while scalar > BigInt::zero() {
            let bit = &scalar % 2;
            scalar >>= 1;
            if bit == BigInt::one() {
                res = self.add(res, temp.clone());
            }
            temp = self.double(temp);
        }

        res
    }
}
