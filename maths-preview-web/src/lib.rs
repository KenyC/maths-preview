mod utils;
mod error;
mod canvas;
mod ui;

use canvas::CanvasContext;
use rex::{font::{backend::ttf_parser::TtfMathFont, common::GlyphId, FontContext}, GraphicsBackend, FontBackend, Backend, parser::parse, layout::Grid, Renderer};
use ui::{ErrorBar, initiate_download_file};
use web_sys::{CanvasRenderingContext2d, CanvasWindingRule, HtmlCanvasElement, HtmlElement, Blob,};
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;
use error::{AppError, AppResult};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}


const FONT_FILE : &[u8] = include_bytes!("../resources/LibertinusMath-Regular.otf");
// const FONT_FILE : &[u8] = include_bytes!("../resources/LibertinusMath-Regular.woff2");
const TEXT_TO_RENDER : &str = "fez";

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    set_panic_hook();


    // -- Extract document
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();


    // -- Get error <div>
    let error_bar = document.get_element_by_id("error").unwrap();
    let error_bar = ErrorBar::new(error_bar);


    // -- Load math font
    error_bar.set_text("Loading math font ...");
    let font = ttf_parser::Face::parse(FONT_FILE, 0).unwrap();
    let math_font  = std::rc::Rc::new(TtfMathFont::new(font).unwrap());





    // -- Get canvas & edit element
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>().unwrap()
        ;
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();
    let text_edit =
        document
        .get_element_by_id("formula").unwrap()
        .dyn_into::<web_sys::HtmlInputElement>().ok().unwrap()
    ;
    // let context = CanvasContext(&context);





    // -- Initial set-up
    error_bar.set_text("Setting up page & handlers ...");
    let body = document.body().unwrap();
    resize_canvas_to_body_size(&body, &canvas);
    update_canvas(&text_edit.value(), CanvasContext(&context), &math_font).unwrap_or(());





    // -- Resize handler
    let canvas_clone = canvas.clone();
    let resize_handler = Closure::wrap(Box::new(move || {
        resize_canvas_to_body_size(&body, &canvas);
    }) as Box<dyn Fn()>
    );
    window.set_onresize(Some(resize_handler.as_ref().unchecked_ref()));
    resize_handler.forget();









    // -- Text edit handler
    let text_edit_clone = text_edit.clone();
    let context_clone   = context.clone();
    let math_font_clone = math_font.clone();
    let error_bar_clone = error_bar.clone();
    let oninput_handler = Closure::wrap(Box::new(move || {
        let text = text_edit_clone.value();
        if let Err(err) = update_canvas(&text, CanvasContext(&context), &math_font) {
            let human_readable_err = err.human_readable();
            log(&human_readable_err);
            error_bar_clone.set_text(&human_readable_err);
        }
        else {
            error_bar_clone.hide();
        }
    }) as Box<dyn Fn()>);
    text_edit.set_oninput(Some(oninput_handler.as_ref().unchecked_ref()));
    oninput_handler.forget();





    // -- Render button & handler
    let button = document
        .get_element_by_id("render").unwrap()
        .dyn_into::<web_sys::HtmlButtonElement>().ok().unwrap()
    ;
    let etzt = Closure::wrap(Box::new(move |blob| {
        initiate_download_file(&document, &blob, None).unwrap();
    }) as Box<dyn Fn(Blob)>);
    let onclick_handler = Closure::wrap(
        Box::new(move || {
            canvas_clone.to_blob(etzt.as_ref().unchecked_ref());
            log("Initiating file download");
        }) as Box<dyn Fn()>
    );
    button.set_onclick(Some(onclick_handler.as_ref().unchecked_ref()));
    onclick_handler.forget();


    error_bar.hide();

    Ok(())
}

fn resize_canvas_to_body_size(body :  &HtmlElement, canvas : &HtmlCanvasElement) {
    const SCREEN_FORMAT : f64 = 16. / 9.;

    let client_width  = body.client_width();
    let client_height = ((client_width as f64) / SCREEN_FORMAT).floor();
    canvas.set_width(client_width as u32);        
    canvas.set_height(client_height as u32);        
}

fn update_canvas<'a>(formula : &str, context : CanvasContext, math_font : &TtfMathFont<'a>) -> AppResult<()> {
    render_formula(formula, context, math_font)
}


