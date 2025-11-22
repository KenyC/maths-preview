use rex::{font::common::GlyphId, Backend, Cursor, FontBackend, GraphicsBackend};
use std::collections::HashMap;

#[cfg(feature = "ttf-parser")]
pub mod ttfparser;

struct TextAsText {
    glyph_to_char_table : HashMap<GlyphId, char>,
    font_name : Box<str>,
}

pub struct SvgContext {
    content : String,
    color_stack : Vec<rex::RGBA>,
    glyph_as_text : Option<TextAsText>,
}

impl SvgContext {
    const DEFAULT_COLOR : rex::RGBA = rex::RGBA(0x00, 0x00, 0x00, 0xff);

    pub fn new() -> Self { 
        Self { content : String::new(), color_stack: Vec::new(), glyph_as_text : None } 
    }

    pub fn finalize(self, x : f64, y : f64, width : f64, height : f64) -> String {
        format!(r#"<svg viewBox="{} {} {} {}">{}</svg>"#, 
            x, y,
            width, height, 
            self.content
        )
    }

    /// Enables the "text as text" feature which allows rendering glyphs as text instead of curves in the SVG directly
    pub fn glyph_as_text(
        &mut self, 
        glyph_to_char_table : HashMap<GlyphId, char>, 
        font_name : &str
    ) {
        self.glyph_as_text = Some(TextAsText {
            glyph_to_char_table,
            font_name : font_name.into(),
        });
    }

    fn current_color(&self) -> rex::RGBA {
        self.color_stack.last().cloned().unwrap_or(Self::DEFAULT_COLOR)
    }


}

impl<T : GivesOutline> Backend<T> for SvgContext {}


impl GraphicsBackend for SvgContext {
    fn rule(&mut self, pos: Cursor, width: f64, height: f64) {
        let Cursor { x, y } = pos;
        self.content.push_str(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" />"#,
            x, y, width, height
            ));
    }

    fn begin_color(&mut self, color: rex::RGBA) {
        self.color_stack.push(color);
    }

    fn end_color(&mut self) {
        self.color_stack.pop();
    }
}

pub trait OutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32);
    fn line_to(&mut self, x: f32, y: f32);
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32);
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32);
    fn close(&mut self);
}

pub trait GivesOutline {
    fn outline_glyph(&self, glyph_id : GlyphId, builder : &mut impl OutlineBuilder);
    fn font_scale(&self) -> (f32, f32);
}

impl<T : GivesOutline> FontBackend<T> for SvgContext {
    fn symbol(&mut self, pos: Cursor, gid: rex::font::common::GlyphId, scale: f64, font: &T) {
        let mut path_string = None;
        if let Some(TextAsText { glyph_to_char_table, font_name }) = &self.glyph_as_text {
            if let Some(character) = glyph_to_char_table.get(&gid) {
                path_string = Some(render_symbol_as_text(pos, scale, *character, &font_name, self.current_color()));
            }
        }

        let path_string = path_string.unwrap_or_else(|| render_symbol_as_curve(font, pos, scale, gid, self.current_color()));
        self.content.push_str(&path_string);
    }
}

fn render_symbol_as_text(pos: Cursor, scale: f64, character: char, font_name: &str, color: rex::RGBA) -> String {
    format!(r#"<text x="{}" y="{}" font-family="{}" font-size="{}px" {}>&#x{:X};</text>"#, 
        pos.x,
        pos.y,
        font_name,
        scale,
        to_xml_color(color),
        character as u32,
    )
}

fn to_xml_color(color : rex::RGBA) -> String {
    let rex::RGBA(r, g, b, a) = color;
    format!(r#"fill="rgb({} {} {})" fill-opacity="{}""#, r, g, b, f64::from(a) / 255.)
}



fn render_symbol_as_curve<T : GivesOutline>(font: &T, pos: Cursor, scale: f64, gid: GlyphId, color : rex::RGBA) -> String {
    let (sx, sy) = font.font_scale();

    struct Builder {
        path : String,
    }

    impl Builder {
        fn new(
            tx : f64, ty : f64, 
            sx : f64, sy : f64,
            color : rex::RGBA,
            ) -> Self { 
            let mut path = String::with_capacity(r#"<path d="" />"#.len()); 
            path.push_str(&format!(
                r#"<path transform="translate({}, {}) scale({}, {})" {} d=""#,
                tx, ty,
                sx, sy,
                to_xml_color(color),
                ));
            Self { path } 
        }

        fn finalize(self) -> String {
            let Self { mut path } = self;
            path.push_str(r#"" />"#);
            path
        }
    }

    impl OutlineBuilder for Builder {
        fn move_to(&mut self, x: f32, y: f32) {
            self.path.push_str(&format!(
                "M{} {} ",
                x, y
                ));
        }

        fn line_to(&mut self, x: f32, y: f32) {
            self.path.push_str(&format!(
                "L{} {} ",
                x, y
                ));
        }

        fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
            self.path.push_str(&format!(
                "Q{} {}, {} {} ",
                x1, y1,
                x, y,
                ));
        }

        fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
            self.path.push_str(&format!(
                "C{} {}, {} {}, {} {} ",
                x1, y1,
                x2, y2,
                x, y,
                ));
        }

        fn close(&mut self) {
            self.path.push_str("Z ");
        }
    }

    let mut builder = Builder::new(
        pos.x, pos.y,
        scale * f64::from(sx), - scale * f64::from(sy),
        color
    );
    font.outline_glyph(gid, &mut builder);
    builder.finalize()
}
