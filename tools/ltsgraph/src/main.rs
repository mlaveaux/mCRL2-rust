slint::include_modules!();

use std::{fs::File, ops::{Deref, DerefMut}, rc::Rc, sync::Mutex, time::Instant};

use anyhow::Result;
use clap::Parser;

use io::aut::read_aut;
use log::{debug, info};
use render_skia::Viewer;
use slint::{Image, Timer, TimerMode};

mod graph_layout;
mod render_skia;
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

// Initialize a tokio runtime for async calls
#[tokio::main]
async fn main() -> Result<()> {
    // Parse the command line arguments.
    env_logger::init();

    let cli = Cli::parse();
    
    // Load the given LTS.
    let simulation = if let Some(path) = cli.labelled_transition_system {
        debug!("Loading LTS {} ...", path);
        let file = File::open(path)?;

        // TODO: Fix this unwrap and replace it by a ?
        let lts = read_aut(file).unwrap();
        info!("{}", lts);

        Some(graph_layout::GraphLayout::new(lts))
    } else {
        None
    };

    // Simulation state.
    let simulation = Rc::new(Mutex::new(simulation));

    // Viewer state.
    let mut viewer = Viewer::new();
    
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