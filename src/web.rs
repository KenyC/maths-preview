mod canvas;
mod owned_math_font;



use canvas::{CanvasContext, OffscreenCanvasContext};
use owned_math_font::TtfMathFont;
use rex::{parser::parse, layout::engine::LayoutBuilder, Renderer};
use rex::parser::macros::CommandCollection;
use web_sys::{CanvasRenderingContext2d, OffscreenCanvasRenderingContext2d,};
use wasm_bindgen::prelude::*;
use owned_ttf_parser::{OwnedFace, AsFaceRef};
use crate::error::{AppError, AppResult};

use crate::geometry::{BBox, Metrics};
use crate::svg::SvgContext;
use crate::render::{render_svg, scale_and_center, layout_and_size, render_layout};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
// #[cfg(feature = "wee_alloc")]
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;



const FONT_FILE : &[u8] = include_bytes!("../resources/LibertinusMath-Regular.otf");
// const FONT_FILE : &[u8] = include_bytes!("../resources/LibertinusMath-Regular.woff2");



#[wasm_bindgen]
pub struct Context {
    face : *const OwnedFace,
    glyph_as_text: bool,
    font_size: f64,
}

impl Context {
    pub fn font<'a>(& 'a self) -> & 'a OwnedFace {
        unsafe { self.face.as_ref::<'a>().unwrap() }
    }

    pub fn new(value : Box<OwnedFace>) -> Self {
        Self { face: Box::leak(value), font_size: FONT_SIZE, glyph_as_text: false, }
    }
}

#[wasm_bindgen]
impl Context {
    pub fn set_settings_from_js(
        &mut self,
        glyph_as_text: bool,
        font_size: &str,
    ) {
        self.glyph_as_text = glyph_as_text;

        if let Ok(value) = font_size.parse() {
            self.font_size = value;
        }
    }

    #[wasm_bindgen(getter)]
    pub fn glyph_as_text(&self) -> bool {
        self.glyph_as_text
    }

    #[wasm_bindgen(getter)]
    pub fn font_size(&self) -> f64 {
        self.font_size
    }
}

#[wasm_bindgen]
pub fn init_font() -> Context {
    let font = Box::new(owned_ttf_parser::OwnedFace::from_vec(FONT_FILE.to_vec(), 0).unwrap());
    Context::new(font)
}

const FONT_SIZE : f64 = 10.;


#[wasm_bindgen]
pub fn render_formula_to_offscreen_canvas_js_err(
    context    : &Context,
    formula    : &str,
    // since we can't know what size the formula will be prior to calling 'layout'
    // the canvas is created by a JS function which takes two arguments
    make_new_canvas : &js_sys::Function,
) -> Result<(), String> {
    render_formula_to_offscreen_canvas(context, formula, make_new_canvas).map_err(|e| {
        e.to_string()
    })
}

fn render_formula_to_offscreen_canvas(
    context    : &Context,
    formula    : &str,
    canvas_with_size : &js_sys::Function,
)  -> Result<(), AppError> {
    const PNG_FONT_SIZE : f64 = 300.;
    let font = context.font();
    let math_font  = TtfMathFont::new(font.as_face_ref()).unwrap();

    let (layout, formula_metrics) = layout_and_size(&math_font, PNG_FONT_SIZE, formula, &CommandCollection::default())?;

    let width  = formula_metrics.bbox.width();
    let height = formula_metrics.bbox.height();
    let canvas_context : OffscreenCanvasRenderingContext2d = 
        canvas_with_size
        .call2(&JsValue::NULL, &JsValue::from_f64(width), &JsValue::from_f64(height),)
        .unwrap()
        .unchecked_into()
    ;


    canvas_context.translate(0., height + formula_metrics.baseline).unwrap();

    let mut context = OffscreenCanvasContext::new(&canvas_context);
    Renderer::new().render(&layout, &mut context);


    Ok(())
}

#[wasm_bindgen]
pub fn render_formula_to_canvas_js_err(
    context : &Context,
    formula : &str, 
    canvas  : &CanvasRenderingContext2d
) -> Result<(), String> {
    render_formula_to_canvas(context, formula, canvas).map_err(|e| {
        e.to_string()
    })
}


#[wasm_bindgen]
pub fn render_formula_to_svg(
    context : &Context,
    formula : &str, 
) -> Result<String, String> {
    let font = context.as_ref();
    let math_font  = TtfMathFont::new(font.as_face_ref()).unwrap();

    let svg_render_result = render_svg(
        formula,
        &math_font,
        context.font_size,
        &CommandCollection::default(),
        context.glyph_as_text,
    );
    match svg_render_result {
        Ok((_, svg_string)) => Ok(svg_string),
        Err(e) => Err(e.to_string()),
    }
}


fn render_formula_to_canvas(
    context : &Context,
    formula : &str, 
    canvas  : &CanvasRenderingContext2d
) -> AppResult<()> {
    let font = context.as_ref();
    let math_font  = TtfMathFont::new(font.as_face_ref()).unwrap();
    let mut context = CanvasContext::new(canvas);
    let canvas_size = get_canvas_size(&context);
    context.rendering_context.clear_rect(0., 0., canvas_size.0, canvas_size.1);
    let (layout, formula_metrics) = layout_and_size(&math_font, FONT_SIZE, formula, &CommandCollection::default())?;
    render_layout(&mut context, Some(canvas_size), &formula_metrics, layout);
    Ok(())
}

fn get_canvas_size(context: &CanvasContext) -> (f64, f64) {
    let width  = context.rendering_context.canvas().unwrap().width() as f64;
    let height = context.rendering_context.canvas().unwrap().height() as f64;
    let canvas_size = (width, height,);
    canvas_size
}







// fn draw_face(context: &web_sys::CanvasRenderingContext2d) {
//     context.begin_path();

//     // Draw the outer circle.
//     context
//         .arc(75.0, 75.0, 50.0, 0.0, std::f64::consts::PI * 2.0)
//         .unwrap();

//     // Draw the mouth.
//     context.move_to(110.0, 75.0);
//     context.arc(75.0, 75.0, 35.0, 0.0, std::f64::consts::PI).unwrap();

//     // Draw the left eye.
//     context.move_to(65.0, 65.0);
//     context
//         .arc(60.0, 65.0, 5.0, 0.0, std::f64::consts::PI * 2.0)
//         .unwrap();

//     // Draw the right eye.
//     context.move_to(95.0, 65.0);
//     context
//         .arc(90.0, 65.0, 5.0, 0.0, std::f64::consts::PI * 2.0)
//         .unwrap();

//     context.stroke();
// }




