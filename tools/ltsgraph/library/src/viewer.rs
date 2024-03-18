use std::sync::Arc;

use cosmic_text::Metrics;
use glam::Vec3;
use io::LabelledTransitionSystem;
use tiny_skia::{Shader, Stroke, Transform};

use crate::{graph_layout::GraphLayout, text_cache::TextCache};

pub struct Viewer {
    /// A cache used to cache strings and font information.
    text_cache: TextCache,

    /// A buffer for transition labels.
    labels_cache: Vec<cosmic_text::Buffer>,

    /// The underlying LTS being displayed.
    lts: Arc<LabelledTransitionSystem>,

    /// Stores a local copy of the state positions.
    layout_states: Vec<Vec3>,
}

impl Viewer {
    pub fn new(lts: &Arc<LabelledTransitionSystem>) -> Viewer {
        let mut text_cache = TextCache::new();
        let mut labels_cache = vec![];

        for label in &lts.labels {
            // Create text elements for all labels that we are going to render.
            let buffer = text_cache.create_buffer(label, Metrics::new(12.0, 12.0));

            // Put it in the label cache.
            labels_cache.push(buffer);
        }

        // Initialize the layout information for the states.
        let mut layout_states = Vec::with_capacity(lts.states.len());
        layout_states.resize(lts.states.len(), Default::default());

        Viewer {
            text_cache,
            labels_cache,
            lts: lts.clone(),
            layout_states,
        }
    }

    /// Update the state of the viewer with the given graph layout.
    pub fn update(&mut self, layout: &GraphLayout) {
        self.layout_states = layout
            .layout_states
            .iter()
            .map(|state| state.position)
            .collect();
    }

    /// Render the current state of the simulation into the pixmap.
    pub fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        state_radius: f32,
        view_x: f32,
        view_y: f32,
        zoom_level: f32,
        label_text_size: f32,
    ) {
        pixmap.fill(tiny_skia::Color::WHITE);

        // Compute the view transform
        let view_transform = Transform::from_translate(view_x, view_y).post_scale(zoom_level, zoom_level);

        // The information for states.
        let state_inner = tiny_skia::Paint {
            shader: Shader::SolidColor(tiny_skia::Color::from_rgba8(255, 255, 255, 255)),
            ..Default::default()
        };
        let state_outer = tiny_skia::Paint {
            shader: Shader::SolidColor(tiny_skia::Color::from_rgba8(0, 0, 0, 255)),
            ..Default::default()
        };

        let state_circle = tiny_skia::PathBuilder::from_circle(0.0, 0.0, state_radius).unwrap();

        // The information for edges
        let edge_paint = tiny_skia::Paint::default();

        // Resize the labels if necessary.
        for buffer in &mut self.labels_cache {
            self.text_cache
                .resize(buffer, Metrics::new(label_text_size, label_text_size));
        }

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
                self.text_cache.draw(
                    &self.labels_cache[*label],
                    pixmap,
                    Transform::from_translate(middle.x - self.labels_cache[*label].size().0 / 2.0, middle.y)
                        .post_concat(view_transform),
                );
            }
        }

        // Draw the path for edges.
        if let Some(path) = edge_builder.finish() {
            pixmap.stroke_path(
                &path,
                &edge_paint,
                &Stroke::default(),
                view_transform,
                None,
            );
        }

        // Draw the states on top.
        for position in &self.layout_states {
            // Draw the state.
            pixmap.fill_path(
                &state_circle,
                &state_inner,
                tiny_skia::FillRule::Winding,
                Transform::from_translate(position.x, position.y)
                    .post_concat(view_transform),
                None,
            );
            pixmap.stroke_path(
                &state_circle,
                &state_outer,
                &Stroke::default(),
                Transform::from_translate(position.x, position.y)
                    .post_concat(view_transform),
                None,
            );
        }
    }
}


#[cfg(test)]
mod tests {
    use io::io_aut::read_aut;
    use tiny_skia::{Pixmap, PixmapMut};

    use super::*;

    #[test]
    fn test_viewer() {
        // Render a single from the alternating bit protocol with some settings.
        let file = include_str!("../../../../examples/lts/abp.aut");
        let lts = Arc::new(read_aut(file.as_bytes()).unwrap());

        let mut viewer = Viewer::new(&lts);

        let mut pixel_buffer = Pixmap::new(800, 600).unwrap();
        viewer.render(&mut PixmapMut::from_bytes(pixel_buffer.data_mut(), 800, 600).unwrap(), 5.0, 0.0, 0.0, 1.0, 14.0);
    }
}