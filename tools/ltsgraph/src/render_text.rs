use cosmic_text::{Attrs, FontSystem, SwashCache, Buffer, Metrics};

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
    pub fn create_buffer(&mut self, font_metrics: Metrics) -> Buffer {      
                
        // A Buffer provides shaping and layout for a UTF-8 string, create one per text widget
        let mut buffer = Buffer::new(&mut self.font_system, font_metrics);
                
        // Set a size for the text buffer, in pixels
        //buffer.set_size(80.0, 25.0);
        
        // Attributes indicate what font to choose
        let attrs = Attrs::new();

        buffer
    }

    /// Draw the given cached text at the given location.
    pub fn draw(&mut self, path_builder: &mut tiny_skia::PathBuilder, key: cosmic_text::CacheKey, color: tiny_skia::Color) {
        // Draw the buffer (for performance, instead use SwashCache directly)

        // Try to get the font outline, which we can draw directly with tiny-skia.
        if let Some(outline) = self.swash_cache.get_outline_commands(&mut self.font_system, key) {
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
        } else {
            // Otherwise render the image using skia.
            if let Some(image) = self.swash_cache.get_image(&mut self.font_system, key) {
                                
            };
        }
    }
}


            