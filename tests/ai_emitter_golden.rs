use aeonmi_project::core::{ ir::*, ai_emitter::emit_ai, vm::Interpreter };

#[test]
fn ai_emitter_golden_minimal() {
    let m = Module {
        name: "golden".into(),
        imports: vec![],
        decls: vec![
            Decl::Const(ConstDecl { name: "PI".into(), value: Expr::Lit(Lit::Number(3.0)) }),
            Decl::Fn(FnDecl {
                name: "main".into(),
                params: vec![],
                body: Block {
                    stmts: vec![
                        Stmt::Let { name: "x".into(), value: Some(Expr::Lit(Lit::Number(2.0))) },
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

    let out = emit_ai(&m);
    assert!(out.contains("const PI = 3;"));
    assert!(out.contains("let x = 2;"));
    assert!(out.contains("print(x * PI);"));

    // VM sanity too (doesn’t assert stdout, just no crash)
    let mut vm = Interpreter::new();
    vm.run_module(&m).unwrap();
}
