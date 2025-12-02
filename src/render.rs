
// use cairo::Context;
#[cfg(not(target_arch = "wasm32"))]
use rex::cairo::CairoBackend;
use rex::{font::MathFont, layout::engine::LayoutBuilder, parser::{macros::CommandCollection, parse_with_custom_commands}, Renderer};
use serde::Serialize;

use crate::{geometry::{Metrics, BBox}, error::{AppResult, AppError}};

pub trait RenderingView {
    fn save(&mut self) -> AppResult<()>;
    fn restore(&mut self) -> AppResult<()>;
    fn translate(&mut self, x : f64, y : f64) -> AppResult<()>;
    fn scale(&mut self, sx : f64, sy : f64) -> AppResult<()>;
}

pub fn draw_formula<'a, F, B>(
    formula : &str, 
    context: &mut B, 
    // font : Rc<TtfMathFont<'a>>, 
    font : &F, 
    font_size : f64, 
    canvas_size : Option<(f64, f64)>,
    custom_cmd : &CommandCollection,
) -> AppResult<()> 
where 
    F : MathFont,
    B : RenderingView + rex::Backend<F>,
{
    let (layout, formula_metrics) = layout_and_size(font, font_size, formula, custom_cmd,)?;
    render_layout(context, canvas_size, &formula_metrics, layout)
}

pub fn render_layout<B, F>(
    backend: &mut B, 
    canvas_size: Option<(f64, f64)>, 
    formula_metrics: &Metrics, 
    layout: rex::layout::Layout<F>,
) -> AppResult<()> 
where 
    F : MathFont,
    B : RenderingView + rex::Backend<F>,
{
    // let (x0, y0, x1, y1) = renderer.size(&node);
    backend.save()?;
    let Metrics { bbox, .. } = formula_metrics;
    if let Some(canvas_size) = canvas_size {
        scale_and_center(*bbox, backend, canvas_size);
    }

    let renderer = Renderer::new();
    renderer.render(&layout, backend);



    backend.restore()?;
    Ok(())
}



#[derive(Debug, Serialize)]
pub struct MetaInfo {
    pub metrics : Metrics,
    pub formula : String,
}


pub fn layout_and_size<'f, T : MathFont>(font: &'f T, font_size : f64, formula: &str, custom_cmd : &CommandCollection) -> AppResult<(rex::layout::Layout<'f, T>, Metrics)> {
    let parse_node = parse_with_custom_commands(formula, custom_cmd).map_err(|e| AppError::ParseError(format!("{}", e)))?;

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
        bbox: BBox::from_typographic(0., formula_bbox.depth, formula_bbox.width, formula_bbox.height,),
        baseline: formula_bbox.depth,
        font_size,
    };

    Ok((layout, metrics))
}

pub fn scale_and_center<C>(bbox: BBox, context: &mut C, canvas_size: (f64, f64)) 
where C : RenderingView
{
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

    // TODO deal with errors that might occur here
    context.scale(scale, scale);
    context.translate(tx, ty);

}

#[cfg(not(target_arch = "wasm32"))]
impl RenderingView for CairoBackend {
    fn save(&mut self) -> AppResult<()> {
        self.context_ref().save()?;
        Ok(())
    }

    fn restore(&mut self) -> AppResult<()> {
        self.context_ref().restore()?;
        Ok(())
    }

    fn translate(&mut self, x : f64, y : f64) -> AppResult<()> {
        self.context_ref().translate(x, y);
        Ok(())
    }

    fn scale(&mut self, sx : f64, sy : f64) -> AppResult<()> {
        self.context_ref().scale(sx, sy);
        Ok(())
    }
}

// #[allow(unused)]
// pub fn draw_bbox<C: RenderingView>(context: &mut C, x0: f64, y0: f64, width: f64, height: f64, x1: f64, y1: f64) {
//     context.set_source_rgb(1., 0., 0.);
//     context.rectangle(x0, y0, width, height);
//     context.stroke().unwrap();

//     context.set_source_rgb(0., 1., 0.);
//     const WIDTH_POINT : f64 = 5.;
//     context.rectangle(x0 - WIDTH_POINT * 0.5, y0 - WIDTH_POINT * 0.5, WIDTH_POINT, WIDTH_POINT);
//     context.fill().unwrap();

//     context.set_source_rgb(0., 1., 0.);
//     context.rectangle(x1 - WIDTH_POINT * 0.5, y1 - WIDTH_POINT * 0.5, WIDTH_POINT, WIDTH_POINT);
//     context.fill().unwrap();
// }