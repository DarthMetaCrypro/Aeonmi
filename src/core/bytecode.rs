//! Simple bytecode IR (feature: bytecode)
//! Stack-based. Operands push values; instructions operate on stack.
//! Initial subset: literals, load/store local, arithmetic, comparison, jumps, call, return.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    LoadConst(u16),     // constants[idx]
    LoadLocal(u16),     // locals[idx]
    StoreLocal(u16),
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Pop,
    Nop,
    Jump(u32),          // absolute pc
    JumpIfFalse(u32),   // absolute pc
    Call(u16, u8),      // function index, arg count (placeholder)
    Return,
}

#[derive(Debug, Default)]
pub struct Chunk {
    pub code: Vec<OpCode>,
    pub constants: Vec<Constant>,
    pub functions: Vec<FunctionInfo>,
    pub opt_stats: OptimizationStats,
}

#[derive(Debug, Clone)]
pub enum Constant { Number(f64), String(String), Bool(bool), Null }

#[derive(Debug, Clone)]
pub struct FunctionInfo { pub name: String, pub start: usize, pub arity: u8, pub locals: u16 }

impl Chunk {    pub fn add_const(&mut self, c: Constant) -> u16 { let idx = self.constants.len(); self.constants.push(c); idx as u16 }    pub fn emit(&mut self, op: OpCode) { self.code.push(op); } }

#[derive(Debug, Default, Clone)]
pub struct OptimizationStats { pub const_folds: u32, pub chain_folds: u32, pub dce_if: u32, pub dce_while: u32, pub dce_for: u32, pub pops_eliminated: u32 }

use crate::core::ast::ASTNode;
use crate::core::token::TokenKind;

pub struct BytecodeCompiler {
    chunk: Chunk,
    locals: Vec<String>, // current function locals
    functions: Vec<(String, usize, usize, u16)>, // temp table: name, start, arity, max locals
    current_function: Option<String>,
    local_max: u16,
}

impl BytecodeCompiler {
    pub fn new() -> Self { Self { chunk: Chunk { code: Vec::new(), constants: Vec::new(), functions: Vec::new(), opt_stats: OptimizationStats::default() }, locals: Vec::new(), functions: Vec::new(), current_function: None, local_max: 0 } }
    pub fn compile(mut self, ast: &ASTNode) -> Chunk { self.visit(ast); self.run_peephole(); for (n,s,a,l) in self.functions { self.chunk.functions.push(FunctionInfo { name: n, start: s, arity: a as u8, locals: l }); } self.chunk }

    fn local_index(&mut self, name: &str) -> u16 {
        if let Some(pos) = self.locals.iter().position(|n| n == name) { return pos as u16; }
        self.locals.push(name.to_string());
        let idx = (self.locals.len()-1) as u16; if idx+1 > self.local_max { self.local_max = idx+1; } idx
    }

