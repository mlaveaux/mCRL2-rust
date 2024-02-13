use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use tiny_skia::{Shader, Stroke, Transform};

use crate::{graph_layout::GraphLayout, render_text::TextCache};

pub struct Viewer {
    /// The slint pixel buffer used for rendering.
    pixel_buffer: SharedPixelBuffer::<Rgba8Pixel>,
    
    /// A cache used to cache strings and font information.
    label_cache: TextCache,

    /// A buffer for every single label in the set.
    labels_cache: Vec<cosmic_text::CacheKey>,
}

impl Viewer {
    pub fn new() -> Viewer {
        Viewer {
            label_cache: TextCache::new(),
        
            pixel_buffer: SharedPixelBuffer::<Rgba8Pixel>::new(1, 1),

            labels_cache: vec![],
        }
    }

    /// Resize the view when necessary.
    pub fn resize(&mut self, width: u32, height: u32) {
        if self.pixel_buffer.width() != width || self.pixel_buffer.height() != height {
            self.pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);
        }
    }

    /// Render the current state of the simulation into the pixmap.
    pub fn render(&mut self, simulation: &GraphLayout, state_radius: f32) -> Image {
        // Create text elements for all labels that we are going to render.
        //let buffer = self.label_cache.create_buffer(Metrics::new(12.0, 12.0));
         
        // Clear the current pixel buffer.
        let width = self.pixel_buffer.width();
        let height = self.pixel_buffer.height();
        let mut pixmap = tiny_skia::PixmapMut::from_bytes(
            self.pixel_buffer.make_mut_bytes(), width, height
        ).unwrap();
        pixmap.fill(tiny_skia::Color::TRANSPARENT);
        
        // The information for states.
        let mut state_inner = tiny_skia::Paint::default();
        state_inner.shader = Shader::SolidColor(tiny_skia::Color::from_rgba8(255, 255, 255, 255));

        let mut state_outer = tiny_skia::Paint::default();
        state_outer.shader = Shader::SolidColor(tiny_skia::Color::from_rgba8(0, 0, 0, 255));

        let state_circle = tiny_skia::PathBuilder::from_circle(0.0, 0.0, state_radius).unwrap();

        // The information for edges
        let edge_paint = tiny_skia::Paint::default();

        // Draw the edges.
        let mut edge_builder = tiny_skia::PathBuilder::new();
        for (index, state) in simulation.lts.states.iter().enumerate() {
            let position = simulation.states_simulation[index].position;

            for (_, to) in &state.outgoing {
                let to_position = simulation.states_simulation[*to].position;

                 edge_builder.move_to(position.x, position.y);
                 edge_builder.line_to(to_position.x, to_position.y);
             }
        }

        if let Some(path) = edge_builder.finish() {
            pixmap.stroke_path(&path, &edge_paint, &Stroke::default(), Transform::default(), None);
        }

        // Draw the states on top.
        for state in &simulation.states_simulation {
            // Draw the state.
            pixmap.fill_path(&state_circle, &state_inner, tiny_skia::FillRule::Winding, Transform::from_translate(state.position.x, state.position.y), None);
            pixmap.stroke_path(&state_circle, &state_outer, &Stroke::default(), Transform::from_translate(state.position.x, state.position.y), None);
        }

        Image::from_rgba8_premultiplied(self.pixel_buffer.clone())
    }
}