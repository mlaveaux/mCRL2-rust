slint::include_modules!();

use std::{
    fs::File,
    ops::Deref,
    path::Path,
    sync::{Arc, Mutex, RwLock},
    thread,
    time::{Duration, Instant},
};

use anyhow::Result;
use clap::Parser;

use graph_layout::GraphLayout;
use io::io_aut::read_aut;
use log::{debug, info};
use slint::{invoke_from_event_loop, Image, SharedPixelBuffer};
use viewer::Viewer;
use pauseable_thread::PauseableThread;

mod error_dialog;
mod graph_layout;
mod text_cache;
mod pauseable_thread;
mod viewer;

#[derive(Parser, Debug)]
#[command(name = "Maurice Laveaux", about = "A lts viewing tool")]
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
    pub label_text_size: f32,

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
                }
                Err(x) => {
                    error_dialog::show_error_dialog("Failed to load LTS!", &format!("{}", x));
                }
            }
        }
    };

    // Loads the given LTS.
    if let Some(path) = &cli.labelled_transition_system {
        load_lts(Path::new(path));
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
                settings.label_text_size = app.global::<Settings>().get_label_text_height();
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
                debug!("Request redraw");
                settings.redraw = true;
            }

            debug!("Updating canvas");
            Image::from_rgba8_premultiplied(buffer)
        });
    }

    {
        // Open the file dialog and load another LTS if necessary.
        app.on_open_filedialog(move || {
            // Open a file dialog to open a new LTS.
            if let Some(path) = rfd::FileDialog::new().add_filter("", &["aut"]).pick_file() {
                load_lts(&path);
            }
        });
    }

    {
        let settings = settings.clone();
        app.on_request_redraw(move || {
            debug!("Request redraw");            
            settings.lock().unwrap().redraw = true;
        })
    }

    // Render the view continuously, but only update the canvas when necessary
    let canvas_handle = {
        let state = state.clone();
        let app_weak: slint::Weak<Application> = app.as_weak();
        let settings = settings.clone();

        PauseableThread::new(
            "ltsgraph canvas worker",
            move || {
                let settings_clone = settings.lock().unwrap().clone();
                if settings_clone.redraw {
                    if let Some(state) = state.read().unwrap().deref() {
                        // Render a new frame...
                        {
                            let start = Instant::now();
                            let mut viewer = state.viewer.lock().unwrap();
                            viewer.on_resize(settings_clone.width, settings_clone.height);
                            let image = viewer.render(
                                settings_clone.state_radius,
                                settings_clone.view_x,
                                settings_clone.view_y,
                                settings_clone.zoom_level,
                                settings_clone.label_text_size,
                            );

                            debug!(
                                "Rendering {} by {} step took {} ms",
                                settings_clone.width,
                                settings_clone.height,
                                (Instant::now() - start).as_millis()
                            );
                            *canvas.lock().unwrap() = image;

                            // Redraw was performed, what if redraw should happen again during update?
                            settings.lock().unwrap().redraw = false;
                        }
                    }

                    // Request a redraw when the canvas has been updated.
                    let app_weak = app_weak.clone();
                    invoke_from_event_loop(move || {
                        if let Some(app) = app_weak.upgrade() {
                            // Update the canvas
                            app.global::<Settings>()
                                .set_refresh(!app.global::<Settings>().get_refresh());
                        };
                    })
                    .unwrap();
                }
            }
        )?
    };

    // Run the graph layout algorithm in a separate thread to avoid blocking the UI.
    let layout_handle = {
        let state = state.clone();
        let settings = settings.clone();

        Arc::new(PauseableThread::new("ltsgraph layout worker", move || {
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
        })?)
    };

    {
        let layout_handle = layout_handle.clone();
        app.on_run_simulation(move |enabled| {
            if enabled {
                layout_handle.resume();
            } else {
                layout_handle.pause();
            }
        })
    }

    app.run()?;

    // Stop the layout and quit.
    layout_handle.stop();
    canvas_handle.stop();

    Ok(())
}
