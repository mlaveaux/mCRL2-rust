use cosmic_text::{Attrs, Color, FontSystem, SwashCache, Buffer, Metrics, Shaping};

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
        
        // Text metrics indicate the font size and line height of a buffer
        let metrics = Metrics::new(14.0, 20.0);
        
        // A Buffer provides shaping and layout for a UTF-8 string, create one per text widget
        let mut buffer = Buffer::new(&mut font_system, metrics);
        
        // Borrow buffer together with the font system for more convenient method calls
        let mut buffer = buffer.borrow_with(&mut font_system);
        
        // Set a size for the text buffer, in pixels
        buffer.set_size(80.0, 25.0);
        
        // Attributes indicate what font to choose
        let attrs = Attrs::new();
        
        // Add some text!
        buffer.set_text("Hello, Rust! 🦀\n", attrs, Shaping::Advanced);
        
        // Perform shaping as desired
        buffer.shape_until_scroll(true);
                   
        // Draw the buffer (for performance, instead use SwashCache directly)
        TextCache {
            font_system,
            swash_cache
        }
    }

    /// Draw the given cached text at the given location.
    pub fn draw(&mut self, path_builder: &mut tiny_skia::PathBuilder, key: cosmic_text::CacheKey, color: tiny_skia::Color) {


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
                        //path_builder.move

                    },
                    _ => {

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


            