    fn visit(&mut self, n: &ASTNode) {
        match n {
            ASTNode::Program(items) => { for it in items { self.visit(it); } }
            ASTNode::Function { name, params, body, .. } => {
                // Forward declare so recursive calls inside body resolve.
                let start = self.chunk.code.len();
                let fn_index = self.functions.len();
                self.functions.push((name.clone(), start, params.len(), 0)); // locals filled later
                let prev_fn = self.current_function.replace(name.clone());
                self.locals.clear();
                self.local_max = 0;
                for p in params { self.locals.push(p.name.clone()); self.local_max = self.local_max.max(self.locals.len() as u16); }
                for stmt in body { self.visit(stmt); }
                // If function didn't end with explicit return, push Null and return implicitly.
                if !matches!(self.chunk.code.last(), Some(OpCode::Return)) {
                    let null_idx = self.null_const();
                    self.chunk.emit(OpCode::LoadConst(null_idx));
                    self.chunk.emit(OpCode::Return);
                }
                // Patch locals count for this function
                if let Some(entry) = self.functions.get_mut(fn_index) { entry.3 = self.local_max; }
                self.current_function = prev_fn;
                self.locals.clear();
                self.local_max = 0;
            }
            ASTNode::VariableDecl { name, value, .. } => { 
                self.visit(value); 
                let idx = self.local_index(name); 
                self.chunk.emit(OpCode::StoreLocal(idx)); 
                // Clean value from stack; declarations as statements shouldn't leak.
                self.chunk.emit(OpCode::Pop);
            }
            ASTNode::Assignment { name, value, .. } => { 
                self.visit(value); 
                let idx = self.local_index(name); 
                self.chunk.emit(OpCode::StoreLocal(idx)); 
                // Treat assignment as statement for now; discard value.
                self.chunk.emit(OpCode::Pop);
            }
            ASTNode::NumberLiteral(v) => { let c = self.chunk.add_const(Constant::Number(*v)); self.chunk.emit(OpCode::LoadConst(c)); }
            ASTNode::StringLiteral(s) => { let c = self.chunk.add_const(Constant::String(s.clone())); self.chunk.emit(OpCode::LoadConst(c)); }
            ASTNode::BooleanLiteral(b) => { let c = self.chunk.add_const(Constant::Bool(*b)); self.chunk.emit(OpCode::LoadConst(c)); }
            ASTNode::Identifier(name) => { let idx = self.local_index(name); self.chunk.emit(OpCode::LoadLocal(idx)); }
            ASTNode::IdentifierSpanned { name, .. } => { let idx = self.local_index(name); self.chunk.emit(OpCode::LoadLocal(idx)); }
            ASTNode::BinaryExpr { .. } => { self.emit_binary_or_fold(n); }
            ASTNode::Return(expr) => { self.visit(expr); self.chunk.emit(OpCode::Return); }
            ASTNode::Log(expr) => { self.visit(expr); self.chunk.emit(OpCode::Pop); } // discard for now
            ASTNode::Block(items) => { for it in items { self.visit(it); } }
            ASTNode::If { condition, then_branch, else_branch } => {
                if let Some(Constant::Bool(b)) = self.fold_const(condition) { // DCE
                    self.chunk.opt_stats.dce_if += 1;
                    if b { self.visit(then_branch); } else if let Some(e)=else_branch { self.visit(e); }
                    return;
                }
                // condition
                self.visit(condition);
                // emit JumpIfFalse placeholder
                let cond_jump_pos = self.chunk.code.len();
                self.chunk.emit(OpCode::JumpIfFalse(0));
                // then branch
                self.visit(then_branch);
                // if else present, emit jump over else
                if let Some(e) = else_branch {
                    let after_then_jump = self.chunk.code.len();
                    self.chunk.emit(OpCode::Jump(0));
                    // patch JumpIfFalse to start of else
                    let else_start = self.chunk.code.len() as u32;
                    if let OpCode::JumpIfFalse(ref mut target) = self.chunk.code[cond_jump_pos] { *target = else_start; }
                    // else body
                    self.visit(e);
                    // patch jump after then to after else
                    let after_else = self.chunk.code.len() as u32;
                    if let OpCode::Jump(ref mut t) = self.chunk.code[after_then_jump] { *t = after_else; }
                } else {
                    // no else: patch JumpIfFalse to next instruction
                    let after_then = self.chunk.code.len() as u32;
                    if let OpCode::JumpIfFalse(ref mut target) = self.chunk.code[cond_jump_pos] { *target = after_then; }
                }
            }
            ASTNode::While { condition, body } => {
                if let Some(Constant::Bool(false)) = self.fold_const(condition) { self.chunk.opt_stats.dce_while += 1; return; }
                let loop_start = self.chunk.code.len() as u32;
                self.visit(condition);
                let jump_if_false_pos = self.chunk.code.len();
                self.chunk.emit(OpCode::JumpIfFalse(0));
                self.visit(body);
                // jump back to loop start
                self.chunk.emit(OpCode::Jump(loop_start));
                let after_loop = self.chunk.code.len() as u32;
                if let OpCode::JumpIfFalse(ref mut target) = self.chunk.code[jump_if_false_pos] { *target = after_loop; }
            }
            ASTNode::For { init, condition, increment, body } => {
                // init
                if let Some(i) = init { self.visit(i); }
                let loop_start = self.chunk.code.len() as u32;
                // condition
                let cond_is_false = if let Some(c) = condition { if let Some(Constant::Bool(false)) = self.fold_const(c) { true } else { false } } else { false };
                if cond_is_false { self.chunk.opt_stats.dce_for += 1; return; }
                if let Some(c) = condition { self.visit(c); } else { // implicit true
                    let true_idx = self.chunk.add_const(Constant::Bool(true));
                    self.chunk.emit(OpCode::LoadConst(true_idx));
                }
                let jump_if_false_pos = self.chunk.code.len();
                self.chunk.emit(OpCode::JumpIfFalse(0));
                // body
                self.visit(body);
                // increment
                if let Some(inc) = increment { self.visit(inc); self.chunk.emit(OpCode::Pop); }
                // jump back
                self.chunk.emit(OpCode::Jump(loop_start));
                let after_for = self.chunk.code.len() as u32;
                if let OpCode::JumpIfFalse(ref mut target) = self.chunk.code[jump_if_false_pos] { *target = after_for; }
            }
            ASTNode::Call { callee, args } => {
                if let ASTNode::Identifier(name) = &**callee {
                    if let Some((idx,_start,arity,_locs)) = self.functions.iter().enumerate().find_map(|(i,(n,s,a,l))| if n==name { Some((i,*s,*a,*l)) } else { None }) {
                        for a in args { self.visit(a); }
                        self.chunk.emit(OpCode::Call(idx as u16, arity as u8));
                    }
                }
            }
            _ => { /* quantum ops not yet */ }
        }
    }

