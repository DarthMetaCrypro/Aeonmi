// --- 28) Paillier-form homomorphism helpers (ciphertext domain) ---
// Given modulus n^2, ciphertexts c1,c2:  Enc(m1)*Enc(m2) mod n^2 = Enc(m1+m2)
// And scalar multiply: Enc(m)^k mod n^2 = Enc(k*m)
use num_bigint::BigUint;

pub fn paillier_homo_add(c1: &BigUint, c2: &BigUint, n_sq: &BigUint) -> BigUint {
    (c1 * c2) % n_sq
}

pub fn paillier_scalar_mul(c: &BigUint, k: &BigUint, n_sq: &BigUint) -> BigUint {
    c.modpow(k, n_sq)
}
