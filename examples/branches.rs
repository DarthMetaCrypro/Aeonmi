use aeonmi_project::core::{ai_emitter::emit_ai, ir::*, vm::Interpreter};

fn main() {
    // const LIMIT = 5;
    // fn main() {
    //   let i = 0;
    //   let acc = 0;
    //   while (i < LIMIT) {
    //     if (i % 2 == 0) { acc = acc + i; } else { acc = acc + 1; }
    //     i = i + 1;
    //   }
    //   print(acc);
    // }

    let m = Module {
        name: "branches".into(),
        imports: vec![],
        decls: vec![
            Decl::Const(ConstDecl {
                name: "LIMIT".into(),
                value: Expr::Lit(Lit::Number(5.0)),
            }),
            Decl::Fn(FnDecl {
                name: "main".into(),
                params: vec![],
                body: Block {
                    stmts: vec![
                        Stmt::Let {
                            name: "i".into(),
                            value: Some(Expr::Lit(Lit::Number(0.0))),
                        },
                        Stmt::Let {
                            name: "acc".into(),
                            value: Some(Expr::Lit(Lit::Number(0.0))),
                        },
                        // while (i < LIMIT) { ... }
                        Stmt::While {
                            cond: Expr::Binary {
                                left: Box::new(Expr::Ident("i".into())),
                                op: BinOp::Lt,
                                right: Box::new(Expr::Ident("LIMIT".into())),
                            },
                            body: Block {
                                stmts: vec![
                                    // if (i % 2 == 0) {...} else {...}
                                    Stmt::If {
                                        cond: Expr::Binary {
                                            left: Box::new(Expr::Binary {
                                                left: Box::new(Expr::Ident("i".into())),
                                                op: BinOp::Mod,
                                                right: Box::new(Expr::Lit(Lit::Number(2.0))),
                                            }),
                                            op: BinOp::Eq,
                                            right: Box::new(Expr::Lit(Lit::Number(0.0))),
                                        },
                                        then_block: Block {
                                            stmts: vec![Stmt::Assign {
                                                target: Expr::Ident("acc".into()),
                                                value: Expr::Binary {
                                                    left: Box::new(Expr::Ident("acc".into())),
                                                    op: BinOp::Add,
                                                    right: Box::new(Expr::Ident("i".into())),
                                                },
                                            }],
                                        },
                                        else_block: Some(Block {
                                            stmts: vec![Stmt::Assign {
                                                target: Expr::Ident("acc".into()),
                                                value: Expr::Binary {
                                                    left: Box::new(Expr::Ident("acc".into())),
                                                    op: BinOp::Add,
                                                    right: Box::new(Expr::Lit(Lit::Number(1.0))),
                                                },
                                            }],
                                        }),
                                    },
                                    // i = i + 1;
                                    Stmt::Assign {
                                        target: Expr::Ident("i".into()),
                                        value: Expr::Binary {
                                            left: Box::new(Expr::Ident("i".into())),
                                            op: BinOp::Add,
                                            right: Box::new(Expr::Lit(Lit::Number(1.0))),
                                        },
                                    },
                                ],
                            },
                        },
                        // print(acc)
                        Stmt::Expr(Expr::Call {
                            callee: Box::new(Expr::Ident("print".into())),
                            args: vec![Expr::Ident("acc".into())],
                        }),
                    ],
                },
            }),
        ],
    };

    // Emit .ai script
    let out = emit_ai(&m);
    println!("=== .ai (branches) ===\n{}", out);

    // Run on the VM
    let mut vm = Interpreter::new();
    vm.run_module(&m).unwrap();
}
