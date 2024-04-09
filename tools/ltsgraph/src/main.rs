// This is a GUI application
#![windows_subsystem = "windows"]

slint::include_modules!();

use std::{
    fs::File,
    ops::Deref,
    path::Path,
    sync::{Arc, Mutex, RwLock},
    thread,
    time::{Duration, Instant},
};

use clap::Parser;

use io::io_aut::read_aut;
use log::{debug, info};
use slint::{invoke_from_event_loop, Image, Rgba8Pixel, SharedPixelBuffer};
use pauseable_thread::PauseableThread;
use ltsgraph_lib::{GraphLayout, Viewer};

mod error_dialog;
mod pauseable_thread;
mod console;

#[derive(Parser, Debug)]
#[command(name = "Maurice Laveaux", about = "A lts viewing tool")]
pub struct Cli {
    #[arg(value_name = "FILE")]
    labelled_transition_system: Option<String>,
}

/// Contains all the GUI related state information.
struct GuiState {
    graph_layout: Mutex<GraphLayout>,
    viewer: Mutex<(Viewer, SharedPixelBuffer<Rgba8Pixel>)>,
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
    pub draw_action_labels: bool,

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
async fn main() -> anyhow::Result<()> {
    // Attach the standard output to the command line.
    let _console = console::init()?;

    // Parse the command line arguments.
    env_logger::init();

    let cli = Cli::parse();

    // Stores the shared state of the GUI components.
    let state = Arc::new(RwLock::new(None::<GuiState>));
    let settings = Arc::new(Mutex::new(GuiSettings::new()));
    let canvas = Arc::new(Mutex::new(SharedPixelBuffer::new(1, 1)));

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

                settings.draw_action_labels = app.global::<Settings>().get_draw_action_labels();
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

        Arc::new(PauseableThread::new(
            "ltsgraph canvas worker",
            move || {
                let settings_clone = settings.lock().unwrap().clone();
                if settings_clone.redraw {

                    if let Some(state) = state.read().unwrap().deref() {
                        // Render a new frame...
                        {
                            let start = Instant::now();
                            let (ref mut viewer, ref mut pixel_buffer) = *state.viewer.lock().unwrap();                           

                            // Resize the canvas when necessary
                            if pixel_buffer.width() != settings_clone.width || pixel_buffer.height() != settings_clone.height {
                                *pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(settings_clone.width, settings_clone.height);
                            }

                            viewer.render(
                                &mut tiny_skia::PixmapMut::from_bytes(pixel_buffer.make_mut_bytes(), settings_clone.width, settings_clone.height).unwrap(),
                                settings_clone.draw_action_labels,
                                settings_clone.state_radius,
                                settings_clone.view_x,
                                settings_clone.view_y,
                                settings_clone.width,
                                settings_clone.height,
                                settings_clone.zoom_level,
                                settings_clone.label_text_size,
                            );

                            debug!(
                                "Rendering {} by {} step took {} ms",
                                settings_clone.width,
                                settings_clone.height,
                                (Instant::now() - start).as_millis()
                            );
                            *canvas.lock().unwrap() = pixel_buffer.clone();

                            // Redraw was performed, what if redraw should happen again during update?
                            settings.lock().unwrap().redraw = false;
                        }
                    } else {
                        return false;
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
                
                return true;
            }
        )?)
    };

    // Run the graph layout algorithm in a separate thread to avoid blocking the UI.
    let layout_handle = {
        let state = state.clone();
        let settings = settings.clone();

        Arc::new(PauseableThread::new("ltsgraph layout worker", move || {
            let start = Instant::now();
            let mut is_stable = true;

            if let Some(state) = state.read().unwrap().deref() {
                // Read the settings and free the lock since otherwise the callback above blocks.
                let settings = settings.lock().unwrap().clone();
                let mut layout = state.graph_layout.lock().unwrap();

                is_stable = layout.update(settings.handle_length, settings.repulsion_strength, settings.delta);
                if is_stable {
                    debug!("Layout is stable!");
                }

                // Copy layout into the view.
                let (ref mut viewer, _) = *state.viewer.lock().unwrap();
                viewer.update(&layout);
            }

            // Request a redraw (if not already in progress).
            settings.lock().unwrap().redraw = true;

            // Keep at least 16 milliseconds between two layout runs.
            let duration = Instant::now() - start;
            debug!("Layout step took {} ms", duration.as_millis());
            thread::sleep(Duration::from_millis(16).saturating_sub(duration));

            // If stable pause the thread.
            return !is_stable;
        })?)
    };

    {
        // When the simulation is toggled enable the layout thread.
        let layout_handle = layout_handle.clone();
        app.on_run_simulation(move |enabled| {
            if enabled {
                layout_handle.resume();
            } else {
                layout_handle.pause();
            }
        })
    }
    
    // Load an LTS from the given path and updates the state.
    let load_lts = {
        let state = state.clone();
        let layout_handle = layout_handle.clone();
        let canvas_handle = canvas_handle.clone();

        move |path: &Path| {
            debug!("Loading LTS {} ...", path.to_string_lossy());

            match File::open(path) {
                Ok(file) => {
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
                                viewer: Mutex::new((viewer, SharedPixelBuffer::new(1, 1))),
                            });

                            // Enable the layout and rendering threads.
                            layout_handle.resume();
                            canvas_handle.resume();
                        }
                        Err(x) => {
                            error_dialog::show_error_dialog("Failed to load LTS!", &format!("{}", x));
                        }
                    }
                },
                Err(x) => {
                    error_dialog::show_error_dialog("Failed to load LTS!", &format!("{}", x));
                }
            }
        }
    };
    
    {
        // Open the file dialog and load another LTS if necessary.
        let load_lts = load_lts.clone();
        app.on_open_filedialog(move || {
            // Open a file dialog to open a new LTS.
            if let Some(path) = rfd::FileDialog::new().add_filter("", &["aut"]).pick_file() {
                load_lts(&path);
            }
        });
    }
    
    // Loads the LTS given on the command line.
    if let Some(path) = &cli.labelled_transition_system {
        load_lts(Path::new(path));
    }

    app.run()?;

    // Stop the layout and quit.
    layout_handle.stop();
    canvas_handle.stop();

    Ok(())
}
