use std::unimplemented;

use rex::{Backend, font::common::GlyphId, GraphicsBackend, FontBackend};
use owned_ttf_parser::OutlineBuilder;
use web_sys::{CanvasRenderingContext2d, CanvasWindingRule, OffscreenCanvasRenderingContext2d};

use super::owned_math_font::{TtfMathFont, into};



#[derive(Debug, Clone, Copy)]
pub struct CanvasContext<'a>(pub &'a CanvasRenderingContext2d);


impl<'a, 'b> Backend<TtfMathFont<'a, 'b>> for CanvasContext<'_> {}

impl GraphicsBackend for CanvasContext<'_> {
    fn rule(&mut self, pos: rex::Cursor, width: f64, height: f64) {
        let canvas = self.0;

        canvas.rect(pos.x, pos.y, width, height);
        canvas.fill();
    }

    fn begin_color(&mut self, _color: rex::RGBA) {
        unimplemented!()
    }

    fn end_color(&mut self) {
        unimplemented!()
    }
}


impl FontBackend<TtfMathFont<'_, '_>> for CanvasContext<'_> {
    fn symbol(&mut self, pos: rex::Cursor, gid: GlyphId, scale: f64, font: &TtfMathFont<'_, '_>) {
        let canvas = self.0;
        
        canvas.save();
        canvas.translate(pos.x, pos.y).unwrap();
        canvas.scale(scale, -scale).unwrap();
        canvas.scale(font.font_matrix().sx.into(), font.font_matrix().sy.into(),).unwrap();
        canvas.begin_path();

        struct Builder<'a> { 
            canvas : &'a CanvasRenderingContext2d,
        }

        impl<'a> Builder<'a> {
            fn fill(self) {
                self.canvas.fill_with_canvas_winding_rule(CanvasWindingRule::Evenodd);
                self.canvas.close_path();
            }
        }

        impl<'a> OutlineBuilder for Builder<'a> {
            fn move_to(&mut self, x: f32, y: f32) {
                // eprintln!("move_to {:?} {:?}", x, y);
                self.canvas.move_to(x.into(), y.into());
            }

            fn line_to(&mut self, x: f32, y: f32) {
                // eprintln!("line_to {:?} {:?}", x, y);
                self.canvas.line_to(x.into(), y.into());
            }

            fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
                // eprintln!("quad_to  {:?} {:?} {:?} {:?}", x1, y1, x, y);
                self.canvas.quadratic_curve_to(x1.into(), y1.into(), x.into(), y.into(),)
            }

            fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
                // eprintln!("curve_to {:?} {:?} {:?} {:?} {:?} {:?}", x1, y1, x2, y2, x, y);
                self.canvas.bezier_curve_to(x1.into(), y1.into(), x2.into(), y2.into(), x.into(), y.into(),)
            }

            fn close(&mut self) {
                // eprintln!("close");
                self.canvas.close_path();
            }

        }

        let mut builder = Builder {canvas,};

        font.font().outline_glyph(into(gid), &mut builder);
        builder.fill();
        canvas.restore();

    }
}




#[derive(Debug, Clone, Copy)]
pub struct OffscreenCanvasContext<'a>(pub &'a OffscreenCanvasRenderingContext2d);


impl<'a, 'b> Backend<TtfMathFont<'a, 'b>> for OffscreenCanvasContext<'_> {}

impl GraphicsBackend for OffscreenCanvasContext<'_> {
    fn rule(&mut self, pos: rex::Cursor, width: f64, height: f64) {
        let canvas = self.0;

        canvas.rect(pos.x, pos.y, width, height);
        canvas.fill();
    }

    fn begin_color(&mut self, _color: rex::RGBA) {
        unimplemented!()
    }

    fn end_color(&mut self) {
        unimplemented!()
    }
}


impl FontBackend<TtfMathFont<'_, '_>> for OffscreenCanvasContext<'_> {
    fn symbol(&mut self, pos: rex::Cursor, gid: GlyphId, scale: f64, font: &TtfMathFont<'_, '_>) {
        let canvas = self.0;
        
        canvas.save();
        canvas.translate(pos.x, pos.y).unwrap();
        canvas.scale(scale, -scale).unwrap();
        canvas.scale(font.font_matrix().sx.into(), font.font_matrix().sy.into(),).unwrap();
        canvas.begin_path();

        struct Builder<'a> { 
            canvas : &'a OffscreenCanvasRenderingContext2d,
        }

        impl<'a> Builder<'a> {
            fn fill(self) {
                self.canvas.fill_with_canvas_winding_rule(CanvasWindingRule::Evenodd);
                self.canvas.close_path();
            }
        }

        impl<'a> OutlineBuilder for Builder<'a> {
            fn move_to(&mut self, x: f32, y: f32) {
                // eprintln!("move_to {:?} {:?}", x, y);
                self.canvas.move_to(x.into(), y.into());
            }

            fn line_to(&mut self, x: f32, y: f32) {
                // eprintln!("line_to {:?} {:?}", x, y);
                self.canvas.line_to(x.into(), y.into());
            }

            fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
                // eprintln!("quad_to  {:?} {:?} {:?} {:?}", x1, y1, x, y);
                self.canvas.quadratic_curve_to(x1.into(), y1.into(), x.into(), y.into(),)
            }

            fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
                // eprintln!("curve_to {:?} {:?} {:?} {:?} {:?} {:?}", x1, y1, x2, y2, x, y);
                self.canvas.bezier_curve_to(x1.into(), y1.into(), x2.into(), y2.into(), x.into(), y.into(),)
            }

            fn close(&mut self) {
                // eprintln!("close");
                self.canvas.close_path();
            }

        }

        let mut builder = Builder {canvas,};

        font.font().outline_glyph(into(gid), &mut builder);
        builder.fill();
        canvas.restore();

    }
}

