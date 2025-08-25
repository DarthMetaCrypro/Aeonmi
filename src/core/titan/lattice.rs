// Simple (integer) LLL reduction (δ in (0.5, 1)), educational version.
// Basis is row vectors. Returns a reduced basis.
pub fn lll_reduce(mut b: Vec<Vec<i64>>, delta: f64) -> Vec<Vec<i64>> {
    assert!(delta > 0.5 && delta < 1.0, "delta must be in (0.5,1)");
    let n = b.len();
    assert!(n > 0);
    let m = b[0].len();
    assert!(b.iter().all(|r| r.len()==m));

    // Gram–Schmidt (floating) helpers
    let mut mu = vec![vec![0.0; n]; n];
    let mut b_star = vec![vec![0.0; m]; n];
    let mut b_star_norm2 = vec![0.0; n];

    let recompute = |b: &Vec<Vec<i64>>, mu: &mut Vec<Vec<f64>>, b_star: &mut Vec<Vec<f64>>, bsn2: &mut Vec<f64>| {
        let n = b.len(); let m = b[0].len();
        for i in 0..n {
            for j in 0..m { b_star[i][j] = b[i][j] as f64; }
            for j in 0..i {
                // mu[i][j] = <b_i, b*_j> / <b*_j, b*_j>
                let mut num = 0.0;
                for t in 0..m { num += (b[i][t] as f64) * b_star[j][t]; }
                mu[i][j] = if bsn2[j] == 0.0 { 0.0 } else { num / bsn2[j] };
                for t in 0..m { b_star[i][t] -= mu[i][j] * b_star[j][t]; }
            }
            bsn2[i] = b_star[i].iter().map(|x| x*x).sum::<f64>();
        }
    };

    recompute(&b, &mut mu, &mut b_star, &mut b_star_norm2);

    let mut k = 1usize;
    while k < n {
        // size reduction: for j = k-1..0
        for j in (0..k).rev() {
            let r = mu[k][j].round(); // nearest integer
            if r != 0.0 {
                for t in 0..m { b[k][t] -= (r as i64) * b[j][t]; }
            }
        }
        recompute(&b, &mut mu, &mut b_star, &mut b_star_norm2);

        // Lovász condition
        if b_star_norm2[k] + mu[k][k-1].powi(2) * b_star_norm2[k-1] >= delta * b_star_norm2[k-1] {
            k += 1;
        } else {
            // swap b_k and b_{k-1}
            b.swap(k, k-1);
            recompute(&b, &mut mu, &mut b_star, &mut b_star_norm2);
            if k > 1 { k -= 1; }
        }
    }
    b
}
