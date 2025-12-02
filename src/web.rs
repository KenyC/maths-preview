mod canvas;
mod owned_math_font;



use canvas::{CanvasContext, OffscreenCanvasContext};
use owned_math_font::TtfMathFont;
use rex::{parser::parse, layout::engine::LayoutBuilder, Renderer};
use web_sys::{CanvasRenderingContext2d, OffscreenCanvasRenderingContext2d,};
use wasm_bindgen::prelude::*;
use owned_ttf_parser::{OwnedFace, AsFaceRef};
use crate::error::{AppError, AppResult};

use crate::geometry::{BBox, Metrics};
use crate::svg::SvgContext;
use crate::render::scale_and_center;

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
}

impl Context {
    pub fn as_ref<'a>(& 'a self) -> & 'a OwnedFace {
        unsafe { self.face.as_ref::<'a>().unwrap() }
    }

    pub fn new(value : Box<OwnedFace>) -> Self {
        Self { face: Box::leak(value) }
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
    let font = context.as_ref();
    let math_font  = TtfMathFont::new(font.as_face_ref()).unwrap();

    let (layout, formula_metrics) = layout_and_size(&math_font, PNG_FONT_SIZE, formula,)?;

    let width  = formula_metrics.bbox.width();
    let height = formula_metrics.bbox.height();
    let canvas_context : OffscreenCanvasRenderingContext2d = 
        canvas_with_size
        .call2(&JsValue::NULL, &JsValue::from_f64(width), &JsValue::from_f64(height),)
        .unwrap()
        .unchecked_into()
    ;


    canvas_context.translate(0., height + formula_metrics.baseline).unwrap();

    let mut context = OffscreenCanvasContext(&canvas_context);
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
) -> String {
    let font = context.as_ref();
    let math_font  = TtfMathFont::new(font.as_face_ref()).unwrap();

    let (layout, formula_metrics) = layout_and_size(&math_font, FONT_SIZE, formula,).unwrap();
    let mut svg_context = SvgContext::new();
    let renderer = Renderer::new();

    renderer.render(&layout, &mut svg_context);

    let height = formula_metrics.bbox.height();
    let width = formula_metrics.bbox.width();
    svg_context.finalize(
        0., - height - formula_metrics.baseline,
        width, height,
    )
}


fn render_formula_to_canvas(
    context : &Context,
    formula : &str, 
    canvas  : &CanvasRenderingContext2d
) -> AppResult<()> {
    let font = context.as_ref();
    let math_font  = TtfMathFont::new(font.as_face_ref()).unwrap();
    let mut context = CanvasContext(canvas);
    let canvas_size = get_canvas_size(context);
    context.0.clear_rect(0., 0., canvas_size.0, canvas_size.1);
    let (layout, formula_metrics) = layout_and_size(&math_font, FONT_SIZE, formula,)?;
    render_layout(&mut context, Some(canvas_size), &formula_metrics, layout);
    Ok(())
}

fn get_canvas_size(context: CanvasContext) -> (f64, f64) {
    let width  = context.0.canvas().unwrap().width() as f64;
    let height = context.0.canvas().unwrap().height() as f64;
    let canvas_size = (width, height,);
    canvas_size
}


fn layout_and_size<'a, 'f, 'b>(font: &'f TtfMathFont<'a, 'b>, font_size : f64, formula: &str) -> AppResult<(rex::layout::Layout<'f, TtfMathFont<'a, 'b>>, Metrics)> {
    let parse_node = parse(formula).map_err(|e| AppError::ParseError(format!("{}", e)))?;

    // Create node
    let layout = 
        LayoutBuilder::new(font)
        .font_size(font_size)
        .build()
        .layout(&parse_node)?
    ;

    let formula_bbox = layout.size();

    // Create metrics
    let metrics = Metrics {
        font_size,
        bbox: BBox::from_typographic(0., formula_bbox.depth, formula_bbox.width, formula_bbox.height,),
        baseline: formula_bbox.depth,
    };

    Ok((layout, metrics))
}


fn render_layout(
    context: &mut CanvasContext, 
    canvas_size: Option<(f64, f64)>, 
    formula_metrics: &Metrics, 
    layout: rex::layout::Layout<TtfMathFont>,
) {
    // let (x0, y0, x1, y1) = renderer.size(&node);
    context.0.save();
    let Metrics { bbox, .. } = formula_metrics;
    if let Some(canvas_size) = canvas_size {
        scale_and_center(*bbox, context, canvas_size);
    }

    let renderer = Renderer::new();
    renderer.render(&layout, context);


    context.0.restore();
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




