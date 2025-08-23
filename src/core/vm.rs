#![cfg_attr(test, allow(dead_code, unused_variables))]
//! Aeonmi VM: tree-walk interpreter over IR.
//! Supports: literals, arrays/objects, let/assign, if/while/for, fn calls/returns,
//! basic binary/unary ops, and built-ins: print, log, time_ms, rand.

use crate::core::ir::*;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Function(Function), // user-defined
    Builtin(Builtin),
}

#[derive(Clone)]
pub struct Function {
    pub params: Vec<String>,
    pub body: Block,
    pub env: Env, // closure (shallow copy at def time)
}

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Function")
            .field("params", &self.params)
            .field("body_len", &self.body.stmts.len())
            .finish()
    }
}

#[derive(Clone)]
pub struct Builtin {
    pub name: &'static str,
    pub arity: usize, // use usize::MAX for variadic
    pub f: fn(&mut Interpreter, Vec<Value>) -> Result<Value, RuntimeError>,
}

impl std::fmt::Debug for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Builtin").field("name", &self.name).finish()
    }
}

#[derive(Clone, Debug)]
pub struct Env {
    frames: Vec<HashMap<String, Value>>,
}

impl Env {
    pub fn new() -> Self { Self { frames: vec![HashMap::new()] } }
    pub fn push(&mut self) { self.frames.push(HashMap::new()); }
    pub fn pop(&mut self) { self.frames.pop(); }
    pub fn define(&mut self, k: String, v: Value) { self.frames.last_mut().unwrap().insert(k, v); }

    pub fn assign(&mut self, k: &str, v: Value) -> bool {
        for frame in self.frames.iter_mut().rev() {
            if frame.contains_key(k) { frame.insert(k.to_string(), v); return true; }
        }
        false
    }

    pub fn get(&self, k: &str) -> Option<Value> {
        for frame in self.frames.iter().rev() {
            if let Some(v) = frame.get(k) { return Some(v.clone()); }
        }
        None
    }
}