    fn translate_bin(&mut self, op: &TokenKind) {
        use TokenKind::*;
        let bc = match op { Plus=>OpCode::Add, Minus=>OpCode::Sub, Star=>OpCode::Mul, Slash=>OpCode::Div,
            DoubleEquals=>OpCode::Eq, NotEquals=>OpCode::Ne, LessThan=>OpCode::Lt, LessEqual=>OpCode::Le, GreaterThan=>OpCode::Gt, GreaterEqual=>OpCode::Ge,
            _ => OpCode::Pop };
        self.chunk.emit(bc);
    }
    fn emit_binary_or_fold(&mut self, node: &ASTNode) {
    if let Some(cst) = self.fold_const(node) { self.chunk.opt_stats.const_folds += 1; let idx = self.chunk.add_const(cst); self.chunk.emit(OpCode::LoadConst(idx)); return; }
        if let ASTNode::BinaryExpr { op, left, right } = node {
            use TokenKind::*;
            match op {
                AndAnd => { // short-circuit: evaluate left; if false -> skip right
                    self.visit(left);
                    let jump_pos = self.chunk.code.len();
                    self.chunk.emit(OpCode::JumpIfFalse(0));
                    // if left true, pop it and evaluate right, leaving right
                    self.chunk.emit(OpCode::Pop);
                    self.visit(right);
                    let after = self.chunk.code.len() as u32;
                    if let OpCode::JumpIfFalse(ref mut t) = self.chunk.code[jump_pos] { *t = after; }
                }
                OrOr => { // short-circuit: if left true skip right
                    self.visit(left);
                    let jump_pos = self.chunk.code.len();
                    // JumpIfFalse goes over next pop/eval when false; we invert by adding Jump for true path
                    self.chunk.emit(OpCode::JumpIfFalse(0));
                    // left true: keep it, jump over evaluation of right
                    let skip_right_jump = self.chunk.code.len();
                    self.chunk.emit(OpCode::Jump(0));
                    // patch JumpIfFalse target to evaluate right
                    let right_start = self.chunk.code.len() as u32;
                    if let OpCode::JumpIfFalse(ref mut t) = self.chunk.code[jump_pos] { *t = right_start; }
                    // evaluate right (discard left first)
                    self.chunk.emit(OpCode::Pop);
                    self.visit(right);
                    let after_right = self.chunk.code.len() as u32;
                    if let OpCode::Jump(ref mut t) = self.chunk.code[skip_right_jump] { *t = after_right; }
                }
                _ => { self.visit(left); self.visit(right); self.translate_bin(op); }
            }
        }
    }

    fn null_const(&mut self) -> u16 {
        if let Some(idx) = self.chunk.constants.iter().position(|c| matches!(c, Constant::Null)) { idx as u16 } else { self.chunk.add_const(Constant::Null) }
    }

