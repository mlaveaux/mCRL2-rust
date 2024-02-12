slint::include_modules!();

use std::{fs::File, ops::{Deref, DerefMut}, rc::Rc, sync::Mutex, time::Instant};

use anyhow::Result;
use clap::Parser;

use io::aut::read_aut;
use log::debug;
use render::Viewer;
use render_text::TextCache;
use slint::{Image, Rgba8Pixel, SharedPixelBuffer, Timer, TimerMode};

mod simulation;
mod render;
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

fn main() -> Result<()> {
    // Parse the command line arguments.
    env_logger::init();

    let cli = Cli::parse();
    
    let label_cache = TextCache::new();

    // Load the given LTS.
    let simulation = if let Some(path) = cli.labelled_transition_system {
        debug!("Loading LTS {} ...", path);
        let file = File::open(path)?;
        let lts = read_aut(file).unwrap();
        debug!("Loading finished");

        Some(simulation::Simulation::new(lts))
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
        let app_weak = app.as_weak();
        app.window().set_rendering_notifier(move |state, _| {
            match state {
                slint::RenderingState::BeforeRendering => {
                    if let Some(app) = app_weak.upgrade() {
                        app.global::<Settings>().set_frame(app.global::<Settings>().get_frame() + 1);
                    }
                },
                _ => {

                }
            }
        }).expect("Cannot set a rendering modifier");
    }

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
        let app = app.as_weak();
        
        timer.start(TimerMode::Repeated, std::time::Duration::from_millis(16), move || {
            if let Some(simulation) = simulation.lock().unwrap().deref_mut() {
                let start = Instant::now();
                simulation.update();
                debug!("Simulation step took {} ms", (Instant::now() - start).as_millis());
                
                // Request a redraw when the simulation has progressed.
                if let Some(app) = app.upgrade() {
                    app.window().request_redraw();
                }
            }
        });
    }

    app.run()?;

    Ok(())
}