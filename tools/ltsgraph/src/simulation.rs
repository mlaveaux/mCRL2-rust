use io::aut::LTS;

use glam::Vec3;
use itertools::Itertools;
use rand::Rng;

pub struct Simulation {
    pub lts: LTS,
    pub states_simulation: Vec<StateInfo>,
}

#[derive(Default)]
pub struct StateInfo {
    pub position: Vec3,
}

/// Compute a sping force between two points with a desired rest length.
fn compute_spring_force(p1: &Vec3, p2: &Vec3, rest_length: f32) -> Vec3 {
    (*p2 - *p1) * (p1.distance(*p2) / rest_length)
}

fn lerp(t: f32, u0: f32, u1: f32) -> f32 {
    ((1.0 - t) * u0 + t * u1).clamp(u1, u0)
}

fn compute_repulsion_force(p1: &Vec3, p2: &Vec3, rest_length: f32) -> Vec3 {
    (*p1 - *p2) * lerp(p1.distance(*p2) / rest_length, 1.0, 0.0)
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
        let repulsion_dist = 25.0;

        for state_index in 1..self.lts.states.len() {

            let (left, right) = self.lts.states.split_at(state_index);   
            let state = left.last().unwrap();

            for (other_index, _) in right.iter().enumerate() {
                let other_index = other_index + state_index;
                
                let mut force = Vec3::default();

                // Accumulate forces over all connected edges.
                let state_pos = &self.states_simulation[state_index];
                if state_index == other_index {
                    for (_, to_index) in &state.outgoing {
                        let to_pos = &self.states_simulation[*to_index];

                        force += compute_spring_force(&state_pos.position, &to_pos.position, handle_length);
                    }
                }

                // Accumulate forces between vertices.
                if state_index != other_index {
                    let other_pos = &self.states_simulation[other_index];
                    force += compute_repulsion_force(&state_pos.position, &other_pos.position, repulsion_dist);
                }

                // Update the position based on a fixed delta.
                let state_pos = &mut self.states_simulation[state_index];
                state_pos.position += force * 0.01;

                assert!(state_pos.position.is_finite(), "Invalid position obtained");

                //debug!("State {} new position {} {}", from_index, from_pos.position.x, from_pos.position.y);
            }
        }

    }
}