fn render_formula<'a>(formula : &str, mut context : CanvasContext, math_font : &TtfMathFont<'a>) -> AppResult<()> {
    let (layout, _a, formula_metrics) = layout_and_size(&math_font, 10., formula)?;
    let renderer = Renderer::new();

    let canvas_size = get_canvas_size(context);
    context.0.clear_rect(0., 0., canvas_size.0, canvas_size.1);
    render_layout(&mut context, Some(canvas_size), &formula_metrics, renderer, layout)?;

    Ok(())
}

fn get_canvas_size(context: CanvasContext) -> (f64, f64) {
    let width  = context.0.canvas().unwrap().width() as f64;
    let height = context.0.canvas().unwrap().height() as f64;
    let canvas_size = (width, height,);
    canvas_size
}


fn layout_and_size<'a, 'f>(font: &'f TtfMathFont<'a>, font_size : f64, formula: &str) -> Result<(rex::layout::Layout<'f, TtfMathFont<'a>>, Renderer, Metrics), AppError> {
    let parse_node = parse(formula).map_err(|e| AppError::ParseError(format!("{}", e)))?;

    // Create node
    let font_context = FontContext::new(font)?;
    let layout_settings = rex::layout::LayoutSettings::new(&font_context, font_size, rex::layout::Style::Display);
    let node = rex::layout::engine::layout(&parse_node, layout_settings)?;
    let depth = node.depth;

    // Lay out node
    let mut grid = Grid::new();
    grid.insert(0, 0, node.as_node());
    let mut layout = rex::layout::Layout::new();
    layout.add_node(grid.build());

    // Size
    let renderer = Renderer::new();
    let formula_bbox = renderer.size(&layout);

    // Create metrics
    let metrics = Metrics {
        bbox: BBox::new(formula_bbox.0, formula_bbox.1, formula_bbox.2, formula_bbox.3,),
        baseline: depth / rex::dimensions::Px,
        font_size,
    };

    Ok((layout, renderer, metrics))
}

fn scale_and_center(bbox: BBox, context: &CanvasContext, canvas_size: (f64, f64)) {
    let width   = bbox.width();
    let height  = bbox.height();
    if width <= 0. || height < 0. {return;}
    let (canvas_width, canvas_height) = canvas_size;
    let BBox { x_min, y_min, x_max, y_max } = bbox;
    let midx = 0.5 * (x_min + x_max);
    let midy = 0.5 * (y_min + y_max);

    let fit_to_width  = canvas_width / width;
    let fit_to_height = canvas_height / height;
    let optimal_scale = f64::min(fit_to_width, fit_to_height);
    // we don't want the scale to keep changing as we type
    // we only zoom out when the formula gets out of bound and we scale conservatively.
    const FACTOR_INCREMENT : f64 = 0.65;
    let scale = FACTOR_INCREMENT.powf((optimal_scale).log(FACTOR_INCREMENT).ceil());

    let tx = - (midx - 0.5 *  canvas_width / scale);
    let ty = - (midy - 0.5 *  canvas_height / scale);
    // draw_bbox(context, 0., 0., canvas_width, canvas_height, 10., 10.);
    context.0.scale(scale, scale).unwrap();
    context.0.translate(tx, ty).unwrap();

}

fn render_layout(
    context: &mut CanvasContext, 
    canvas_size: Option<(f64, f64)>, 
    formula_metrics: &Metrics, 
    renderer: Renderer, 
    layout: rex::layout::Layout<TtfMathFont>,
) -> Result<(), AppError> {
    // let (x0, y0, x1, y1) = renderer.size(&node);
    context.0.save();
    let Metrics { bbox, .. } = formula_metrics;
    if let Some(canvas_size) = canvas_size {
        scale_and_center(*bbox, context, canvas_size);
    }

    renderer.render(&layout, context);



    context.0.restore();
    Ok(())
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




#[derive(Debug,)]
struct Metrics {
    bbox      : BBox,
    baseline  : f64,
    font_size : f64,
}

#[derive(Debug, Clone, Copy)]
struct BBox {
    x_min  : f64,
    y_min  : f64,
    x_max  : f64,
    y_max  : f64,
}

impl BBox {
    fn new(x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> Self { Self { x_min, y_min, x_max, y_max } }

    #[inline]
    fn width(&self) -> f64 { self.x_max - self.x_min }

    #[inline]
    fn height(&self) -> f64 { self.y_max - self.y_min }
}