use std::io::Write;
use std::path::PathBuf;

use rex::parser::macros::CommandCollection;

use gtk4::glib::VariantDict;
use gtk4::prelude::*;
use gtk4::Application;



use crate::error::AppResult;
use crate::desktop::app::AppContext;


pub const EXAMPLE_FORMULA : &str = r"\left.x^{x^{x^x_x}_{x^x_x}}_{x^{x^x_x}_{x^x_x}}\right\} \mathrm{wat?}";
pub const UI_FONT_SIZE : f64 = 10.0;
pub const DEFAULT_FONT : &[u8] = include_bytes!("../../resources/LibertinusMath-Regular.otf");


#[derive(Debug, Clone, Copy)]
pub enum Format {
    Svg { glyph_as_text : bool }, 
    Tex,
}

impl Default for Format {
    fn default() -> Self 
    { Self::Tex }
}




#[derive(Debug,)]
pub enum Output {
    Stdout,
    Path(PathBuf),
}

impl Output {
    pub fn stream(&self) -> std::io::Result<Box<dyn Write + 'static>> {
        match self {
            Output::Stdout     => Ok(Box::new(std::io::stdout())),
            Output::Path(path) => Ok(Box::new(std::fs::File::create(path)?)),
        }
    } 
}

impl Default for Output {
    fn default() -> Self { Self::Stdout }
}


pub fn setup_command_line(application: &Application) {
    application.add_main_option(
        "mathfont", 
        gtk4::glib::Char(b'm' as i8), 
        gtk4::glib::OptionFlags::IN_MAIN, 
        gtk4::glib::OptionArg::Filename, 
        "Path to an OpenType maths font to use for render (default: STIX Maths, bundled in the executable)", 
        None,
    );

    application.add_main_option(
        "informula", 
        gtk4::glib::Char(b'i' as i8), 
        gtk4::glib::OptionFlags::IN_MAIN, 
        gtk4::glib::OptionArg::String, 
        &format!("Formula to edit (default: ${}$)", EXAMPLE_FORMULA), 
        None,
    );

    application.add_main_option(
        "outfile", 
        gtk4::glib::Char(b'o' as i8), 
        gtk4::glib::OptionFlags::IN_MAIN, 
        gtk4::glib::OptionArg::Filename, 
        "Output file ; if left unspecified, output is directed to stdout.", 
        None,
    );

    application.add_main_option(
        "styfile", 
        gtk4::glib::Char(b'y' as i8), 
        gtk4::glib::OptionFlags::IN_MAIN, 
        gtk4::glib::OptionArg::Filename, 
        "Reads a style file to provide custom command", 
        None,
    );


    application.add_main_option(
        "metainfo", 
        gtk4::glib::Char(b'd' as i8), 
        gtk4::glib::OptionFlags::IN_MAIN,
        gtk4::glib::OptionArg::None, 
        "For SVG outputs, whether to output some meta-info on stdout (baseline position, font size, formula, etc). All measures reported are in SVG user units. If 'outfile' is not specified and this option is used, stdout will contain both the output and the meta-info. If 'format' is tex, this option does nothing.", 
        None,
    );

    application.add_main_option(
        "glyphastext", 
        gtk4::glib::Char(b't' as i8), 
        gtk4::glib::OptionFlags::IN_MAIN,
        gtk4::glib::OptionArg::None, 
        "For SVG outputs, renders glyphs as '<text>' where possible (the default is to render them as curves and lines), deferring the job of rendering the glyph to the SVG viewer. This may result in crispier renders on some platforms and software, e.g. because these platforms use subpixel antialiasing, but it also makes the resulting file dependent on the font being installed on the platform.", 
        None,
    );

    application.add_main_option(
        "format",
        gtk4::glib::Char(b'f' as i8),
        gtk4::glib::OptionFlags::IN_MAIN, 
        gtk4::glib::OptionArg::String, 
        "Format of 'outfile' ('svg', 'tex') ; defaults to 'tex'.", 
        None,
    );

    application.add_main_option(
        "fontsize",
        gtk4::glib::Char(b's' as i8),
        gtk4::glib::OptionFlags::IN_MAIN, 
        gtk4::glib::OptionArg::Double, 
        "Size of font in the SVG output (default: 10)", 
        None,
    );
}

