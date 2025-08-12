#[cfg(feature = "quantum")]
fn main() -> anyhow::Result<()> {
    use std::env;
    use std::path::PathBuf;
    use aeonmi_project::commands::quantum::quantum_run;

    // args: backend [titan|aer]  file(ai)  [shots]
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() < 2 {
        eprintln!("usage: cargo run --example quantum_demo --features quantum[,qiskit] -- <backend> <file.ai> [shots]");
        eprintln!("example (Titan): cargo run --example quantum_demo --features quantum -- titan examples/h_stub.ai");
        eprintln!("example (Aer):   cargo run --example quantum_demo --features qiskit -- aer examples/h_stub.ai 2000");
        std::process::exit(2);
    }

    let backend = &args[0];
    let file = PathBuf::from(&args[1]);
    let shots = args.get(2).and_then(|s| s.parse::<usize>().ok());
    quantum_run(file, backend, shots)
}

#[cfg(not(feature = "quantum"))]
fn main() {
    eprintln!("Build with --features quantum (and optionally qiskit).");
}
