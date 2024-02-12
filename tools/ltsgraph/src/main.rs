slint::include_modules!();

use std::{fs::File, ops::{Deref, DerefMut}, rc::Rc, sync::Mutex};

use anyhow::Result;
use clap::Parser;

use io::aut::read_aut;
use log::debug;
use slint::{Image, Rgba8Pixel, SharedPixelBuffer, Timer, TimerMode};

mod simulation;
mod render;

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

    // Load the given LTS.
    let simulation = if let Some(path) = cli.labelled_transition_system {
        debug!("Loading LTS {} ...", path);
        let file = File::open(path)?;
        let lts = read_aut(file).unwrap();
        Some(simulation::Simulation::new(lts))
    } else {
        None
    };

    let simulation = Rc::new(Mutex::new(simulation));
    
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
        app.on_render_simulation(move |width, height, _| {            
            // Render a new frame...
            let mut pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width as u32, height as u32);
            
            // Clear the current pixel buffer.
            let width = pixel_buffer.width();
            let height = pixel_buffer.height();
            let mut pixmap = tiny_skia::PixmapMut::from_bytes(
                pixel_buffer.make_mut_bytes(), width, height
            ).unwrap();
            pixmap.fill(tiny_skia::Color::TRANSPARENT);

            if let Some(simulation) = simulation.lock().unwrap().deref() {
                render::render(&mut pixmap, simulation);
            }
            
            Image::from_rgba8_premultiplied(pixel_buffer.clone())
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
                simulation.update();
                
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