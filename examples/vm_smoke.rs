use aeonmi_project::core::{
    ir::*,
    vm::Interpreter,
    ai_emitter::emit_ai,
};

fn main() {
    let m = Module {
        name: "demo".into(),
        imports: vec![Import { path: "std/io".into(), alias: Some("io".into()) }],
        decls: vec![
            Decl::Const(ConstDecl {
                name: "PI".into(),
                value: Expr::Lit(Lit::Number(3.0)),
            }),
            Decl::Fn(FnDecl {
                name: "main".into(),
                params: vec![],
                body: Block {
                    stmts: vec![
                        Stmt::Let {
                            name: "x".into(),
                            value: Some(Expr::Lit(Lit::Number(2.0))),
                        },
                        Stmt::Expr(Expr::Call {
                            callee: Box::new(Expr::Ident("print".into())),
                            args: vec![Expr::Binary {
                                left: Box::new(Expr::Ident("x".into())),
                                op: BinOp::Mul,
                                right: Box::new(Expr::Ident("PI".into())),
                            }],
                        }),
                    ],
                },
            }),
        ],
    };

    // Emit .ai script
    let out = emit_ai(&m);
    println!("=== .ai ===\n{}", out);

    // Run on the VM
    let mut vm = Interpreter::new();
    vm.run_module(&m).unwrap();
}
