use std::{unimplemented, format};

use owned_ttf_parser::OutlineBuilder;
use rex::{GraphicsBackend, Cursor, FontBackend, Backend};

use crate::owned_math_font::{TtfMathFont, into};


pub struct SvgContext {
	content : String
}

impl SvgContext {
    pub fn new() -> Self { 
    	Self { content : String::new() } 
    }

	pub fn finalize(self, x : f64, y : f64, width : f64, height : f64) -> String {
		format!(r#"<svg viewBox="{} {} {} {}">{}</svg>"#, 
			x, y,
			width, height, 
			self.content
		)
	}
}

impl<'a, 'b> Backend<TtfMathFont<'a, 'b>> for SvgContext {}


impl GraphicsBackend for SvgContext {
	fn rule(&mut self, pos: Cursor, width: f64, height: f64) {
		let Cursor { x, y } = pos;
		self.content.push_str(&format!(
			r#"<rect x="{}" y="{}" width="{}" height="{}" />"#,
			x, y, width, height
			));
	}

	fn begin_color(&mut self, color: rex::RGBA) {
		unimplemented!()
	}

	fn end_color(&mut self) {
		unimplemented!()
	}
}

impl FontBackend<TtfMathFont<'_, '_>> for SvgContext {
	fn symbol(&mut self, pos: Cursor, gid: rex::font::common::GlyphId, scale: f64, font: &TtfMathFont<'_, '_>) {
		let font_matrix = font.font_matrix();

		struct Builder {
			path : String,
		}

		impl Builder {
			fn new(
				tx : f64, ty : f64, 
				sx : f64, sy : f64,
			) -> Self { 
				let mut path = String::with_capacity(r#"<path d="" />"#.len()); 
				path.push_str(&format!(
					r#"<path transform="translate({}, {}) scale({}, {})" fill="black" d=""#,
					tx, ty,
					sx, sy,
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
			scale * f64::from(font_matrix.sx), - scale * f64::from(font_matrix.sy),
		);
		font.font().outline_glyph(into(gid), &mut builder);
		self.content.push_str(&builder.finalize());

	}
}

