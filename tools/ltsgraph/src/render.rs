use tiny_skia::{Shader, Stroke, Transform};

use crate::simulation::Simulation;

pub fn render(pixmap: &mut tiny_skia::PixmapMut, simulation: &Simulation) {

    let mut paint = tiny_skia::Paint::default();
    
    let circle = tiny_skia::PathBuilder::from_circle(5.0, 5.0, 15.0).unwrap();
    let mut edge_builder = tiny_skia::PathBuilder::new();
    
    // Draw the states.
    for (index, state) in simulation.lts.states.iter().enumerate() {
        let position = simulation.states_simulation[index].position;

        // Draw the state.
        paint.shader = Shader::SolidColor(tiny_skia::Color::from_rgba8(50, 127, 150, 255));
        pixmap.fill_path(&circle, &paint, tiny_skia::FillRule::Winding, Transform::from_translate(position.x, position.y), None);

        // Draw the edges
        for (_, to) in &state.outgoing {
            let to_position = simulation.states_simulation[*to].position;

            edge_builder.move_to(position.x, position.y);
            edge_builder.line_to(to_position.x, to_position.y);
        }

    }

    // Draw the edges
    pixmap.stroke_path(&edge_builder.finish().unwrap(), &paint, &Stroke::default(), Transform::default(), None);


}