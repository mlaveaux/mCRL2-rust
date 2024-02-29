use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, SwashCache};
use tiny_skia::{PathBuilder, PixmapMut, PixmapPaint, Stroke, Transform};

pub struct TextCache {
    /// A FontSystem provides access to detected system fonts, create one per application
    font_system: FontSystem,

    /// A SwashCache stores rasterized glyphs, create one per application
    swash_cache: SwashCache,
}

impl TextCache {
    pub fn new() -> TextCache {  
        let mut font_system = FontSystem::new();
        let mut swash_cache = SwashCache::new();
                   
        TextCache {
            font_system,
            swash_cache
        }
    }

    /// Front metrics indicate the font size and line height of a buffer
    pub fn create_buffer(&mut self, text: &String, font_metrics: Metrics) -> Buffer {      
                
        // A Buffer provides shaping and layout for a UTF-8 string, create one per text widget
        let mut buffer = Buffer::new(&mut self.font_system, font_metrics);
                
        // Set a size for the text buffer, in pixels
        buffer.set_size(&mut self.font_system, 80.0, 25.0);
        
        // Attributes indicate what font to choose.
        let attrs = Attrs::new();

        // Add some text!
        buffer.set_text(&mut self.font_system, text, attrs, Shaping::Advanced);

        // Perform shaping as desired
        buffer.shape_until_scroll(&mut self.font_system, true);

        buffer
    }

    /// Draw the given cached text at the given location.
    pub fn draw(&mut self, buffer: &Buffer, pixmap: &mut PixmapMut) {
        let paint = tiny_skia::Paint::default();

        // Draw the buffer        
        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical_glyph = glyph.physical((0., 0.), 1.0);

                // let glyph_color = match glyph.color_opt {
                //     Some(some) => some,
                //     None => Color,
                // };

                // Try to get the font outline, which we can draw directly with tiny-skia.
                if let Some(outline) = self.swash_cache.get_outline_commands(&mut self.font_system, physical_glyph.cache_key) {
                    let mut path_builder = PathBuilder::new();

                    for command in outline {
                        match *command {
                            cosmic_text::Command::MoveTo(p0) => {
                                path_builder.move_to(p0.x, p0.y);
                            },
                            cosmic_text::Command::LineTo(p0) => {
                                path_builder.line_to(p0.x, p0.y);
                            },
                            cosmic_text::Command::CurveTo(p0, p1, p2) => {
                                path_builder.cubic_to(p0.x, p0.y, p1.x, p1.y, p2.x, p2.y);
                            },
                            cosmic_text::Command::Close => {
                                path_builder.close();
                            },
                            cosmic_text::Command::QuadTo(p0, p1) => {
                                path_builder.quad_to(p0.x, p0.y, p1.x, p1.y);
                            }
                        }
                    }

                    pixmap.stroke_path(&path_builder.finish().unwrap(), &paint, &Stroke::default(), Transform::from_translate(physical_glyph.x as f32, physical_glyph.y as f32), None);
                } else {
                    // Otherwise render the image using skia.
                    if let Some(image) = self.swash_cache.get_image(&mut self.font_system, physical_glyph.cache_key) {
                        let mut data = image.data.clone();
                        let pixmap_image = PixmapMut::from_bytes(&mut data, image.placement.width, image.placement.height);

                        pixmap.draw_pixmap(0, 0, pixmap_image.unwrap().as_ref(), &PixmapPaint::default(), Transform::default(), None);  
                    };
                }
            }
        }
    }
}


            