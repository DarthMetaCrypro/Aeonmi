//! Titan math/quantum library (pre-integration)
//! Many public APIs aren’t referenced yet; keep builds clean while we wire them in.
#![allow(dead_code)]

pub mod algebra;
pub mod arithmetic;
pub mod advanced_linear_algebra;
pub mod advanced_fourier_signal_processing;
pub mod advanced_quantum_math;
pub mod advanced_tensor_calculus;
pub mod calculus;
pub mod chaos_theory_dynamical_systems;
pub mod complex_numbers;
pub mod differential_equations;
pub mod differential_geometry;
pub mod discrete_math;
pub mod fourier_wavelet;
pub mod fractals;
pub mod geometry;
pub mod linear_algebra;
pub mod multi_dimensional_math;
pub mod numerical_solvers;
pub mod optimization;
pub mod probability_statistics;
pub mod quantum_gates;
pub mod quantum_math;
pub mod quantum_superposition;
pub mod quantum_tensor_ops;
pub mod statistics;
pub mod stochastic_processes;
pub mod symbolic_math;
pub mod tensor_calculus;

// --- Quantum core (feature-gated) ---
#[cfg(feature = "quantum")]
pub mod types;

#[cfg(feature = "quantum")]
pub mod gates;

#[cfg(feature = "quantum")]
pub mod ops;

// Python/Qiskit bridge (separate feature)
#[cfg(feature = "qiskit")]
pub mod qiskit_bridge;
