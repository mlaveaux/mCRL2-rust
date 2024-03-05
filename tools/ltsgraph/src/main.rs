slint::include_modules!();

use std::{fs::File, ops::Deref, path::Path, sync::{atomic::AtomicBool, Arc, Mutex, RwLock}, thread, time::{Duration, Instant}};

use anyhow::Result;
use clap::Parser;

use graph_layout::GraphLayout;
use io::io_aut::read_aut;
use log::{debug, info};
use viewer::Viewer;
use slint::{invoke_from_event_loop, Image, SharedPixelBuffer};

mod graph_layout;
mod viewer;
mod text_cache;
mod error_dialog;

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
pub struct GuiSettings {

    // Layout related settings
    pub handle_length: f32,
    pub repulsion_strength: f32,
    pub delta: f32,

    // View related settings
    pub width: u32,
    pub height: u32,
    pub state_radius: f32,

    pub redraw: bool,

    pub zoom_level: f32,
    pub view_x: f32,
    pub view_y: f32,
}

impl GuiSettings {
    pub fn new() -> GuiSettings {
        GuiSettings {
            width: 1,
            height: 1,
            redraw: false,
            zoom_level: 1.0,
            view_x: 500.0,
            view_y: 500.0,
            ..Default::default()
        }
    }
}

// Initialize a tokio runtime for async calls
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Parse the command line arguments.
    env_logger::init();

    let cli = Cli::parse();

    // Stores the shared state of the GUI components.
    let state = Arc::new(RwLock::new(None));
    let settings = Arc::new(Mutex::new(GuiSettings::new()));
    let canvas = Arc::new(Mutex::new(SharedPixelBuffer::new(1, 1)));
    
    // Load an LTS from the given path and updates the state.
    let load_lts = {
        let state = state.clone();

        move |path: &Path| {
            debug!("Loading LTS {} ...", path.to_string_lossy());
            let file = File::open(path).unwrap();

            match read_aut(file) {
                Ok(lts) => {
                    let lts = Arc::new(lts);
                    info!("{}", lts);

                    // Create the layout and viewer separately to make the initial state sensible.
                    let layout = GraphLayout::new(&lts);
                    let mut viewer = Viewer::new(&lts);
                    viewer.update(&layout);

                    *state.write().unwrap() = Some(GuiState {
                        graph_layout: Mutex::new(layout),
                        viewer: Mutex::new(viewer),
                    });                    
                },
                Err(x) => {
                    error_dialog::show_error_dialog("Failed to load LTS!", &format!("{}", x));
                }
            }

        }
    };

    // Loads the given LTS.
    if let Some(path) = &cli.labelled_transition_system {
        load_lts(&Path::new(path));
    };
   
    // Show the UI
    let app = Application::new()?;

    {
        let app_weak = app.as_weak();
        let settings = settings.clone();

        app.on_settings_changed(move || {        
            // Request the settings for the next simulation tick.
            if let Some(app) = app_weak.upgrade() {
                let mut settings = settings.lock().unwrap();
                settings.handle_length = app.global::<Settings>().get_handle_length();
                settings.repulsion_strength = app.global::<Settings>().get_repulsion_strength();
                settings.delta = app.global::<Settings>().get_timestep();
                settings.state_radius = app.global::<Settings>().get_state_radius();
                
                settings.zoom_level = app.global::<Settings>().get_zoom_level();
                settings.view_x = app.global::<Settings>().get_view_x();
                settings.view_y = app.global::<Settings>().get_view_y();
            }
        });
    };

    // Trigger it once to set the default values.
    app.invoke_settings_changed();
    
    {
        let canvas = canvas.clone();
        let settings = settings.clone();

        // Simply return the current canvas, can be updated in the meantime.
        app.on_update_canvas(move |width, height, _| {   
            let mut settings = settings.lock().unwrap();
            settings.width = width as u32;
            settings.height = height as u32;

            let buffer = canvas.lock().unwrap().clone();

            if buffer.width() != settings.width || buffer.height() != settings.height {
                // Request another redraw when the size has changed.
                settings.redraw = true;
            }

            debug!("Redraw canvas");
            Image::from_rgba8_premultiplied(buffer)
        });
    }

    {        
        // Open the file dialog and load another LTS if necessary.
        app.on_open_filedialog(move || {
            // Open a file dialog to open a new LTS.
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("", &["aut"])
                .pick_file() {

                load_lts(&path);
            }
        });
    }

    // Render the view continuously, but only update the canvas when necessary
    let run_canvas = Arc::new(AtomicBool::new(true));

    let canvas_handle = {
        let run_canvas = run_canvas.clone();
        let state = state.clone();
        let app_weak: slint::Weak<Application> = app.as_weak();
        let settings = settings.clone();

        thread::Builder::new().name("ltsgraph canvas worker".to_string()).spawn(move || {
            while run_canvas.load(std::sync::atomic::Ordering::Relaxed) {

                let settings = settings.lock().unwrap().clone();
                if settings.redraw {
                    if let Some(state) = state.read().unwrap().deref() {                                
                        // Render a new frame...
                        {
                            let start = Instant::now();
                            let mut viewer = state.viewer.lock().unwrap();
                            viewer.on_resize(settings.width, settings.height);
                            let image = viewer.render(settings.state_radius, settings.view_x, settings.view_y, settings.zoom_level);

                            debug!("Rendering step took {} ms", (Instant::now() - start).as_millis());
                            *canvas.lock().unwrap() = image;
                        }
                    }

                    // Request a redraw when the canvas has been updated.
                    let app_weak = app_weak.clone();
                    invoke_from_event_loop( move || {
                        if let Some(app) = app_weak.upgrade() {
                            // Update the canvas
                            app.global::<Settings>().set_refresh(!app.global::<Settings>().get_refresh());
                        };
                    }).unwrap();
                }
            }
        })?
    };
    
    // Run the graph layout algorithm in a separate thread to avoid blocking the UI.
    let run_layout = Arc::new(AtomicBool::new(true));

    let layout_handle = {
        let state = state.clone();
        let settings = settings.clone();
        let run_layout = run_layout.clone();

         thread::Builder::new().name("ltsgraph layout worker".to_string()).spawn(move || {
            while run_layout.load(std::sync::atomic::Ordering::Relaxed) {
                let start = Instant::now();

                if let Some(state) = state.read().unwrap().deref() {
                    // Read the settings and free the lock since otherwise the callback above blocks.
                    let settings = settings.lock().unwrap().clone();
                    let mut layout = state.graph_layout.lock().unwrap();

                    layout.update(settings.handle_length, settings.repulsion_strength, settings.delta);
                                        
                    // Copy layout into the view.
                    let mut viewer = state.viewer.lock().unwrap();
                    viewer.update(&layout);
                }
                
                // Request a redraw (if not already in progress).
                settings.lock().unwrap().redraw = true;

                // Keep at least 16 milliseconds between two layout runs.
                let duration = Instant::now() - start;
                debug!("Layout step took {} ms", duration.as_millis());
                thread::sleep(Duration::from_millis(100).saturating_sub(duration));
            }
        })?
    };

    app.run()?;

    // Stop the layout and quit.
    run_layout.store(false, std::sync::atomic::Ordering::Relaxed);
    run_canvas.store(false, std::sync::atomic::Ordering::Relaxed);

    layout_handle.join().unwrap();
    canvas_handle.join().unwrap();

    Ok(())
}