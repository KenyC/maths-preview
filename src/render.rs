use std::rc::Rc;

use cairo::Context;
use rex::{Renderer, font::{backend::ttf_parser::TtfMathFont, FontContext}, parser::{macros::CommandCollection, parse_with_custom_commands}, layout::engine::layout};
use serde::Serialize;

use crate::{geometry::{Metrics, BBox}, error::{AppResult, AppError}};



pub fn draw_formula<'a>(
    formula : &str, 
    context: &Context, 
    font : Rc<TtfMathFont<'a>>, 
    font_size : f64, 
    canvas_size : Option<(f64, f64)>,
    custom_cmd : &CommandCollection,
) -> AppResult<()> {
    let (layout, formula_metrics) = layout_and_size(font.as_ref(), font_size, formula, custom_cmd,)?;
    render_layout(context, canvas_size, &formula_metrics, layout)
}

pub fn render_layout(
    context: &Context, 
    canvas_size: Option<(f64, f64)>, 
    formula_metrics: &Metrics, 
    layout: rex::layout::Layout<TtfMathFont>,
) -> AppResult<()> {
    // let (x0, y0, x1, y1) = renderer.size(&node);
    context.save()?;
    let Metrics { bbox, .. } = formula_metrics;
    if let Some(canvas_size) = canvas_size {
        scale_and_center(*bbox, context, canvas_size);
    }

    let mut backend = rex::render::cairo::CairoBackend::new(context.clone());
    let renderer = Renderer::new();
    renderer.render(&layout, &mut backend);



    context.restore()?;
    Ok(())
}



#[derive(Debug, Serialize)]
pub struct MetaInfo {
    pub metrics : Metrics,
    pub formula : String,
}


pub fn layout_and_size<'a, 'f>(font: &'f TtfMathFont<'a>, font_size : f64, formula: &str, custom_cmd : &CommandCollection) -> AppResult<(rex::layout::Layout<'f, TtfMathFont<'a>>, Metrics)> {
    let parse_node = parse_with_custom_commands(formula, custom_cmd).map_err(|e| AppError::ParseError(format!("{}", e)))?;

    // Create node
    let font_context = FontContext::new(font);
    let layout_settings = rex::layout::LayoutSettings::new(&font_context).font_size(font_size);
    let layout = layout(&parse_node, layout_settings)?;

    let formula_bbox = layout.size();

    // Create metrics
    let metrics = Metrics {
        bbox: BBox::from_typographic(0., formula_bbox.depth, formula_bbox.width, formula_bbox.height,),
        baseline: formula_bbox.depth,
        font_size,
    };

    Ok((layout, metrics))
}

pub fn scale_and_center(bbox: BBox, context: &Context, canvas_size: (f64, f64)) {
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
    context.scale(scale, scale);
    context.translate(tx, ty);

}

#[allow(unused)]
pub fn draw_bbox(context: &Context, x0: f64, y0: f64, width: f64, height: f64, x1: f64, y1: f64) {
    context.set_source_rgb(1., 0., 0.);
    context.rectangle(x0, y0, width, height);
    context.stroke().unwrap();

    context.set_source_rgb(0., 1., 0.);
    const WIDTH_POINT : f64 = 5.;
    context.rectangle(x0 - WIDTH_POINT * 0.5, y0 - WIDTH_POINT * 0.5, WIDTH_POINT, WIDTH_POINT);
    context.fill().unwrap();

    context.set_source_rgb(0., 1., 0.);
    context.rectangle(x1 - WIDTH_POINT * 0.5, y1 - WIDTH_POINT * 0.5, WIDTH_POINT, WIDTH_POINT);
    context.fill().unwrap();
}