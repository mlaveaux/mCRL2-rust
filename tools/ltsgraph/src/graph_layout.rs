use std::sync::Arc;

use io::aut::LTS;

use glam::Vec3;
use log::debug;
use rand::Rng;

pub struct GraphLayout {
    pub lts: Arc<LTS>,
    pub layout_states: Vec<StateLayout>,
}

#[derive(Default)]
pub struct StateLayout {
    pub position: Vec3,
    pub force: Vec3,
}

enum Pair<T> {
    Both(T, T),
    One(T),
}

fn index_twice<T>(slc: &mut [T], a: usize, b: usize) -> Pair<&mut T> {
    if a == b {
        Pair::One(slc.get_mut(a).unwrap())
    } else {
        assert!(a <= slc.len() && b < slc.len());

        // safe because a, b are in bounds and distinct
        unsafe {
            let ar = &mut *(slc.get_unchecked_mut(a) as *mut _);
            let br = &mut *(slc.get_unchecked_mut(b) as *mut _);
            Pair::Both(ar, br)
        }
    }
}

impl GraphLayout {
    /// Construct a new layout for the given LTS.
    pub fn new(lts: &Arc<LTS>) -> GraphLayout {
        // Keep track of state layout information.
        let mut states_simulation = Vec::with_capacity(lts.states.len());
        states_simulation.resize_with(lts.states.len(), || StateLayout::default());

        // Place the states at a render position
        let mut rng = rand::thread_rng();
        for state in &mut states_simulation {
            state.position.x = rng.gen_range(0.0..1000.0);
            state.position.y = rng.gen_range(0.0..1000.0);
        }

        GraphLayout {
            lts: lts.clone(),
            layout_states: states_simulation,
        }
    }

    /// Update the layout one step using spring forces for transitions and repulsion between states.
    pub fn update(&mut self, handle_length: f32, repulsion_strength: f32, delta: f32) {

        debug!("delta = {}", delta);

        for (state_index, state) in self.lts.states.iter().enumerate() {
            // Ignore the last state since it cannot repulse with any other state.
            if state_index < self.layout_states.len() {
                // Use split_at_mut to get two mutable slices at every split point.
                let (left_layout, right_layout) =
                    self.layout_states.split_at_mut(state_index + 1);
                let state_layout = left_layout.last_mut().unwrap();

                // Accumulate repulsion forces between vertices.
                for other_state_layout in right_layout {
                    let force = compute_repulsion_force(
                        &state_layout.position,
                        &other_state_layout.position,
                        repulsion_strength,
                    );

                    state_layout.force += force;
                    other_state_layout.force -= force;
                }
            }

            // Accumulate forces over all connected edges.
            for (_, to_index) in &state.outgoing {
                // Index an edge in the graph.
                match index_twice(&mut self.layout_states, state_index, *to_index) {
                    Pair::One(_) => {
                        // Handle self loop, but we apply no forces in this case.
                    }
                    Pair::Both(from_info, to_info) => {
                        let force = compute_spring_force(
                            &from_info.position,
                            &to_info.position,
                            handle_length,
                        );

                        from_info.force += force;
                        to_info.force -= force;

                        // Remove the forces applied above since these vertices are connected, this is cheaper than trying to figure
                        // out that states are not connected. The graph is typically sparse.
                        from_info.force -= compute_repulsion_force(
                            &from_info.position,
                            &to_info.position,
                            repulsion_strength,
                        );
                    }
                }
            }
        }

        for state_info in &mut self.layout_states {
            // Integrate the forces.
            state_info.position += state_info.force * delta;

            // Reset the force.
            state_info.force = Vec3::default();

            // A safety check for when the layout exploded.
            assert!(
                state_info.position.is_finite(),
                "Invalid position {} obtained",
                state_info.position
            );
        }
    }
}

/// Compute a sping force between two points with a desired rest length.
fn compute_spring_force(p1: &Vec3, p2: &Vec3, rest_length: f32) -> Vec3 {
    let dist = p1.distance(*p2);

    if dist < 1.0 {
        // Give it some offset force.
        Vec3::new(0.0, 0.0, 0.0)
    } else {
        // This is already multiplied by -1.0, i.e. (p2 - p1) == (p1 - p2) * -1.0
        (*p2 - *p1) / dist * f32::log2(dist / rest_length)
    }
}

/// Computes a repulsion force between two points with a given strength.
fn compute_repulsion_force(p1: &Vec3, p2: &Vec3, repulsion_strength: f32) -> Vec3 {
    let dist = p1.distance_squared(*p2);

    if dist < 1.0 {
        // Give it some offset force.
        Vec3::new(0.0, 0.0, 0.0)
    } else {
        (*p1 - *p2) * repulsion_strength / dist
    }
}