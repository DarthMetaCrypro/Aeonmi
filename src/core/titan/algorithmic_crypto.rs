// src/core/titan/algorithmic_crypto.rs
// Minimal, readable primitives to back the “equation” forms.
// NOTE: this is educational-grade; swap for audited libs in production.

use num_bigint::{BigInt, BigUint, ToBigInt};
use num_traits::{One, Zero};
use sha2::{Digest, Sha256};

// ---------- 21) RSA ENCRYPT: c = m^e mod n ----------
pub fn rsa_encrypt(m: &BigUint, e: &BigUint, n: &BigUint) -> BigUint {
    m.modpow(e, n)
}

// ---------- 22) RSA DECRYPT: m = c^d mod n ----------
pub fn rsa_decrypt(c: &BigUint, d: &BigUint, n: &BigUint) -> BigUint {
    c.modpow(d, n)
}

// ---------- 23) DIFFIE–HELLMAN SHARED: K = g^{ab} mod p ----------
// Given a's secret and B’s public (g^b mod p), compute K = (B_pub)^a mod p
pub fn dh_shared_from_other_pub(other_pub: &BigUint, a_secret: &BigUint, p: &BigUint) -> BigUint {
    other_pub.modpow(a_secret, p)
}

// ---------- 24) ELLIPTIC CURVE (short Weierstrass over prime field) ----------
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ECPoint {
    pub x: BigUint,
    pub y: BigUint,
    pub infinity: bool,
}

#[derive(Clone, Debug)]
pub struct EllipticCurve {
    pub a: BigUint, // y^2 = x^3 + a x + b (mod p)
    pub b: BigUint,
    pub p: BigUint, // prime
}

impl EllipticCurve {
    pub fn point_at_infinity(&self) -> ECPoint {
        ECPoint { x: BigUint::zero(), y: BigUint::zero(), infinity: true }
    }
}

fn mod_add(a: &BigUint, b: &BigUint, p: &BigUint) -> BigUint {
    (a + b) % p
}
fn mod_sub(a: &BigUint, b: &BigUint, p: &BigUint) -> BigUint {
    ((a + p) - b) % p
}
fn mod_mul(a: &BigUint, b: &BigUint, p: &BigUint) -> BigUint {
    (a * b) % p
}
fn mod_inv(a: &BigUint, p: &BigUint) -> Option<BigUint> {
    // Extended Euclid on BigInt, then normalize to [0, p)
    let (g, x, _) = egcd(&a.to_bigint().unwrap(), &p.to_bigint().unwrap());
    if g != BigInt::one() { return None; }
    let mut x = x % p.to_bigint().unwrap();
    if x < BigInt::zero() { x += p.to_bigint().unwrap(); }
    Some(x.to_biguint().unwrap())
}

fn egcd(a: &BigInt, b: &BigInt) -> (BigInt, BigInt, BigInt) {
    if b.is_zero() { (a.clone(), BigInt::one(), BigInt::zero()) } else {
        let (g, x1, y1) = egcd(b, &(a % b));
        (g, y1.clone(), x1 - (a / b) * y1)
    }
}

// Point addition: R = P + Q
pub fn ec_add(curve: &EllipticCurve, p1: &ECPoint, p2: &ECPoint) -> ECPoint {
    if p1.infinity { return p2.clone(); }
    if p2.infinity { return p1.clone(); }

    let p = &curve.p;

    // If x1 == x2 and y1 == -y2 (mod p), return infinity
    if p1.x == p2.x && (p1.y + &p2.y) % p == BigUint::zero() {
        return curve.point_at_infinity();
    }

    // slope λ
    let lambda = if p1 == p2 {
        // λ = (3x1^2 + a) / (2y1) mod p
        let three = BigUint::from(3u32);
        let two = BigUint::from(2u32);
        let num = mod_add(&mod_mul(&three, &mod_mul(&p1.x, &p1.x, p), p), &curve.a, p);
        let den = mod_mul(&two, &p1.y, p);
        let inv = mod_inv(&den, p).expect("no inverse for 2y1");
        mod_mul(&num, &inv, p)
    } else {
        // λ = (y2 - y1) / (x2 - x1) mod p
        let num = mod_sub(&p2.y, &p1.y, p);
        let den = mod_sub(&p2.x, &p1.x, p);
        let inv = mod_inv(&den, p).expect("no inverse for x2-x1");
        mod_mul(&num, &inv, p)
    };

    // x3 = λ^2 - x1 - x2 ; y3 = λ(x1 - x3) - y1
    let x3 = {
        let l2 = mod_mul(&lambda, &lambda, p);
        let tmp = mod_sub(&mod_sub(&l2, &p1.x, p), &p2.x, p);
        tmp
    };
    let y3 = {
        let x1_minus_x3 = mod_sub(&p1.x, &x3, p);
        let t = mod_mul(&lambda, &x1_minus_x3, p);
        mod_sub(&t, &p1.y, p)
    };

    ECPoint { x: x3, y: y3, infinity: false }
}

// Scalar multiply (double-and-add), for completeness
pub fn ec_scalar_mul(curve: &EllipticCurve, k: &BigUint, p: &ECPoint) -> ECPoint {
    let mut res = curve.point_at_infinity();
    let mut addend = p.clone();
    let mut k_bits = k.clone();

    while k_bits > BigUint::zero() {
        if (&k_bits & BigUint::one()) == BigUint::one() {
            res = ec_add(curve, &res, &addend);
        }
        addend = ec_add(curve, &addend, &addend);
        k_bits >>= 1;
    }
    res
}

// ---------- 25) HASH “COMPRESSION” STEP: H_i = f(H_{i-1}, M_i) ----------
// Simple Davies–Meyer-style wrapper using SHA-256 as f:
// H_i = SHA256( H_{i-1} || M_i )  (returns 32-byte state)
pub fn dm_sha256_compress(h_prev: &[u8; 32], m_block: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(h_prev);
    hasher.update(m_block);
    let out = hasher.finalize();
    let mut h = [0u8; 32];
    h.copy_from_slice(&out);
    h
}
