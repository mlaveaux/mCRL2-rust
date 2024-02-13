use io::aut::LTS;

use glam::Vec3;
use rand::Rng;

pub struct GraphLayout {
    pub lts: LTS,
    pub states_simulation: Vec<StateInfo>,
}

#[derive(Default)]
pub struct StateInfo {
    pub position: Vec3,
    pub force: Vec3,
}

/// Compute a sping force between two points with a desired rest length.
fn compute_spring_force(p1: &Vec3, p2: &Vec3, rest_length: f32) -> Vec3 {
    let dist = p1.distance(*p2);

    if dist < 1.0 {
        // Give it some offset force.
        Vec3::new(1.0, 0.0, 0.0)
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
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        (*p1 - *p2) * repulsion_strength / dist
    }
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

    /// Construct a new simulation for the given LTS.
    pub fn new(lts: LTS) -> GraphLayout {
        // Keep track of state information.
        let mut states_simulation = Vec::with_capacity(lts.states.len());
        states_simulation.resize_with(lts.states.len(), || {
            StateInfo::default()
        });

        // Place the states at a render position
        let mut rng = rand::thread_rng();
        for state in &mut states_simulation {
            state.position.x = rng.gen_range(0.0 .. 1000.0);
            state.position.y = rng.gen_range(0.0 .. 1000.0);
        }

        GraphLayout {
            lts,
            states_simulation
        }
    }

    /// Update the simulation one step.
    pub fn update(&mut self, handle_length: f32, repulsion_strength: f32, delta: f32) {
        for (state_index, state) in self.lts.states.iter_mut().enumerate() {

            // Ignore the last state since it cannot repulse with any other state.
            if state_index < self.states_simulation.len() {                
                // Use split_at_mut to get two mutable slices at every split point.
                let (left_simulation, right_simulation) = self.states_simulation.split_at_mut(state_index + 1);
                let state_info = left_simulation.last_mut().unwrap();

                // Accumulate repulsion forces between vertices.
                for other_state_info in right_simulation{
                    let force = compute_repulsion_force(&state_info.position, &other_state_info.position, repulsion_strength);
                    
                    state_info.force += force;
                    other_state_info.force -= force;
                }
            }

            // Accumulate forces over all connected edges.
            for (_, to_index) in &state.outgoing {

                // Index an edge in the graph.
                match index_twice(&mut self.states_simulation, state_index, *to_index) {
                    Pair::One(x) => {
                        // Handle self loop
                    },
                    Pair::Both(from_info, to_info) => {
                        let force = compute_spring_force(&from_info.position, &to_info.position, handle_length);

                        from_info.force += force;
                        to_info.force -= force;

                        // Remove the forces applied above since these vertices are connected, this is cheaper than trying to figure
                        // out that states are not connected. The graph is typically sparse.
                        from_info.force -= compute_repulsion_force(&from_info.position, &to_info.position, repulsion_strength);

                    }
                }                
            }
        }
        
        for state_info in &mut self.states_simulation {
            // Integrate the forces.
            state_info.position += state_info.force * delta;

            // Reset the force.
            state_info.force = Vec3::default();

            // A safety check for when the layout exploded.
            assert!(state_info.position.is_finite(), "Invalid position {} obtained", state_info.position);
        }

    }
}