fn parse_path(option : &VariantDict) -> AppResult<Option<& 'static [u8]>> {
    if let Some(mathfont) = option.lookup_value("mathfont", None) {
        if let Some(path) = mathfont.try_get::<PathBuf>().ok() {
            let font_bytes = std::fs::read(&path)?;
            // TODO: find a more elegant way to deal with lifetimes.
            // The lifetime in TtfMathFont & the requirement that closures fed to GTK are 'static come in conflict.
            // We leak the memory of the box so as to get a 'static reference.
            // This is ok, because we only leak once, but it's somewhat inelegant.
            Ok(Some(Box::leak(font_bytes.into_boxed_slice())))
        }
        else { Ok(None) }
    }
    else { Ok(None) }

}

fn parse_outfile(option : &VariantDict) -> Output {
    fn aux(option : &VariantDict) -> Option<PathBuf> {
        let outfile = option.lookup_value("outfile", None)?;
        outfile.try_get().ok()
    }
    aux(option).map(|path| Output::Path(path)).unwrap_or_default()
}

fn parse_format(option : &VariantDict) -> Option<Format> {
    let outfile = option.lookup_value("format", None)?;
    let format_string = outfile.try_get::<String>().ok()?;
    match format_string.as_str() {
        "svg" => Some(Format::Svg { glyph_as_text: option.lookup_value("glyphastext", None).is_some() }),
        "tex" => Some(Format::Tex),
        _     => None,
    } 
}


fn parse_styfile(option : &VariantDict) -> AppResult<Option<CommandCollection>> {
    if let Some(styfile) = option.lookup_value("styfile", None) {
        if let Ok(sty_filepath) = styfile.try_get::<PathBuf>() {
            let sty_file = std::fs::read_to_string(&sty_filepath)?;
            Ok(Some(CommandCollection::parse(&sty_file)?))
        }
        else { Ok(None) }
    }
    else { Ok(None) }
}


fn parse_font_size(option : &VariantDict) -> Option<f64> {
    let outfile = option.lookup_value("fontsize", None)?;
    let result = outfile.try_get::<f64>().unwrap();
    Some(result)
}


fn parse_in_formula(option : &VariantDict) -> Option<String> {
    let outfile = option.lookup_value("informula", None)?;
    let result = outfile.try_get::<String>().unwrap();
    Some(result)
}

fn parse_metainfo(option : &VariantDict) -> bool {
    option.lookup_value("metainfo", None).is_some()
}

pub fn handle_options(app_context : &AppContext, option : &VariantDict) -> std::ops::ControlFlow<gtk4::glib::ExitCode> {
	let AppContext {math_font,format,font_size,outfile,informula,metainfo,custom_cmd, } = app_context;
	match parse_path(option) {
	    Ok(Some(font_file)) => math_font.set(font_file),
	    Err(e) => {
	        eprintln!("{}", e);
	        // FIXME: for whatever reason, GTK ignores the exit status code here?
	        // We resort to something more brutal
	        // return 1;
	        std::process::exit(1);
	    },
	    Ok(None) => (),
	}
	*outfile.borrow_mut() = parse_outfile(option);
	if let Some(option_format) = parse_format(option) {
	    format.set(option_format);
	} 
	if let Some(font_size_arg) = parse_font_size(option) {
	    font_size.set(font_size_arg);
	} 
	if let Some(formula) = parse_in_formula(option) {
	    *informula.borrow_mut() = formula;
	} 
	match parse_styfile(option) {
	    Ok(Some(new_custom_cmd)) => *custom_cmd.borrow_mut() = new_custom_cmd,
	    Err(e) => {
	        eprintln!("{}", e);
	        // FIXME: for whatever reason, GTK ignores the exit status code here?
	        // We resort to something more brutal
	        // return 1;
	        std::process::exit(1);
	    },
	    Ok(None) => (),
	}
	if parse_metainfo(option) {
	    metainfo.set(true);
	} 
	std::ops::ControlFlow::Continue(())
}