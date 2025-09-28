//! Simple bytecode VM (feature: bytecode)
use crate::core::bytecode::{Chunk, OpCode, Constant};

#[derive(Debug, Clone)]
pub enum Value { Number(f64), String(String), Bool(bool), Null }

#[derive(Debug)]
struct Frame { return_ip: usize, locals: Vec<Value> }

pub struct VM<'a> { pub chunk: &'a Chunk, stack: Vec<Value>, ip: usize, frames: Vec<Frame>, pub stack_overflow: bool, max_frames: usize }

impl<'a> VM<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        let max_frames = std::env::var("AEONMI_MAX_FRAMES")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .map(|n| n.clamp(4, 65_536))
            .unwrap_or(256);
        Self { chunk, stack: Vec::new(), ip: 0, frames: vec![Frame { return_ip: usize::MAX, locals: vec![Value::Null; 64] }], stack_overflow: false, max_frames }
    }
    pub fn run(&mut self) -> Option<Value> {
        while self.ip < self.chunk.code.len() {
            match self.chunk.code[self.ip] { op => { self.ip += 1; if !self.dispatch(op) { break; } } }
        }
        self.stack.pop()
    }

    fn dispatch(&mut self, op: OpCode) -> bool {
        use OpCode::*;
        match op {
            LoadConst(i) => { let c = &self.chunk.constants[i as usize]; self.stack.push(match c { Constant::Number(n)=>Value::Number(*n), Constant::String(s)=>Value::String(s.clone()), Constant::Bool(b)=>Value::Bool(*b), Constant::Null=>Value::Null }); }
            LoadLocal(i) => { if let Some(frame) = self.frames.last() { let v = frame.locals.get(i as usize).cloned().unwrap_or(Value::Null); self.stack.push(v); } }
            StoreLocal(i) => { if let Some(frame) = self.frames.last_mut() { if let Some(v)= self.stack.last().cloned() { if (i as usize) < frame.locals.len() { frame.locals[i as usize] = v; } } } }
            Add => add_any(self), Sub => bin(self, |a,b| a-b), Mul => bin(self, |a,b| a*b), Div => bin(self, |a,b| if b==0.0 { 0.0 } else { a/b }),
            Eq|Ne|Lt|Le|Gt|Ge => cmp(self, op),
            And => logical(self, true), Or => logical(self, false),
            Pop => { self.stack.pop(); },
            Return => {
                // Pop current frame; if no previous frame, halt.
                if let Some(frame) = self.frames.pop() {
                    if frame.return_ip == usize::MAX { return false; } // root frame returned => halt
                    self.ip = frame.return_ip;
                } else { return false; }
            }
        Call(func_index, arity) => {
                if let Some(info) = self.chunk.functions.get(func_index as usize) {
            if self.frames.len() >= self.max_frames { self.stack_overflow = true; return false; }
                    // Pop args into temp (reverse to locals order)
                    let mut args: Vec<Value> = Vec::new();
                    for _ in 0..arity { if let Some(v)=self.stack.pop() { args.push(v); } }
                    args.reverse();
                    // Allocate new frame sized to function max locals (at least 1)
                    let mut locals = vec![Value::Null; (info.locals as usize).max(1)];
                    for (i,arg) in args.into_iter().enumerate() { if i < locals.len() { locals[i] = arg; } }
                    let ret_ip = self.ip;
                    self.frames.push(Frame { return_ip: ret_ip, locals });
                    self.ip = info.start;
                }
            }
            Nop => { },
            _ => { /* unimplemented ops ignored for now */ }
        }
        true
    }
}

fn bin(vm: &mut VM, f: impl Fn(f64,f64)->f64) { if let (Some(r), Some(l)) = (vm.stack.pop(), vm.stack.pop()) { if let (Value::Number(rb), Value::Number(lb)) = (r,l) { vm.stack.push(Value::Number(f(lb,rb))); } else { vm.stack.push(Value::Null); } } }
fn cmp(vm: &mut VM, op: OpCode) { use OpCode::*; if let (Some(r), Some(l)) = (vm.stack.pop(), vm.stack.pop()) { if let (Value::Number(rb), Value::Number(lb)) = (r,l) { let res = match op { Eq=> lb==rb, Ne=> lb!=rb, Lt=> lb<rb, Le=> lb<=rb, Gt=> lb>rb, Ge=> lb>=rb, _=> false }; vm.stack.push(Value::Bool(res)); } else { vm.stack.push(Value::Bool(false)); } } }
fn logical(vm: &mut VM, is_and: bool) { if let (Some(r), Some(l)) = (vm.stack.pop(), vm.stack.pop()) {
    let lb = matches!(l, Value::Bool(true)); let rb = matches!(r, Value::Bool(true));
    let res = if is_and { lb && rb } else { lb || rb }; vm.stack.push(Value::Bool(res));
} }
fn add_any(vm: &mut VM) { if let (Some(r), Some(l)) = (vm.stack.pop(), vm.stack.pop()) {
    match (l,r) {
        (Value::Number(a), Value::Number(b)) => vm.stack.push(Value::Number(a+b)),
        (Value::String(a), Value::String(b)) => vm.stack.push(Value::String(format!("{}{}", a,b))),
        (Value::String(a), Value::Number(b)) => vm.stack.push(Value::String(format!("{}{}", a,b))),
        (Value::Number(a), Value::String(b)) => vm.stack.push(Value::String(format!("{}{}", a,b))),
        (Value::Bool(a), Value::String(b)) => vm.stack.push(Value::String(format!("{}{}", a,b))),
        (Value::String(a), Value::Bool(b)) => vm.stack.push(Value::String(format!("{}{}", a,b))),
        (Value::Bool(a), Value::Bool(b)) => vm.stack.push(Value::String(format!("{}{}", a,b))),
    (_l,_r) => vm.stack.push(Value::Null),
    }
} }
