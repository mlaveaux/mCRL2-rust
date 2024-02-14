slint::include_modules!();

use std::{fs::File, ops::Deref, sync::{atomic::AtomicBool, Arc, Mutex, RwLock}, thread, time::{Duration, Instant}};

use anyhow::Result;
use clap::Parser;

use graph_layout::GraphLayout;
use io::aut::read_aut;
use log::{debug, info};
use viewer::Viewer;
use slint::{invoke_from_event_loop, RenderingState};

mod graph_layout;
mod viewer;
mod render_text;

#[derive(Parser, Debug)]
#[command(
    name = "Maurice Laveaux",
    about = "A lts viewing tool",
)]
pub struct Cli {
    #[arg(value_name = "FILE")]
    labelled_transition_system: Option<String>,
}

/// Contains all the GUI related state information.
struct GuiState {
    graph_layout: Mutex<GraphLayout>,
    viewer: Mutex<Viewer>,
}

#[derive(Clone, Default)]
pub struct GraphLayoutSettings {
    pub handle_length: f32,
    pub repulsion_strength: f32,
    pub delta: f32,
}

// Initialize a tokio runtime for async calls
#[tokio::main]
async fn main() -> Result<()> {
    // Parse the command line arguments.
    env_logger::init();

    let cli = Cli::parse();
    let state = Arc::new(RwLock::new(None));
    let layout_settings = Arc::new(Mutex::new(GraphLayoutSettings::default()));
    
    // Load the given LTS.
    if let Some(path) = cli.labelled_transition_system {
        debug!("Loading LTS {} ...", path);
        let file = File::open(path)?;

        // TODO: Fix this unwrap and replace it by a ?
        let lts = Arc::new(read_aut(file).unwrap());
        info!("{}", lts);

        *state.write().unwrap() = Some(GuiState {
            graph_layout: Mutex::new(GraphLayout::new(&lts)),
            viewer: Mutex::new(Viewer::new(&lts)),
        });
    };
   
    // Show the UI
    let app = Application::new()?;
    {
        let simulation = simulation.clone();
        let app_weak = app.as_weak();
        app.on_render_simulation(move |width, height, _| {            
            // Render a new frame...
            if let Some(simulation) = simulation.lock().unwrap().deref() {
                if let Some(app) = app_weak.upgrade() {
                    let start = Instant::now();

                    viewer.resize(width as u32, height as u32);
                    let image = viewer.render(simulation, app.global::<Settings>().get_state_radius());

                    debug!("Rendering step took {} ms", (Instant::now() - start).as_millis());
                    image
                } else {
                    Image::default()
                }
            } else {
                Image::default()
            }
        });

        app.on_open_filedialog(move || {
            // Open a file dialog to open a new LTS.
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("", &[".aut"])
                .pick_file() {

                debug!("Loading LTS {} ...", path.to_string_lossy());
                let file = File::open(path).expect("This error should be handled");
                let lts = read_aut(file).unwrap();
                
                // TODO: How to update the mutex/rc.
                //simulation.lock().unwrap() = Some(simulation::Simulation::new(lts));
            }
        });
    }
    
    // Run the simulation on a timer.
    let timer = Timer::default();
    {
        let simulation = simulation.clone();
        let app_weak = app.as_weak();
        
        timer.start(TimerMode::Repeated, std::time::Duration::from_millis(16), move || {
            if let Some(simulation) = simulation.lock().unwrap().deref_mut() {
                if let Some(app) = app_weak.upgrade() {
                    let start = Instant::now();
                    simulation.update(app.global::<Settings>().get_handle_length(), app.global::<Settings>().get_repulsion_strength(), app.global::<Settings>().get_timestep());
                    debug!("Simulation step took {} ms", (Instant::now() - start).as_millis());
                    
                    // Request a redraw when the simulation has progressed.
                    app.global::<Settings>().set_refresh(!app.global::<Settings>().get_refresh());
                }
            }
        });
    }

    app.run()?;

    Ok(())
}