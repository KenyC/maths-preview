mod error;
mod canvas;
mod owned_math_font;

use canvas::CanvasContext;
use owned_math_font::TtfMathFont;
use rex::{font::FontContext, parser::parse, layout::Grid, Renderer};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement, Blob,};
use wasm_bindgen::prelude::*;
use owned_ttf_parser::{OwnedFace, AsFaceRef};
use error::{AppError, AppResult};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;



const FONT_FILE : &[u8] = include_bytes!("../resources/LibertinusMath-Regular.otf");
// const FONT_FILE : &[u8] = include_bytes!("../resources/LibertinusMath-Regular.woff2");

// static FONT_CONTEXT: RefCell<Option<OwnedFace>> = RefCell::default();


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

#[wasm_bindgen]
pub fn render_formula_no_err(
    context : &Context,
    formula : &str, 
    canvas : &CanvasRenderingContext2d
) -> () {
    render_formula(context, formula, canvas).unwrap()
}

fn resize_canvas_to_body_size(body :  &HtmlElement, canvas : &HtmlCanvasElement) {
    const SCREEN_FORMAT : f64 = 16. / 9.;

    let client_width  = body.client_width();
    let client_height = ((client_width as f64) / SCREEN_FORMAT).floor();
    canvas.set_width(client_width as u32);        
    canvas.set_height(client_height as u32);        
}


fn render_formula(
    context : &Context,
    formula : &str, 
    canvas  : &CanvasRenderingContext2d
) -> AppResult<()> {
    {
        let font = context.as_ref().as_face_ref();
        let math_font  = TtfMathFont::new(font).unwrap();

        let mut context = CanvasContext(canvas);
        let (layout, _a, formula_metrics) = layout_and_size(&math_font, 10., formula)?;

        let canvas_size = get_canvas_size(context);
        context.0.clear_rect(0., 0., canvas_size.0, canvas_size.1);
        let renderer = Renderer::new();
        render_layout(&mut context, Some(canvas_size), &formula_metrics, renderer, layout)?;

        Ok(())
    }
}

fn get_canvas_size(context: CanvasContext) -> (f64, f64) {
    let width  = context.0.canvas().unwrap().width() as f64;
    let height = context.0.canvas().unwrap().height() as f64;
    let canvas_size = (width, height,);
    canvas_size
}


fn layout_and_size<'a, 'f, 'b>(font: &'f TtfMathFont<'a, 'b>, font_size : f64, formula: &str) -> Result<(rex::layout::Layout<'f, TtfMathFont<'a, 'b>>, Renderer, Metrics), AppError> {
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
    let formula_bbox = layout.size();

    // Create metrics
    let metrics = Metrics {
        bbox: BBox::from_typographic(0., formula_bbox.depth, formula_bbox.width, formula_bbox.height,),
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
    if let Some(canvas_size) = canvas_size {
        let Metrics { bbox, .. } = formula_metrics;
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

    /// This assumes the baseline is at y = 0
    pub fn from_typographic(x_min: f64, depth: f64, x_max: f64, height: f64) -> Self { 
        // height is signed distance from baseline to top of the glyph's bounding box
        // height > 0 means that top of bouding box is above baseline (i.e. y_min)
        // above in the screen's coordinate system means Y < 0
        // So y_min = - height
        // Similar reasoning for depth
        Self { x_min, y_min : -height, x_max, y_max : -depth } 
    }
}