#[derive(Debug)]
pub struct Interpreter {
    pub env: Env,
}

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut env = Env::new();
        // Builtins
        env.define("print".into(), Value::Builtin(Builtin { name: "print", arity: usize::MAX, f: builtin_print }));
        env.define("log".into(), Value::Builtin(Builtin { name: "log", arity: usize::MAX, f: builtin_print }));
        env.define("time_ms".into(), Value::Builtin(Builtin { name: "time_ms", arity: 0, f: builtin_time_ms }));
        env.define("rand".into(), Value::Builtin(Builtin { name: "rand", arity: 0, f: builtin_rand }));
        Self { env }
    }

    pub fn run_module(&mut self, m: &Module) -> Result<(), RuntimeError> {
        // Load top-level decls
        for d in &m.decls {
            match d {
                Decl::Const(c) => {
                    let v = self.eval_expr(&c.value)?;
                    self.env.define(c.name.clone(), v);
                }
                Decl::Let(l) => {
                    let v = if let Some(e) = &l.value { self.eval_expr(e)? } else { Value::Null };
                    self.env.define(l.name.clone(), v);
                }
                Decl::Fn(f) => {
                    let func = Value::Function(Function {
                        params: f.params.clone(),
                        body: f.body.clone(),
                        env: self.env.clone(),
                    });
                    self.env.define(f.name.clone(), func);
                }
            }
        }
        // If there is a `main` fn with zero params, run it.
        if let Some(Value::Function(_)) = self.env.get("main") {
            let _ = self.call_ident("main", vec![])?;
        }
        Ok(())
    }

    fn call_ident(&mut self, name: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let callee = self.env.get(name).ok_or_else(|| err(format!("Undefined function `{}`", name)))?;
        self.call_value(callee, args)
    }

    fn call_value(&mut self, callee: Value, args: Vec<Value>) -> Result<Value, RuntimeError> {
        match callee {
            Value::Builtin(b) => {
                if b.arity != usize::MAX && b.arity != args.len() {
                    return Err(err(format!("builtin `{}` expected {} args, got {}", b.name, b.arity, args.len())));
                }
                (b.f)(self, args)
            }
            Value::Function(fun) => {
                if fun.params.len() != args.len() {
                    return Err(err(format!("function expected {} args, got {}", fun.params.len(), args.len())));
                }
                // New scope with closure base
                let saved = self.env.clone();
                self.env = fun.env.clone();
                self.env.push();
                for (p, v) in fun.params.iter().zip(args.into_iter()) {
                    self.env.define(p.clone(), v);
                }
                // Execute
                let ret = self.exec_block(&fun.body);
                // Restore
                let out = match ret {
                    ControlFlow::Ok => Ok(Value::Null),
                    ControlFlow::Return(v) => Ok(v.unwrap_or(Value::Null)),
                    ControlFlow::Err(e) => Err(e),
                };
                self.env = saved;
                out
            }
            other => Err(err(format!("callee is not callable: {:?}", other))),
        }
    }

    fn exec_block(&mut self, b: &Block) -> ControlFlow {
        self.env.push();
        for s in &b.stmts {
            match self.exec_stmt(s) {
                ControlFlow::Ok => {}
                other => {
                    self.env.pop();
                    return other;
                }
            }
        }
        self.env.pop();
        ControlFlow::Ok
    }

    fn exec_stmt(&mut self, s: &Stmt) -> ControlFlow {
        use Stmt::*;
        match s {
            Expr(e) => {
                if let Err(e) = self.eval_expr(e) { return ControlFlow::Err(e); }
                ControlFlow::Ok
            }
            Return(None) => ControlFlow::Return(None),
            Return(Some(e)) => {
                let v = match self.eval_expr(e) {
                    Ok(v) => v,
                    Err(e) => return ControlFlow::Err(e),
                };
                ControlFlow::Return(Some(v))
            }
            If { cond, then_block, else_block } => {
                let c = match self.eval_expr(cond) {
                    Ok(v) => self.truthy(&v),
                    Err(e) => return ControlFlow::Err(e),
                };
                if c {
                    self.exec_block(then_block)
                } else if let Some(e) = else_block {
                    self.exec_block(e)
                } else {
                    ControlFlow::Ok
                }
            }
            While { cond, body } => {
                loop {
                    let c = match self.eval_expr(cond) {
                        Ok(v) => self.truthy(&v),
                        Err(e) => return ControlFlow::Err(e),
                    };
                    if !c { break; }
                    match self.exec_block(body) {
                        ControlFlow::Ok => {}
                        other => return other,
                    }
                }
                ControlFlow::Ok
            }
            For { init, cond, step, body } => {
                if let Some(i) = init {
                    if let ControlFlow::Err(e) = self.exec_stmt(i) { return ControlFlow::Err(e); }
                }
                loop {
                    if let Some(c) = cond {
                        let ok = match self.eval_expr(c) {
                            Ok(v) => self.truthy(&v),
                            Err(e) => return ControlFlow::Err(e),
                        };
                        if !ok { break; }
                    }
                    match self.exec_block(body) {
                        ControlFlow::Ok => {}
                        other => return other,
                    }
                    if let Some(st) = step {
                        if let Err(e) = self.eval_expr(st) { return ControlFlow::Err(e); }
                    }
                }
                ControlFlow::Ok
            }
            Let { name, value } => {
                let v = if let Some(e) = value {
                    match self.eval_expr(e) {
                        Ok(v) => v,
                        Err(e) => return ControlFlow::Err(e),
                    }
                } else { Value::Null };
                self.env.define(name.clone(), v);
                ControlFlow::Ok
            }
            Assign { target, value } => {
                // Only Ident target in v0
                if let crate::core::ir::Expr::Ident(name) = target {
                    let v = match self.eval_expr(value) {
                        Ok(v) => v,
                        Err(e) => return ControlFlow::Err(e),
                    };
                    if !self.env.assign(name, v) {
                        return ControlFlow::Err(err(format!("Undefined variable `{}`", name)));
                    }
                    ControlFlow::Ok
                } else {
                    ControlFlow::Err(err("Only simple identifier assignment supported in v0".into()))
                }
            }
        }
    }

    fn eval_expr(&mut self, e: &Expr) -> Result<Value, RuntimeError> {
        use Expr::*;
        Ok(match e {
            Lit(l) => match l {
                crate::core::ir::Lit::Null   => Value::Null,
                crate::core::ir::Lit::Bool(b)   => Value::Bool(*b),
                crate::core::ir::Lit::Number(n) => Value::Number(*n),
                crate::core::ir::Lit::String(s) => Value::String(s.clone()),
            },
            Ident(s) => self.env.get(s)
                .ok_or_else(|| err(format!("Undefined identifier `{}`", s)))?,
            Call { callee, args } => {
                // Fast path: direct ident call (avoids allocating callee Value if builtin/func)
                if let Expr::Ident(name) = &**callee {
                    let argv = collect_vals(self, args)?;
                    self.call_ident(name, argv)?
                } else {
                    let callee_v = self.eval_expr(callee)?;
                    let argv = collect_vals(self, args)?;
                    self.call_value(callee_v, argv)?
                }
            }
            Unary { op, expr } => {
                let v = self.eval_expr(expr)?;
                match op {
                    UnOp::Neg => match v {
                        Value::Number(n) => Value::Number(-n),
                        other => return Err(err(format!("Unary `-` on non-number: {:?}", other))),
                    }
                    UnOp::Not => Value::Bool(!self.truthy(&v)),
                }
            }
            Binary { left, op, right } => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                self.eval_binop(op, l, r)?
            }
            Array(items) => {
                let mut out = Vec::with_capacity(items.len());
                for it in items { out.push(self.eval_expr(it)?); }
                Value::Array(out)
            }
            Object(kvs) => {
                let mut map = HashMap::with_capacity(kvs.len());
                for (k, v) in kvs { map.insert(k.clone(), self.eval_expr(v)?); }
                Value::Object(map)
            }
        })
    }

    fn eval_binop(&self, op: &BinOp, l: Value, r: Value) -> Result<Value, RuntimeError> {
        use BinOp::*;
        match op {
            Add => match (l, r) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                (Value::String(a), b) => Ok(Value::String(format!("{}{}", a, display(&b)))),
                (a, Value::String(b)) => Ok(Value::String(format!("{}{}", display(&a), b))),
                (a, b) => Err(err(format!("`+` on incompatible types: {:?}, {:?}", a, b))),
            }
            Sub => num2(l, r, |a,b| a-b),
            Mul => num2(l, r, |a,b| a*b),
            Div => num2(l, r, |a,b| a/b),
            Mod => num2(l, r, |a,b| a%b),
            Eq  => Ok(Value::Bool(eq_val(&l, &r))),
            Ne  => Ok(Value::Bool(!eq_val(&l, &r))),
            Lt  => cmp2(l, r, |a,b| a<b),
            Le  => cmp2(l, r, |a,b| a<=b),
            Gt  => cmp2(l, r, |a,b| a>b),
            Ge  => cmp2(l, r, |a,b| a>=b),
            And => Ok(Value::Bool(self.truthy(&l) && self.truthy(&r))),
            Or  => Ok(Value::Bool(self.truthy(&l) || self.truthy(&r))),
        }
    }

    fn truthy(&self, v: &Value) -> bool {
        match v {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Object(o) => !o.is_empty(),
            Value::Function(_) | Value::Builtin(_) => true,
        }
    }
}

