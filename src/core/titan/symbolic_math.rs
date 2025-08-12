use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Expr {
    Constant(f64),
    Variable(String),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
}

impl Expr {
    pub fn evaluate(&self, variables: &HashMap<String, f64>) -> f64 {
        match self {
            Expr::Constant(value) => *value,
            Expr::Variable(name) => *variables
                .get(name)
                .expect(&format!("Variable '{}' not found.", name)),
            Expr::Add(lhs, rhs) => lhs.evaluate(variables) + rhs.evaluate(variables),
            Expr::Sub(lhs, rhs) => lhs.evaluate(variables) - rhs.evaluate(variables),
            Expr::Mul(lhs, rhs) => lhs.evaluate(variables) * rhs.evaluate(variables),
            Expr::Div(lhs, rhs) => lhs.evaluate(variables) / rhs.evaluate(variables),
            Expr::Pow(base, exponent) => base.evaluate(variables).powf(exponent.evaluate(variables)),
        }
    }

    pub fn differentiate(&self, var: &str) -> Expr {
        match self {
            Expr::Constant(_) => Expr::Constant(0.0),
            Expr::Variable(name) => {
                if name == var {
                    Expr::Constant(1.0)
                } else {
                    Expr::Constant(0.0)
                }
            }
            Expr::Add(lhs, rhs) => Expr::Add(
                Box::new(lhs.differentiate(var)),
                Box::new(rhs.differentiate(var)),
            ),
            Expr::Sub(lhs, rhs) => Expr::Sub(
                Box::new(lhs.differentiate(var)),
                Box::new(rhs.differentiate(var)),
            ),
            Expr::Mul(lhs, rhs) => Expr::Add(
                Box::new(Expr::Mul((*lhs).clone(), Box::new((*rhs).differentiate(var)))),
                Box::new(Expr::Mul((*rhs).clone(), Box::new((*lhs).differentiate(var)))),
            ),
            Expr::Div(lhs, rhs) => Expr::Div(
                Box::new(Expr::Sub(
                    Box::new(Expr::Mul(Box::new((*lhs).differentiate(var)), (*rhs).clone())),
                    Box::new(Expr::Mul(Box::new((*rhs).differentiate(var)), (*lhs).clone())),
                )),
                Box::new(Expr::Mul((*rhs).clone(), (*rhs).clone())),
            ),
            Expr::Pow(base, exponent) => Expr::Mul(
                Box::new(Expr::Mul(
                    Box::new(*(*exponent).clone()),
                    Box::new(Expr::Pow(
                        (*base).clone(),
                        Box::new(Expr::Sub(
                            Box::new(*(*exponent).clone()),
                            Box::new(Expr::Constant(1.0))
                        ))
                    )),
                )),
                Box::new((*base).differentiate(var)),
            ),
        }
    }
}

pub fn simplify(expr: Expr) -> Expr {
    match expr {
        Expr::Add(lhs, rhs) => match (*lhs, *rhs) {
            (Expr::Constant(a), Expr::Constant(b)) => Expr::Constant(a + b),
            (l, r) => Expr::Add(Box::new(l), Box::new(r)),
        },
        Expr::Sub(lhs, rhs) => match (*lhs, *rhs) {
            (Expr::Constant(a), Expr::Constant(b)) => Expr::Constant(a - b),
            (l, r) => Expr::Sub(Box::new(l), Box::new(r)),
        },
        Expr::Mul(lhs, rhs) => match (*lhs, *rhs) {
            (Expr::Constant(a), Expr::Constant(b)) => Expr::Constant(a * b),
            (l, r) => Expr::Mul(Box::new(l), Box::new(r)),
        },
        Expr::Div(lhs, rhs) => match (*lhs, *rhs) {
            (Expr::Constant(a), Expr::Constant(b)) => Expr::Constant(a / b),
            (l, r) => Expr::Div(Box::new(l), Box::new(r)),
        },
        Expr::Pow(base, exponent) => match (*base, *exponent) {
            (Expr::Constant(a), Expr::Constant(b)) => Expr::Constant(a.powf(b)),
            (b, e) => Expr::Pow(Box::new(b), Box::new(e)),
        },
        _ => expr,
    }
}
