// --- 26) SHA-256 single compression round (one i) ---
// Inputs are the working vars (a..h), message word w_i, constant k_i.
// Returns updated (a..h). See FIPS 180-4 for full schedule/loop.
#[inline(always)] fn rotr(x: u32, n: u32) -> u32 { (x >> n) | (x << (32 - n)) }
#[inline(always)] fn ch(x:u32,y:u32,z:u32)->u32{ (x & y) ^ (!x & z) }
#[inline(always)] fn maj(x:u32,y:u32,z:u32)->u32{ (x & y) ^ (x & z) ^ (y & z) }
#[inline(always)] fn big_sigma0(x:u32)->u32{ rotr(x,2) ^ rotr(x,13) ^ rotr(x,22) }
#[inline(always)] fn big_sigma1(x:u32)->u32{ rotr(x,6) ^ rotr(x,11) ^ rotr(x,25) }

pub fn sha256_round(
    a:u32,b:u32,c:u32,d:u32,e:u32,f:u32,g:u32,h:u32,
    w_i:u32, k_i:u32
)->(u32,u32,u32,u32,u32,u32,u32,u32){
    let t1 = h
        .wrapping_add(big_sigma1(e))
        .wrapping_add(ch(e,f,g))
        .wrapping_add(k_i)
        .wrapping_add(w_i);
    let t2 = big_sigma0(a).wrapping_add(maj(a,b,c));

    let new_h = g;
    let new_g = f;
    let new_f = e;
    let new_e = d.wrapping_add(t1);
    let new_d = c;
    let new_c = b;
    let new_b = a;
    let new_a = t1.wrapping_add(t2);
    (new_a,new_b,new_c,new_d,new_e,new_f,new_g,new_h)
}
