use rand::Rng;

pub fn simulate_markov_chain(
    transition_matrix: &[Vec<f64>],
    initial_state: &[f64],
    steps: usize,
) -> Vec<f64> {
    // Simulates a Markov chain given a transition matrix and initial state
    let mut current_state = initial_state.to_vec();

    for _ in 0..steps {
        let mut next_state = vec![0.0; transition_matrix.len()];
        for (i, row) in transition_matrix.iter().enumerate() {
            next_state[i] = row
                .iter()
                .zip(&current_state)
                .map(|(&prob, &state)| prob * state)
                .sum();
        }
        current_state = next_state;
    }

    current_state
}

pub fn random_walk_1d(steps: usize) -> Vec<i64> {
    // Simulates a 1D random walk
    let mut position = 0;
    let mut path = vec![position];
    let mut rng = rand::thread_rng(); // Still mutable as RNG is used to generate random values

    for _ in 0..steps {
        let step: i64 = if rng.gen_bool(0.5) { 1 } else { -1 };
        position += step;
        path.push(position);
    }

    path
}

pub fn simulate_poisson_process(rate: f64, time: f64) -> Vec<f64> {
    // Simulates a Poisson process with a given rate over a specified time
    let mut rng = rand::thread_rng();
    let mut events = Vec::new();
    let mut current_time = 0.0;

    while current_time < time {
        let interval = rng.gen::<f64>().ln() / (-rate);
        current_time += interval;
        if current_time < time {
            events.push(current_time);
        }
    }

    events
}

pub fn brownian_motion(steps: usize, delta_t: f64) -> Vec<f64> {
    // Simulates Brownian motion in 1D
    let mut rng = rand::thread_rng(); // Still mutable as RNG is used to generate random values
    let mut path = vec![0.0];
    let mut current_position = 0.0;

    for _ in 1..=steps {
        let delta = (delta_t.sqrt()) * rng.gen::<f64>().ln();
        current_position += delta;
        path.push(current_position);
    }

    path
}
