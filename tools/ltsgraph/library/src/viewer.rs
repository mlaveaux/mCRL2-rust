use std::sync::Arc;

use cosmic_text::{rustybuzz::ttf_parser::apple_layout::state, Metrics};
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
        draw_actions: bool,
        state_radius: f32,
        view_x: f32,
        view_y: f32,
        screen_x: u32,
        screen_y: u32,
        zoom_level: f32,
        label_text_size: f32,
    ) {
        pixmap.fill(tiny_skia::Color::WHITE);

        // Compute the view transform
        let view_transform = Transform::from_translate(view_x, view_y)
            .post_scale(zoom_level, zoom_level)
            .post_translate(screen_x as f32 / 2.0, screen_y as f32 / 2.0);

        // The color information for states.
        let state_inner_paint = tiny_skia::Paint {
            shader: Shader::SolidColor(tiny_skia::Color::from_rgba8(255, 255, 255, 255)),
            ..Default::default()
        };
        let initial_state_paint = tiny_skia::Paint {
            shader: Shader::SolidColor(tiny_skia::Color::from_rgba8(100, 255, 100, 255)),
            ..Default::default()
        };
        let state_outer = tiny_skia::Paint {
            shader: Shader::SolidColor(tiny_skia::Color::from_rgba8(0, 0, 0, 255)),
            ..Default::default()
        };

        // The color information for edges
        let edge_paint = tiny_skia::Paint::default();

        // The arrow to indicate the direction of the edge, this unwrap should never fail.
        let arrow = {
            let mut builder = tiny_skia::PathBuilder::new();
            builder.line_to(2.0, -5.0);
            builder.line_to(-2.0, -5.0);
            builder.close();
            builder.finish().unwrap()
        };

        // A single circle that is used to render colored states.
        let circle = {
            let mut builder = tiny_skia::PathBuilder::new();
            builder.push_circle(0.0, 0.0, state_radius);
            builder.finish().unwrap()
        };

        // Resize the labels if necessary.
        for buffer in &mut self.labels_cache {
            self.text_cache
                .resize(buffer, Metrics::new(label_text_size, label_text_size));
        }

        // Draw the edges and the arrows on them
        let mut edge_builder = tiny_skia::PathBuilder::new();
        let mut arrow_builder = tiny_skia::PathBuilder::new();

        for (index, state) in self.lts.states.iter().enumerate() {
            let position = self.layout_states[index];

            for (label, to) in &state.outgoing {
                let to_position = self.layout_states[*to];

                edge_builder.move_to(position.x, position.y);
                edge_builder.line_to(to_position.x, to_position.y);

                // Draw the text label
                if draw_actions {
                    let middle = (to_position + position) / 2.0;
                    let buffer = &self.labels_cache[*label];
                    self.text_cache.draw(
                        buffer,
                        pixmap,
                        Transform::from_translate(middle.x, middle.y).post_concat(view_transform),
                    );
                }

                // Draw the arrow of the edge
                if let Some(path) = arrow.clone().transform(
                    Transform::from_translate(0.0, -state_radius - 0.5)
                        .post_rotate(
                            (position - to_position)
                                .angle_between(Vec3::new(0.0, -1.0, 0.0))
                                .to_degrees(),
                        )
                        .post_translate(to_position.x, to_position.y),
                ) {
                    arrow_builder.push_path(&path);
                };
            }
        }

        if let Some(path) = arrow_builder.finish() {
            pixmap.fill_path(
                &path,
                &edge_paint,
                tiny_skia::FillRule::Winding,
                view_transform,
                None,
            );
        }

        // Draw the path for edges.
        if let Some(path) = edge_builder.finish() {
            pixmap.stroke_path(&path, &edge_paint, &Stroke::default(), view_transform, None);
        }

        // Draw the states on top.
        let mut state_path_builder = tiny_skia::PathBuilder::new();

        for (index, position) in self.layout_states.iter().enumerate() {
            if index != self.lts.initial_state {
                state_path_builder.push_circle(position.x, position.y, state_radius);
            } else {
                // Draw the colored states individually
                let transform = Transform::from_translate(position.x, position.y)
                    .post_concat(view_transform);

                pixmap.fill_path(
                    &circle,
                    &initial_state_paint,
                    tiny_skia::FillRule::Winding,
                    transform,
                    None,
                );

                pixmap.stroke_path(&circle, &state_outer, &Stroke::default(), transform, None);
            }
        }

        // Draw the states with an outline.
        if let Some(path) = state_path_builder.finish() {
            pixmap.fill_path(
                &path,
                &state_inner_paint,
                tiny_skia::FillRule::Winding,
                view_transform,
                None,
            );

            pixmap.stroke_path(
                &path,
                &state_outer,
                &Stroke::default(),
                view_transform,
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
        viewer.render(
            &mut PixmapMut::from_bytes(pixel_buffer.data_mut(), 800, 600).unwrap(),
            true,
            5.0,
            0.0,
            0.0,
            800,
            600,
            1.0,
            14.0,
        );
    }
}