    // Attempt to recursively fold a constant expression into a single Constant.
    fn fold_const(&mut self, node: &ASTNode) -> Option<Constant> {
        use TokenKind::*;
        match node {
            ASTNode::NumberLiteral(n) => Some(Constant::Number(*n)),
            ASTNode::StringLiteral(s) => Some(Constant::String(s.clone())),
            ASTNode::BooleanLiteral(b) => Some(Constant::Bool(*b)),
            ASTNode::BinaryExpr { op, left, right } => {
                // Associative chain folding for +, *, &&, ||
                if matches!(op, Plus | Star | AndAnd | OrOr) {
                    if let Some(folded) = self.fold_associative_chain(op, node) { return Some(folded); }
                }
                let lc = self.fold_const(left)?;
                let rc = self.fold_const(right)?;
                match (op, lc, rc) {
                    (Plus, Constant::Number(a), Constant::Number(b)) => Some(Constant::Number(a + b)),
                    (Plus, Constant::Number(a), Constant::String(b)) => Some(Constant::String(format!("{}{}", a, b))),
                    (Plus, Constant::String(a), Constant::Number(b)) => Some(Constant::String(format!("{}{}", a, b))),
                    (Minus, Constant::Number(a), Constant::Number(b)) => Some(Constant::Number(a - b)),
                    (Star, Constant::Number(a), Constant::Number(b)) => Some(Constant::Number(a * b)),
                    (Slash, Constant::Number(a), Constant::Number(b)) => Some(Constant::Number(if b==0.0 { 0.0 } else { a / b })),
                    (DoubleEquals, Constant::Number(a), Constant::Number(b)) => Some(Constant::Bool(a == b)),
                    (NotEquals, Constant::Number(a), Constant::Number(b)) => Some(Constant::Bool(a != b)),
                    (LessThan, Constant::Number(a), Constant::Number(b)) => Some(Constant::Bool(a < b)),
                    (LessEqual, Constant::Number(a), Constant::Number(b)) => Some(Constant::Bool(a <= b)),
                    (GreaterThan, Constant::Number(a), Constant::Number(b)) => Some(Constant::Bool(a > b)),
                    (GreaterEqual, Constant::Number(a), Constant::Number(b)) => Some(Constant::Bool(a >= b)),
                    (AndAnd, Constant::Bool(a), Constant::Bool(b)) => Some(Constant::Bool(a && b)),
                    (OrOr, Constant::Bool(a), Constant::Bool(b)) => Some(Constant::Bool(a || b)),
                    // String concatenation
                    (Plus, Constant::String(a), Constant::String(b)) => Some(Constant::String(format!("{}{}", a, b))),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn fold_associative_chain(&mut self, op: &TokenKind, root: &ASTNode) -> Option<Constant> {
        use TokenKind::*;
        // Gather all operands in a flat vec
        fn gather<'a>(node: &'a ASTNode, target: &TokenKind, out: &mut Vec<&'a ASTNode>) {
            if let ASTNode::BinaryExpr { op, left, right } = node {
                if op == target { gather(left, target, out); gather(right, target, out); return; }
            }
            out.push(node);
        }
        let mut ops: Vec<&ASTNode> = Vec::new(); gather(root, op, &mut ops);
    match op {
            Plus => {
                // Numbers only or strings only
                if ops.is_empty() { return None; }
                let any_string = ops.iter().any(|n| matches!(n, ASTNode::StringLiteral(_)));
                let all_numbers = ops.iter().all(|n| matches!(n, ASTNode::NumberLiteral(_)));
                if all_numbers && !any_string { let mut sum = 0.0; for n in &ops { if let ASTNode::NumberLiteral(v)=*n { sum += v; } } if ops.len()>2 { self.chunk.opt_stats.chain_folds +=1; } return Some(Constant::Number(sum)); }
                if any_string && ops.iter().all(|n| matches!(n, ASTNode::NumberLiteral(_)|ASTNode::StringLiteral(_))) {
                    let mut s = String::new();
                    for n in &ops { match *n { ASTNode::NumberLiteral(v)=> s.push_str(&v.to_string()), ASTNode::StringLiteral(ref st)=> s.push_str(st), _=> return None } }
                    if ops.len()>2 { self.chunk.opt_stats.chain_folds +=1; } return Some(Constant::String(s));
                }
                None
            }
            Star => {
                if ops.is_empty() { return None; }
                if ops.iter().all(|n| matches!(n, ASTNode::NumberLiteral(_))) {
                    let mut prod = 1.0; for n in &ops { if let ASTNode::NumberLiteral(v)=*n { prod *= v; } }
                    if ops.len()>2 { self.chunk.opt_stats.chain_folds +=1; } return Some(Constant::Number(prod));
                }
                None
            }
            AndAnd => {
                if ops.iter().all(|n| matches!(n, ASTNode::BooleanLiteral(_))) {
                    let res = ops.iter().all(|n| if let ASTNode::BooleanLiteral(b)=*n { *b } else { false });
                    if ops.len()>2 { self.chunk.opt_stats.chain_folds +=1; } return Some(Constant::Bool(res));
                }
                None
            }
            OrOr => {
                if ops.iter().all(|n| matches!(n, ASTNode::BooleanLiteral(_))) {
                    let res = ops.iter().any(|n| if let ASTNode::BooleanLiteral(b)=*n { *b } else { false });
                    if ops.len()>2 { self.chunk.opt_stats.chain_folds +=1; } return Some(Constant::Bool(res));
                }
                None
            }
            _ => None
        }
    }

    fn run_peephole(&mut self) {
        // Replace redundant Pop sequences (Pop Pop) with Pop + Nop so addresses stable
        for i in 1..self.chunk.code.len() { if matches!(self.chunk.code[i-1], OpCode::Pop) && matches!(self.chunk.code[i], OpCode::Pop) { self.chunk.code[i] = OpCode::Nop; self.chunk.opt_stats.pops_eliminated += 1; } }
    }
}

// Simple textual disassembler (debug)
pub fn disassemble(chunk: &Chunk) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    writeln!(&mut out, "== constants ({} ) ==", chunk.constants.len()).ok();
    for (i,c) in chunk.constants.iter().enumerate() { match c { Constant::Number(n)=>writeln!(&mut out, "[{i}] num {n}").ok(), Constant::String(s)=>writeln!(&mut out, "[{i}] str \"{s}\"").ok(), Constant::Bool(b)=>writeln!(&mut out, "[{i}] bool {b}").ok(), Constant::Null=>writeln!(&mut out, "[{i}] null").ok(), }; }
    writeln!(&mut out, "== functions ({} ) ==", chunk.functions.len()).ok();
    for (i,f) in chunk.functions.iter().enumerate() { writeln!(&mut out, "fn#{i} {} start={} arity={} locals={}", f.name, f.start, f.arity, f.locals).ok(); }
    writeln!(&mut out, "== code ({} ops) ==", chunk.code.len()).ok();
    for (i,op) in chunk.code.iter().enumerate() { use OpCode::*; match op { LoadConst(c)=>writeln!(&mut out, "{i:04} LOAD_CONST {c}").ok(), LoadLocal(l)=>writeln!(&mut out, "{i:04} LOAD_LOCAL {l}").ok(), StoreLocal(l)=>writeln!(&mut out, "{i:04} STORE_LOCAL {l}").ok(), Add=>writeln!(&mut out, "{i:04} ADD").ok(), Sub=>writeln!(&mut out, "{i:04} SUB").ok(), Mul=>writeln!(&mut out, "{i:04} MUL").ok(), Div=>writeln!(&mut out, "{i:04} DIV").ok(), Eq=>writeln!(&mut out, "{i:04} EQ").ok(), Ne=>writeln!(&mut out, "{i:04} NE").ok(), Lt=>writeln!(&mut out, "{i:04} LT").ok(), Le=>writeln!(&mut out, "{i:04} LE").ok(), Gt=>writeln!(&mut out, "{i:04} GT").ok(), Ge=>writeln!(&mut out, "{i:04} GE").ok(), And=>writeln!(&mut out, "{i:04} AND").ok(), Or=>writeln!(&mut out, "{i:04} OR").ok(), Pop=>writeln!(&mut out, "{i:04} POP").ok(), Nop=>writeln!(&mut out, "{i:04} NOP").ok(), Jump(t)=>writeln!(&mut out, "{i:04} JUMP {t}").ok(), JumpIfFalse(t)=>writeln!(&mut out, "{i:04} JUMP_IF_FALSE {t}").ok(), Call(f,a)=>writeln!(&mut out, "{i:04} CALL f={} argc={}", f,a).ok(), Return=>writeln!(&mut out, "{i:04} RETURN").ok(), }; }
    out }
