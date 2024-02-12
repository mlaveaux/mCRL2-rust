use io::aut::{State, LTS};

use glam::Vec3;
use log::debug;
use rand::Rng;

pub struct Simulation {
    pub lts: LTS,
    pub states_simulation: Vec<StateInfo>,
}

#[derive(Default)]
pub struct StateInfo {
    pub position: Vec3,
}

/// 
fn compute_force(p1: &Vec3, p2: &Vec3, handle_length: f32) -> Vec3 {
    (*p2 - *p1) * (p1.distance(*p2) / handle_length)
}

impl Simulation {

    /// Construct a new simulation for the given LTS.
    pub fn new(lts: LTS) -> Simulation {
        // Keep track of state information.
        let mut states_simulation = Vec::with_capacity(lts.states.len());
        states_simulation.resize_with(lts.states.len(), || {
            StateInfo::default()
        });

        // Place the states at a render position
        let mut rng = rand::thread_rng();
        for state in &mut states_simulation {
            state.position.x = rng.gen_range(0.0 .. 800.0);
            state.position.y = rng.gen_range(0.0 .. 600.0);
        }

        Simulation {
            lts,
            states_simulation
        }
    }

    /// Update the simulation one step.
    pub fn update(&mut self) {
        let handle_length = 100.0;

        for (from_index, state) in &mut self.lts.states.iter().enumerate() {
            let mut force = Vec3::default();

            // Accumulate forces over all connected edges.
            for (_, to_index) in &state.outgoing {
                let from_pos = &self.states_simulation[from_index];
                let to_pos = &self.states_simulation[*to_index];

                force += compute_force(&from_pos.position, &to_pos.position, handle_length);
            }

            let from_pos = &mut self.states_simulation[from_index];
            from_pos.position += force * 0.01;

            assert!(from_pos.position.is_finite(), "Invalid position obtained");

            //debug!("State {} new position {} {}", from_index, from_pos.position.x, from_pos.position.y);
        }

    }
}