// src/core/titan/algorithmic/glyph.rs

use std::collections::HashMap;

pub enum GlyphOp {
    // Optimization starters — keep adding as you wire more files
    GradDescent,   // "∇GD"
    Newton,        // "NR"
    ConjugateGrad, // "CG"
    AnnealAccept,  // "SAp"
    Xover1P,       // "X1P"
}

impl GlyphOp {
    pub fn parse(tag: &str) -> Option<Self> {
        match tag {
            "∇GD" | "GD" => Some(Self::GradDescent),
            "NR"         => Some(Self::Newton),
            "CG"         => Some(Self::ConjugateGrad),
            "SAp"        => Some(Self::AnnealAccept),
            "X1P"        => Some(Self::Xover1P),
            _ => None,
        }
    }
}

/// Uniform call signature so glyphs can dispatch into any math file.
/// Keep it dead-simple: inputs as slices/args; return Vec for generality.
pub trait GlyphExec {
    fn call(&self, args: &GlyphArgs) -> Result<Vec<f64>, String>;
}

#[derive(Clone, Debug)]
pub struct GlyphArgs {
    pub scalars: HashMap<&'static str, f64>,
    pub v1: Vec<f64>,
    pub v2: Vec<f64>,
    pub idx: Option<usize>,
}

impl GlyphArgs {
    pub fn new() -> Self {
        Self { scalars: HashMap::new(), v1: vec![], v2: vec![], idx: None }
    }
    pub fn with_scalar(mut self, k: &'static str, v: f64) -> Self { self.scalars.insert(k, v); self }
    pub fn with_v1(mut self, v: Vec<f64>) -> Self { self.v1 = v; self }
    pub fn with_v2(mut self, v: Vec<f64>) -> Self { self.v2 = v; self }
    pub fn with_idx(mut self, i: usize) -> Self { self.idx = Some(i); self }
}

/// Concrete execs that wrap your optimization functions
pub struct ExecGradDescent;
pub struct ExecNewton;
pub struct ExecConjGrad;
pub struct ExecAnneal;
pub struct ExecXover1P;

impl GlyphExec for ExecGradDescent {
    // expects: theta, grad, lr  -> returns {theta_next}
    fn call(&self, a: &GlyphArgs) -> Result<Vec<f64>, String> {
        use crate::core::titan::algorithmic::optimization::gradient_descent_update;
        let theta = *a.scalars.get("theta").ok_or("missing theta")?;
        let grad  = *a.scalars.get("grad").ok_or("missing grad")?;
        let lr    = *a.scalars.get("lr").ok_or("missing lr")?;
        Ok(vec![gradient_descent_update(theta, grad, lr)])
    }
}

impl GlyphExec for ExecNewton {
    // expects: x, f, fp -> returns {x_next}
    fn call(&self, a: &GlyphArgs) -> Result<Vec<f64>, String> {
        use crate::core::titan::algorithmic::optimization::newton_raphson_step;
        let x  = *a.scalars.get("x").ok_or("missing x")?;
        let f  = *a.scalars.get("f").ok_or("missing f")?;
        let fp = *a.scalars.get("fp").ok_or("missing fp")?;
        let xn = newton_raphson_step(x, f, fp).map_err(|e| e.to_string())?;
        Ok(vec![xn])
    }
}

impl GlyphExec for ExecConjGrad {
    // expects: x_k (v1), p_k (v2), alpha -> returns {x_next...}
    fn call(&self, a: &GlyphArgs) -> Result<Vec<f64>, String> {
        use crate::core::titan::algorithmic::optimization::conjugate_gradient_step;
        let alpha = *a.scalars.get("alpha").ok_or("missing alpha")?;
        Ok(conjugate_gradient_step(a.v1.clone(), alpha, a.v2.clone()))
    }
}

impl GlyphExec for ExecAnneal {
    // expects: dE, k, T -> returns {P}
    fn call(&self, a: &GlyphArgs) -> Result<Vec<f64>, String> {
        use crate::core::titan::algorithmic::optimization::simulated_annealing_acceptance;
        let de = *a.scalars.get("dE").ok_or("missing dE")?;
        let k  = *a.scalars.get("k").ok_or("missing k")?;
        let t  = *a.scalars.get("T").ok_or("missing T")?;
        Ok(vec![simulated_annealing_acceptance(de, k, t)])
    }
}

impl GlyphExec for ExecXover1P {
    // expects: parent1 (v1), parent2 (v2), idx -> returns {child1..., child2...}
    fn call(&self, a: &GlyphArgs) -> Result<Vec<f64>, String> {
        use crate::core::titan::algorithmic::optimization::one_point_crossover;
        let idx = a.idx.ok_or("missing crossover idx")?;
        let (c1, c2) = one_point_crossover(&a.v1, &a.v2, idx);
        let mut out = c1; out.extend(c2);
        Ok(out)
    }
}

/// Simple in-memory registry (swap to static once the list stabilizes)
pub fn registry() -> HashMap<GlyphOp, Box<dyn GlyphExec + Send + Sync>> {
    use GlyphOp::*;
    let mut r: HashMap<GlyphOp, Box<dyn GlyphExec + Send + Sync>> = HashMap::new();
    r.insert(GradDescent,   Box::new(ExecGradDescent));
    r.insert(Newton,        Box::new(ExecNewton));
    r.insert(ConjugateGrad, Box::new(ExecConjGrad));
    r.insert(AnnealAccept,  Box::new(ExecAnneal));
    r.insert(Xover1P,       Box::new(ExecXover1P));
    r
}

/// One-shot dispatcher: pass a glyph tag + args, get result vector.
pub fn dispatch(tag: &str, args: &GlyphArgs) -> Result<Vec<f64>, String> {
    let op = GlyphOp::parse(tag).ok_or_else(|| format!("unknown glyph '{tag}'"))?;
    let reg = registry();
    reg.get(&op).ok_or("unregistered glyph".to_string())?.call(args)
}
