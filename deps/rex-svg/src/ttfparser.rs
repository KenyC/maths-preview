use crate::GivesOutline;
use crate::OutlineBuilder;
use rex::font::common::GlyphId;
use rex::font::backend::ttf_parser::TtfMathFont;


impl GivesOutline for TtfMathFont<'_> {
    fn outline_glyph(&self, glyph_id : GlyphId, builder : &mut impl OutlineBuilder) {
        let glyph_id = ttf_parser::GlyphId(glyph_id.into());
        self.font().outline_glyph(glyph_id, &mut OutlineBuilderCompatibilityLater(builder));
    }
    fn font_scale(&self) -> (f32, f32) {
        let matrix = self.font_matrix();
        (matrix.sx, matrix.sy)
    }
}


struct OutlineBuilderCompatibilityLater<'a, T : OutlineBuilder>(& 'a mut T);

impl<'a, T : OutlineBuilder> ttf_parser::OutlineBuilder for OutlineBuilderCompatibilityLater<'a, T> {

    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(x, y)
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.quad_to(x1, y1, x, y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.curve_to(x1, y1, x2, y2, x, y);
    }

    fn close(&mut self) {
        self.0.close();
    }


}