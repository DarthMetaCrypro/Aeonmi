use aeonmi_project::core::{ai_emitter::emit_ai, ir::*, vm::Interpreter};

#[test]
fn smoke_emits_and_runs() {
    // const PI = 3; fn main(){ let x = 2; print(x * PI); }
    let m = Module {
        name: "demo".into(),
        imports: vec![], // no std/io import needed for this smoke test
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

    // 1) Emitter contains key lines
    let out = emit_ai(&m);
    assert!(out.contains("const PI = 3"), "emitter output missing const");
    assert!(out.contains("let x = 2"), "emitter output missing let");
    assert!(
        out.contains("print(x * PI)"),
        "emitter output missing print expr"
    );

    // 2) VM runs module without error
    let mut vm = Interpreter::new();
    assert!(vm.run_module(&m).is_ok(), "VM failed to run module");
}
