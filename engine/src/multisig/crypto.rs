// We want to re-export certain types here
// to make sure all of our dependencies on
// this module are in one place
use curv::elliptic::curves::{
    secp256_k1::{Secp256k1Point, Secp256k1Scalar},
    ECScalar,
};

pub use curv::{
    arithmetic::traits::Converter as BigIntConverter, elliptic::curves::ECPoint, BigInt,
};

#[derive(Clone, Copy, Debug, PartialEq, Zeroize)]
pub struct Point(pub Secp256k1Point);

#[derive(Clone, Debug, PartialEq)]
pub struct Scalar(Secp256k1Scalar);

impl zeroize::Zeroize for Scalar {
    fn zeroize(&mut self) {
        // Secp256k1Scalar doesn't expose a way to "zeroize" it apart from dropping, so have
        // to do it manually (I think assigning a different value would be sufficient to drop
        // and zeroize the value, but we are not 100% sure that it won't get optimised away).
        use core::sync::atomic;
        unsafe { std::ptr::write_volatile(&mut self.0, Secp256k1Scalar::zero()) };
        atomic::compiler_fence(atomic::Ordering::SeqCst);
    }
}

impl<'de> Deserialize<'de> for Point {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = Vec::deserialize(deserializer)?;

        Secp256k1Point::deserialize(&bytes)
            .map(|x| Point(x))
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for Point {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.0.serialize_compressed().as_ref())
    }
}

impl std::iter::Sum for Point {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Point(Secp256k1Point::zero()), |a, b| a + b)
    }
}

impl<'de> Deserialize<'de> for Scalar {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = Vec::deserialize(deserializer)?;

        Secp256k1Scalar::deserialize(&bytes)
            .map(|x| Scalar(x))
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for Scalar {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.0.serialize().as_ref())
    }
}

impl std::iter::Sum for Scalar {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Scalar(Secp256k1Scalar::zero()), |a, b| a + b)
    }
}

use generic_array::GenericArray;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyShare {
    pub y: Point,
    pub x_i: Scalar,
}

#[cfg(test)]
impl Point {
    pub fn random() -> Self {
        Point::from_scalar(&Scalar::random())
    }
}

impl Point {
    pub fn from_scalar(scalar: &Scalar) -> Self {
        Point(Secp256k1Point::generator().scalar_mul(&scalar.0))
    }

    pub fn get_element(&self) -> secp256k1::PublicKey {
        // TODO: ensure that we don't create points at infinity
        // (we might want to sanitize p2p data)
        self.0
            .underlying_ref()
            .expect("unexpected point at infinity")
            .0
    }

    pub fn as_bytes(&self) -> GenericArray<u8, <Secp256k1Point as ECPoint>::CompressedPointLength> {
        self.0.serialize_compressed()
    }
}

impl Scalar {
    pub fn random() -> Self {
        Scalar(Secp256k1Scalar::random())
    }

    pub fn zero() -> Self {
        Scalar(Secp256k1Scalar::zero())
    }

    pub fn from_usize(a: usize) -> Self {
        Scalar(ECScalar::from_bigint(&BigInt::from(a as u32)))
    }

    pub fn from_bytes(x: &[u8; 32]) -> Self {
        Scalar(ECScalar::from_bigint(&BigInt::from_bytes(x)))
    }

    #[cfg(test)]
    pub fn from_hex(sk_hex: &str) -> Self {
        let bytes = hex::decode(sk_hex).expect("input must be hex encoded");

        Scalar(Secp256k1Scalar::deserialize(&bytes).expect("input must represent a scalar"))
    }

    pub fn invert(&self) -> Option<Self> {
        self.0.invert().map(|x| Scalar(x))
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        match self.0.underlying_ref() {
            Some(secret_key) => secret_key.as_ref(),
            // None represents "zero" scalar in `curv`
            None => &[0; 32],
        }
    }
}

// TODO: Look at how to dedup these adds
impl std::ops::Add for &Scalar {
    type Output = Scalar;

    fn add(self, rhs: Self) -> Self::Output {
        Scalar(self.0.add(&rhs.0))
    }
}

impl std::ops::Add for Scalar {
    type Output = Scalar;

    fn add(self, rhs: Self) -> Self::Output {
        <&Scalar>::add(&self, &rhs)
    }
}

impl std::ops::Mul for &Scalar {
    type Output = Scalar;

    fn mul(self, rhs: Self) -> Self::Output {
        Scalar(self.0.mul(&rhs.0))
    }
}

impl std::ops::Mul for Scalar {
    type Output = Scalar;

    fn mul(self, rhs: Self) -> Self::Output {
        &self * &rhs
    }
}

impl std::ops::Mul<&Scalar> for Scalar {
    type Output = Scalar;

    fn mul(self, rhs: &Scalar) -> Self::Output {
        &self * rhs
    }
}

impl std::ops::Sub for &Scalar {
    type Output = Scalar;

    fn sub(self, rhs: Self) -> Self::Output {
        Scalar(self.0.sub(&rhs.0))
    }
}

impl std::ops::Sub for Scalar {
    type Output = Scalar;

    fn sub(self, rhs: Self) -> Self::Output {
        &self - &rhs
    }
}

impl std::ops::Mul<Scalar> for Point {
    type Output = Point;

    fn mul(self, rhs: Scalar) -> Self::Output {
        Point(self.0.scalar_mul(&rhs.0))
    }
}

impl std::ops::Mul<&Scalar> for Point {
    type Output = Point;

    fn mul(self, rhs: &Scalar) -> Self::Output {
        Point(self.0.scalar_mul(&rhs.0))
    }
}

impl std::ops::Mul<&Scalar> for &Point {
    type Output = Point;

    fn mul(self, rhs: &Scalar) -> Self::Output {
        Point(self.0.scalar_mul(&rhs.0))
    }
}

// TODO: Look at how to dedup these adds
// (See above impl Add for Scalar too)
impl std::ops::Add for &Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Point(self.0.add_point(&rhs.0))
    }
}

impl std::ops::Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        <&Point>::add(&self, &rhs)
    }
}

impl std::ops::Sub for Point {
    type Output = Point;

    fn sub(self, rhs: Self) -> Self::Output {
        Point(self.0.sub_point(&rhs.0))
    }
}

impl std::ops::Sub<Point> for &Point {
    type Output = Point;

    fn sub(self, rhs: Point) -> Self::Output {
        *self - rhs
    }
}