enum ControlFlow {
    Ok,
    Return(Option<Value>),
    Err(RuntimeError),
}

impl From<RuntimeError> for ControlFlow {
    fn from(e: RuntimeError) -> Self { ControlFlow::Err(e) }
}

fn err(msg: String) -> RuntimeError { RuntimeError { message: msg } }

fn collect_vals(i: &mut Interpreter, es: &[Expr]) -> Result<Vec<Value>, RuntimeError> {
    let mut out = Vec::with_capacity(es.len());
    for e in es { out.push(i.eval_expr(e)?); }
    Ok(out)
}

fn num2(l: Value, r: Value, f: fn(f64, f64) -> f64) -> Result<Value, RuntimeError> {
    match (l, r) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(f(a,b))),
        (a, b) => Err(err(format!("numeric op on non-numbers: {:?}, {:?}", a, b))),
    }
}

fn cmp2(l: Value, r: Value, f: fn(f64, f64) -> bool) -> Result<Value, RuntimeError> {
    match (l, r) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(f(a,b))),
        (a, b) => Err(err(format!("comparison on non-numbers: {:?}, {:?}", a, b))),
    }
}

fn eq_val(a: &Value, b: &Value) -> bool {
    use Value::*;
    match (a, b) {
        (Null, Null) => true,
        (Bool(x), Bool(y)) => x == y,
        (Number(x), Number(y)) => x == y,
        (String(x), String(y)) => x == y,

        (Array(x), Array(y)) => {
            if x.len() != y.len() { return false; }
            for (lx, ry) in x.iter().zip(y.iter()) {
                if !eq_val(lx, ry) { return false; }
            }
            true
        }

        (Object(x), Object(y)) => {
            if x.len() != y.len() { return false; }
            for (k, vx) in x.iter() {
                match y.get(k) {
                    Some(vy) if eq_val(vx, vy) => {}
                    _ => return false,
                }
            }
            true
        }

        // Functions/builtins: not comparable for now
        (Function(_), Function(_)) => false,
        (Builtin(_), Builtin(_)) => false,

        _ => false,
    }
}

// ---------- Builtins ----------

fn builtin_print(_i: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    let parts: Vec<String> = args.iter().map(display).collect();
    println!("{}", parts.join(" "));
    Ok(Value::Null)
}

fn builtin_time_ms(_i: &mut Interpreter, _args: Vec<Value>) -> Result<Value, RuntimeError> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    Ok(Value::Number(now.as_millis() as f64))
}

fn builtin_rand(_i: &mut Interpreter, _args: Vec<Value>) -> Result<Value, RuntimeError> {
    // Simple LCG for determinism across runs (not crypto).
    // seed = time-based mod; in a real impl, keep a per-VM seed.
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u128;
    let mut x = (nanos & 0xFFFF_FFFF) as u64;
    x = x.wrapping_mul(1664525).wrapping_add(1013904223);
    Ok(Value::Number(((x >> 8) as f64) / (u32::MAX as f64)))
}

fn display(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => {
            if n.fract() == 0.0 { format!("{}", *n as i64) } else { n.to_string() }
        }
        Value::String(s) => s.clone(),
        Value::Array(a) => {
            let parts: Vec<String> = a.iter().map(display).collect();
            format!("[{}]", parts.join(", "))
        }
        Value::Object(o) => {
            let mut parts: Vec<(String, String)> = o.iter().map(|(k,v)|(k.clone(), display(v))).collect();
            parts.sort_by(|a,b| a.0.cmp(&b.0));
            let s = parts.into_iter().map(|(k,v)| format!("{}: {}", k, v)).collect::<Vec<_>>().join(", ");
            format!("{{{}}}", s)
        }
        Value::Function(_) => "<fn>".into(),
        Value::Builtin(b) => format!("<builtin:{}>", b.name),
    }
}
