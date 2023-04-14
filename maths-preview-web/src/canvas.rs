use rex::{Backend, font::{backend::ttf_parser::TtfMathFont, common::GlyphId}, GraphicsBackend, FontBackend};
use ttf_parser::OutlineBuilder;
use web_sys::{CanvasRenderingContext2d, CanvasWindingRule};



#[derive(Debug, Clone, Copy)]
pub struct CanvasContext<'a>(pub &'a CanvasRenderingContext2d);


impl<'a> Backend<TtfMathFont<'a>> for CanvasContext<'_> {}

impl GraphicsBackend for CanvasContext<'_> {
    fn rule(&mut self, pos: rex::Cursor, width: f64, height: f64) {
        let canvas = self.0;

        canvas.rect(pos.x, pos.y, width, height);
        canvas.fill();
    }

    fn begin_color(&mut self, color: rex::RGBA) {
    }

    fn end_color(&mut self) {
    }
    // add code here
}


impl FontBackend<TtfMathFont<'_>> for CanvasContext<'_> {
    fn symbol(&mut self, pos: rex::Cursor, gid: GlyphId, scale: f64, font: &TtfMathFont<'_>) {
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

        font.font().outline_glyph(gid.into(), &mut builder);
        builder.fill();
        canvas.restore();

    }
}


