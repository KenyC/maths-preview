use std::cell::{RefCell, Cell};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::rc::Rc;


use rex::font::common::GlyphId;
use rex::font::backend::ttf_parser::TtfMathFont;
use rex::layout::engine::LayoutBuilder;
use rex::parser::macros::CommandCollection;
use rex::parser::parse_with_custom_commands;
use rex::Renderer;
use serde_json;




use crate::error::AppResult;
use crate::desktop::cli::{Format, Output, DEFAULT_FONT, EXAMPLE_FORMULA, UI_FONT_SIZE};
use crate::render::{MetaInfo, render_svg};
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
            let (metrics, svg_string) = render_svg(&text, font.as_ref(), font_size, custom_cmd, glyph_as_text)?;
            outfile.stream()?.write(svg_string.as_bytes())?;

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



