use std::sync::Arc;

use cosmic_text::Metrics;
use glam::Vec3;
use io::aut::LTS;
use slint::{Rgba8Pixel, SharedPixelBuffer};
use tiny_skia::{Path, PathBuilder, Shader, Stroke, Transform};

use crate::{graph_layout::GraphLayout, text_cache::TextCache};

pub struct Viewer {
    /// The slint pixel buffer used for rendering.
    pixel_buffer: SharedPixelBuffer::<Rgba8Pixel>,
    
    /// A cache used to cache strings and font information.
    text_cache: TextCache,

    /// A buffer for transition labels.
    labels_cache: Vec<(cosmic_text::Buffer, Path)>,

    /// The underlying LTS being displayed.
    lts: Arc<LTS>,

    /// Stores a local copy of the state positions.
    layout_states: Vec<Vec3>,
}

impl Viewer {
    pub fn new(lts: &Arc<LTS>) -> Viewer {

        let mut text_cache = TextCache::new();
        let mut labels_cache = vec![];

        for label in &lts.labels {
            // Create text elements for all labels that we are going to render.
            let buffer = text_cache.create_buffer(label, Metrics::new(12.0, 12.0));
            
            // Draw the label of the edge          
            let mut text_builder = PathBuilder::new();       
            text_cache.draw(&buffer, &mut text_builder);

            // Put it in the label cache.
            labels_cache.push((buffer, text_builder.finish().unwrap()));
        }

        // Initialize the layout information for the states.
        let mut layout_states = Vec::with_capacity(lts.states.len());
        layout_states.resize(lts.states.len(), Default::default());

        Viewer {
            text_cache,
            labels_cache,        
            pixel_buffer: SharedPixelBuffer::<Rgba8Pixel>::new(1, 1),
            lts: lts.clone(),
            layout_states
        }
    }

    /// Resize the output image dimensions when necessary.
    pub fn on_resize(&mut self, width: u32, height: u32) {
        if self.pixel_buffer.width() != width || self.pixel_buffer.height() != height {
            self.pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);
        }
    }

    /// Update the state of the viewer with the given graph layout.
    pub fn update(&mut self, layout: &GraphLayout) {
        self.layout_states = layout.layout_states.iter().map(|state| {
            state.position
        }).collect();
    }

    /// Render the current state of the simulation into the pixmap.
    pub fn render(&mut self, state_radius: f32) -> SharedPixelBuffer::<Rgba8Pixel> {
        
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

        for (index, state) in self.lts.states.iter().enumerate() {
            let position = self.layout_states[index];

            for (label, to) in &state.outgoing {
                let to_position = self.layout_states[*to];

                edge_builder.move_to(position.x, position.y);
                edge_builder.line_to(to_position.x, to_position.y);

                // Draw the text label              
                let middle = (to_position + position) / 2.0;  
                pixmap.fill_path(&self.labels_cache[*label].1, &state_outer, tiny_skia::FillRule::Winding, Transform::from_translate(middle.x, middle.y), None);
             }
        }

        // Draw the path for edges.
        if let Some(path) = edge_builder.finish() {
            pixmap.stroke_path(&path, &edge_paint, &Stroke::default(), Transform::default(), None);
        }

        // Draw the states on top.
        for position in &self.layout_states {
            // Draw the state.
            pixmap.fill_path(&state_circle, &state_inner, tiny_skia::FillRule::Winding, Transform::from_translate(position.x, position.y), None);
            pixmap.stroke_path(&state_circle, &state_outer, &Stroke::default(), Transform::from_translate(position.x, position.y), None);
        }

        self.pixel_buffer.clone()
    }
}