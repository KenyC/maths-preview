use std::cell::{RefCell, Cell};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::rc::Rc;


use gtk4::prelude::{ApplicationExt, ActionMapExt, ApplicationExtManual};
use gtk4::prelude::{GtkApplicationExt, GtkWindowExt};
use rex::font::common::GlyphId;
use rex::font::backend::ttf_parser::TtfMathFont;
use rex::layout::engine::LayoutBuilder;
use rex::parser::macros::CommandCollection;
use rex::parser::parse_with_custom_commands;
use rex::Renderer;
use serde_json;

use gtk4::gio::SimpleAction;
use gtk4::glib::clone;
use gtk4::glib;
use gtk4::Application;



use crate::error::AppResult;
use crate::ui::build_ui;
use crate::cli::{Format, Output, DEFAULT_FONT, EXAMPLE_FORMULA, UI_FONT_SIZE};
use crate::render::MetaInfo;
use crate::geometry::Metrics;
use crate::error::AppError;
use crate::geometry::BBox;
use crate::glyph_to_character::collect_chars;


#[derive(Clone)]
pub struct AppContext {
    pub math_font  : Rc<Cell<& 'static [u8]>>,
    pub format     : Rc<Cell<Format>>,
    pub font_size  : Rc<Cell<f64>>,
    pub custom_cmd : Rc<RefCell<CommandCollection>>,
    pub outfile    : Rc<RefCell<Output>>,
    pub informula  : Rc<RefCell<String>>,
    pub metainfo   : Rc<Cell<bool>>,
}

impl Default for AppContext {
    fn default() -> Self {
        Self {
            math_font:  Rc::new(Cell::new(DEFAULT_FONT)),
            format:     Rc::new(Cell::default()),
            font_size:  Rc::new(Cell::new(UI_FONT_SIZE)),
            outfile:    Rc::new(RefCell::default()),
            informula:  Rc::new(RefCell::new(EXAMPLE_FORMULA.to_string())),
            metainfo:   Rc::new(Cell::new(false)),
            custom_cmd: Rc::default(),
        }
    }
}

pub fn save_to_output(text: &str, outfile: &Output, format : Format, font : Rc<TtfMathFont>, font_size : f64, print_metainfo : bool, custom_cmd : &CommandCollection) -> AppResult<()> {
    eprintln!("Saving to {:?}", outfile);

    match format {
        Format::Svg { glyph_as_text } => {
            let metrics = save_svg(outfile, &text, font, font_size, custom_cmd, glyph_as_text)?;
            if print_metainfo {
                let metainfo = MetaInfo { metrics, formula: text.to_string() };
                let json = serde_json::to_string(&metainfo);
                match json {
                    Ok(json)  => println!("{}", json),
                    Err(err)  => {dbg!(err);},
                }
            }
            Ok(())
        },
        Format::Tex => save_tex(outfile, &text),
    }
}


fn save_tex(outfile: &Output, text: &str) -> AppResult<()> {
    outfile.stream()?.write(text.as_bytes())?;
    Ok(())
}


fn save_svg(path : &Output, formula : &str, font : Rc<TtfMathFont>, font_size : f64, custom_cmd : &CommandCollection, glyph_as_text : bool) -> AppResult<Metrics> {
    eprintln!("Saving to SVG!");
    let font_ref = font.as_ref();
    let nodes = parse_with_custom_commands(formula, custom_cmd).map_err(|e| AppError::ParseError(format!("{}", e)))?;



    let layout_engine = 
        LayoutBuilder::new(font_ref)
        .font_size(font_size)
        .build()
    ;
    let layout = layout_engine.layout(&nodes)?;


    // Create metrics
    let layout_size = layout.size();
    let formula_bbox = BBox::from_typographic(0., layout_size.depth, layout_size.width, layout_size.height,);
    let formula_metrics = Metrics {
        bbox: formula_bbox,
        baseline: layout_size.depth,
        font_size,
    };

    let x = formula_bbox.x_min;
    let y = formula_bbox.y_min;
    let width  = formula_bbox.width();
    let height = formula_bbox.height();
    



    // For text-as-text rendering, we need to construct the glyph to char oracle


    // render_layout(&context, None, &formula_metrics, layout)?;
    let mut svg = rex_svg::SvgContext::new();
    if glyph_as_text {
        let mut char_set = HashSet::new();
        for node in nodes {
            collect_chars(&node, &mut char_set);
        }
        let glyph_to_char_table : HashMap<GlyphId, char> = 
            char_set
            .into_iter()
            .map(|character| font.font().glyph_index(character).map(|glyph_id| (GlyphId::from(glyph_id), character)))
            .flatten()
            .collect()
        ;
        let font_name = find_font_family_name(font.as_ref());
        if let Some(font_name) = font_name {
            svg.glyph_as_text(glyph_to_char_table, &font_name);
        }
    }

    let renderer = Renderer::new();
    renderer.render(&layout, &mut svg);

    path.stream()?.write(svg.finalize(x, y, width, height).as_bytes())?;

    Ok(formula_metrics)

}

fn find_font_family_name<'a>(font: & 'a TtfMathFont) -> Option<String> {
    let table = font.font().tables().name?.names;

    for name in table {
        // Cf https://learn.microsoft.com/en-us/typography/opentype/spec/name for meaning of id's
        // This gets font-family name
        // TODO take into account language & platform ID 
        if name.name_id == 1 {
            if let Ok(to_return) = utf16string::WStr::from_utf16be(name.name) {
                return Some(to_return.to_utf8());
            }
        }
    }

    None
}



fn load_font<'a>(file : &'a [u8]) -> AppResult<TtfMathFont<'a>> {
    let font = ttf_parser::Face::parse(file, 0)?;
    Ok(TtfMathFont::new(font)?)
}

