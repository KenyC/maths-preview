pub mod undo_stack;
pub mod error;
pub mod geometry;
mod render;

use std::cell::{RefCell, Cell};
use std::io::Write;
use std::ops::Deref;
use std::path::PathBuf;
use std::rc::Rc;

use rex::parser::macros::CommandCollection;
use serde_json;

use cairo::glib::VariantDict;
use gtk::cairo::Context;
use gtk::gio::SimpleAction;
use gtk::glib::clone;
use gtk::{prelude::*, DrawingArea, glib, Statusbar, Entry};
use gtk::{Application, ApplicationWindow};

use rex::font::backend::ttf_parser::TtfMathFont;

use error::AppResult;
use undo_stack::{UndoStack, get_selection};
use render::{render_layout, layout_and_size, draw_formula, MetaInfo};
use geometry::Metrics;


// const EXAMPLE_FORMULA : &str = r"\iint \sqrt{1 + f^2(x,t,t)}\,\mathrm{d}x\mathrm{d}y\mathrm{d}t = \sum \xi(t)";
const EXAMPLE_FORMULA : &str = r"\left.x^{x^{x^x_x}_{x^x_x}}_{x^{x^x_x}_{x^x_x}}\right\} \mathrm{wat?}";

const SVG_PATH : &str = "example.svg";
const UI_FONT_SIZE : f64 = 10.0;
// const DEFAULT_FONT : &[u8] = include_bytes!("../resources/rex-xits.otf");
const DEFAULT_FONT : &[u8] = include_bytes!("../resources/LibertinusMath-Regular.otf");


#[derive(Debug, Clone, Copy)]
enum Format {
    Svg, 
    Tex,
}

impl Default for Format {
    fn default() -> Self 
    { Self::Tex }
}


#[derive(Debug,)]
enum Output {
    Stdout,
    Path(PathBuf),
}

impl Output {
    fn stream(&self) -> std::io::Result<Box<dyn Write + 'static>> {
        match self {
            Output::Stdout     => Ok(Box::new(std::io::stdout())),
            Output::Path(path) => Ok(Box::new(std::fs::File::create(path)?)),
        }
    } 
}

impl Default for Output {
    fn default() -> Self { Self::Stdout }
}




#[derive(Clone)]
struct AppContext {
    math_font  : Rc<Cell<& 'static [u8]>>,
    format     : Rc<Cell<Format>>,
    font_size  : Rc<Cell<f64>>,
    custom_cmd : Rc<RefCell<CommandCollection>>,
    outfile    : Rc<RefCell<Output>>,
    informula  : Rc<RefCell<String>>,
    metainfo   : Rc<Cell<bool>>,
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


fn main() {
    let app_context = AppContext::default();


    let application = Application::builder()
        .application_id("com.example.MathPreview")
        .build();

    setup_command_line(&application);



    application.connect_handle_local_options(clone!(
            @strong app_context, 
            => move |_application, option| {
        let AppContext { math_font, format, font_size, outfile, informula, metainfo, custom_cmd } = &app_context;
        if let Some(font_file) = parse_path(option) {
            math_font.set(font_file);
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
        if let Some(new_custom_cmd) = parse_styfile(option) {
            *custom_cmd.borrow_mut() = new_custom_cmd;
        } 
        if parse_metainfo(option) {
            metainfo.set(true);
        } 
        -1
    }));
    application.connect_activate(clone!(@strong app_context => move |app| 
        let font = {
            load_font(app_context.math_font.get())
        };
        build_ui(app, font, app_context.clone())
    ));



    let action_close = SimpleAction::new("quit", None);
    action_close.connect_activate(clone!(@weak application => move |_, _| {
        application.windows()[0].close();
        // application.quit(); <- QUIT does not call delete window
    }));
    application.add_action(&action_close);
    application.set_accels_for_action("app.quit", &["<Ctrl>Q", "Escape"]);
    

    application.run();
}

fn setup_command_line(application: &Application) {
    application.add_main_option(
        "mathfont", 
        gtk::glib::Char(b'm' as i8), 
        gtk::glib::OptionFlags::IN_MAIN, 
        gtk::glib::OptionArg::Filename, 
        "Path to an OpenType maths font to use for render (default: STIX Maths, bundled in the executable)", 
        None,
    );

    application.add_main_option(
        "informula", 
        gtk::glib::Char(b'i' as i8), 
        gtk::glib::OptionFlags::IN_MAIN, 
        gtk::glib::OptionArg::String, 
        &format!("Formula to edit (default: ${}$)", EXAMPLE_FORMULA), 
        None,
    );

    application.add_main_option(
        "outfile", 
        gtk::glib::Char(b'o' as i8), 
        gtk::glib::OptionFlags::IN_MAIN, 
        gtk::glib::OptionArg::Filename, 
        "Output file ; if left unspecified, output is directed to stdout.", 
        None,
    );

    application.add_main_option(
        "styfile", 
        gtk::glib::Char(b'y' as i8), 
        gtk::glib::OptionFlags::IN_MAIN, 
        gtk::glib::OptionArg::Filename, 
        "Reads a style file to provide custom command", 
        None,
    );


    application.add_main_option(
        "metainfo", 
        gtk::glib::Char(b'd' as i8), 
        gtk::glib::OptionFlags::IN_MAIN,
        gtk::glib::OptionArg::None, 
        "Whether to output some meta-info on stdout (baseline position, font size, formula, etc.). If 'outfile' is not specified and this option is used, stdout will contain both the output and the meta-info", 
        None,
    );

    application.add_main_option(
        "format",
        gtk::glib::Char(b'f' as i8),
        gtk::glib::OptionFlags::IN_MAIN, 
        gtk::glib::OptionArg::String, 
        "Format of 'outfile' ('svg', 'tex') ; defaults to 'tex'.", 
        None,
    );

    application.add_main_option(
        "fontsize",
        gtk::glib::Char(b's' as i8),
        gtk::glib::OptionFlags::IN_MAIN, 
        gtk::glib::OptionArg::Double, 
        "Size of font in the SVG output (default: 10)", 
        None,
    );
}

fn parse_path(option : &VariantDict) -> Option<& 'static [u8]> {
    let mathfont = option.lookup_value("mathfont", None)?;
    let path : PathBuf = mathfont.try_get().ok()?;

    // TODO: deal with failure to read file elegantly
    let font_bytes = std::fs::read(&path).unwrap();
    // TODO: find a more elegant way to deal with lifetimes.
    // The lifetime in TtfMathFont & the requirement that closures fed to GTK are 'static come in conflict.
    // We leak the memory of the box so as to get a 'static reference.
    // This is ok, because we only leak once, but it's somewhat inelegant.
    Some(Box::leak(font_bytes.into_boxed_slice()))
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
        "svg" => Some(Format::Svg),
        "tex" => Some(Format::Tex),
        _     => None,
    } 
}


fn parse_styfile(option : &VariantDict) -> Option<CommandCollection> {
    let styfile = option.lookup_value("styfile", None)?;
    let sty_filepath = styfile.try_get::<PathBuf>().ok()?;
    let sty_file = std::fs::read_to_string(&sty_filepath).unwrap();
    Some(CommandCollection::parse(&sty_file).unwrap())
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


fn build_ui(app : &Application, font : TtfMathFont<'static>, app_context : AppContext) {
    let AppContext { format, font_size, outfile, informula, metainfo, custom_cmd, .. } = app_context;
    let format     = format.get();
    let metainfo   = metainfo.get();
    let font_size  = font_size.get();
    dbg!(font_size);
    dbg!(format);
    let font = Rc::new(font);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Math Preview")
        .default_width(350)
        .default_height(70)
        .build();



    let text_field = Entry::builder()
        .valign(gtk::Align::Center)
        .build()
    ;

    let draw_area = DrawingArea::builder()
        .height_request(250)
        .expand(true)
        .margin(3)
        .build()
    ;

    let status_bar = Statusbar::builder()
        .build()
    ;
    text_field.select_region(0, text_field.selection_bound());
    text_field.grab_focus();
    text_field.set_text(informula.borrow().as_str());
    // let text_buffer = text_field.buffer();
    let undo_stack = Rc::new(RefCell::new(UndoStack::new()));


    let save_svg_action = SimpleAction::new("save-svg", None);
    // let button = Button::with_label("Save to SVG");
    save_svg_action.connect_activate(clone!(@strong text_field, @strong font, @strong custom_cmd => move |_, _| {
        let text = text_field.text();
        // TODO : error handling
        let result = save_svg(&Output::Path(PathBuf::from(SVG_PATH)), text.as_str(), font.clone(), font_size, custom_cmd.borrow().deref());
        result.unwrap();
    }));

    let undo_action = SimpleAction::new("undo", None);
    let redo_action = SimpleAction::new("redo", None);

    app.add_action(&undo_action);
    app.add_action(&redo_action);
    app.set_accels_for_action("app.undo", &["<Ctrl>Z"]);
    app.set_accels_for_action("app.redo", &["<Ctrl><Shift>Z"]);


    undo_action.connect_activate(clone!(@strong text_field, @strong undo_stack => move |_, _| {
        undo_stack.borrow_mut().undo(text_field.clone());
    }));

    redo_action.connect_activate(clone!(@strong text_field, @strong undo_stack => move |_, _| {
        undo_stack.borrow_mut().redo(text_field.clone());
    }));

    let vbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(0)
        .margin(0)
        .build()
    ;

    let scrolled_window = gtk::ScrolledWindow::builder()
        .valign(gtk::Align::Start)
        .build()
    ;
    scrolled_window.add(&text_field);
    // \oint_C \vec{E} \cdot \mathrm{d} \vec \ell= - \frac{\mathrm{d}}{\mathrm{d}t} \left( \int_S \vec{B}\cdot\mathrm{d} \vec{S} \right)

    status_bar.push(0, "Loading ...");


    vbox.add(&scrolled_window);
    vbox.add(&draw_area);
    vbox.pack_start(&status_bar, false, true, 0);
    window.add(&vbox);

    let last_ok_string = Rc::new(RefCell::new(EXAMPLE_FORMULA.to_string()));

    draw_area.connect_draw(clone!(@strong font, @strong text_field, @strong last_ok_string, @strong status_bar, @strong custom_cmd => move |area, context| {
        let text = text_field.text();
        context.set_source_rgb(0.0, 0.0, 0.0);

        let width  = area.allocated_width()  as f64;
        let height = area.allocated_height() as f64; 

        let result = draw_formula(text.as_str(), context, font.clone(), UI_FONT_SIZE, Some((width, height)), custom_cmd.borrow().deref());
        match result {
            Ok(_)  => {
                status_bar.pop(0);
                status_bar.hide();
                let mut str_ref = last_ok_string.borrow_mut();
                str_ref.clear();
                str_ref.push_str(text.as_str());
            },
            Err(error) => {
                status_bar.pop(0);
                status_bar.show();
                let error_string = error.human_readable();
                eprintln!("{}", error_string);
                status_bar.push(0, &error_string);
                draw_formula(last_ok_string.borrow().as_str(), context, font.clone(), UI_FONT_SIZE, Some((width, height)), custom_cmd.borrow().deref()).unwrap_or(());
            },
        }
        Inhibit(false)
    }));


    text_field.connect_changed(clone!(@weak draw_area => move |_text_buffer| {
        draw_area.queue_draw()
    }));
    text_field.connect_insert_text(clone!(@strong undo_stack => move |entry, text, pt| {
        let selection = get_selection(&entry);
        undo_stack.borrow_mut().insert_text(text, *pt, selection);
    }));
    text_field.connect_delete_text(clone!(@strong undo_stack => move |entry, start_pos, end_pos| {
        let deleted_text = entry.chars(start_pos, end_pos).unwrap();
        let selection = get_selection(&entry);
        undo_stack.borrow_mut().delete_text(deleted_text.as_str(), start_pos, end_pos, selection);
    }));


    window.connect_delete_event(clone!(@strong text_field, @strong outfile, @strong font, @strong custom_cmd => move |_, _| {
        let text = text_field.text();
        save_to_output(&text, outfile.borrow().deref(), format, font.clone(), font_size, metainfo, custom_cmd.borrow().deref()).unwrap();
        Inhibit(false)
    }));

    window.show_all();
    
}

fn save_to_output(text: &str, outfile: &Output, format : Format, font : Rc<TtfMathFont>, font_size : f64, print_metainfo : bool, custom_cmd : &CommandCollection) -> AppResult<()> {
    eprintln!("Saving to {:?}", outfile);

    match format {
        Format::Svg => {
            let metrics = save_svg(outfile, &text, font, font_size, custom_cmd)?;
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
    if let Output::Path(outfile_path) = outfile {
        let result = std::fs::write(outfile_path, text)?;
        Ok(result)
    }
    else {
        println!("{}", text);
        Ok(())
    }
}


fn save_svg(path : &Output, formula : &str, font : Rc<TtfMathFont>, font_size : f64, custom_cmd : &CommandCollection) -> AppResult<Metrics> {
    let (layout, formula_metrics) = layout_and_size(font.as_ref(), font_size, formula, custom_cmd,)?;

    eprintln!("Saving to SVG!");
    let formula_bbox = &formula_metrics.bbox;
    let width  = formula_bbox.width();
    let height = formula_bbox.height();
    let svg_surface = gtk::cairo::SvgSurface::for_stream(width, height, path.stream()?)?;
    let context = Context::new(svg_surface)?;
    // In Cairo SVG, we aren't at a liberty to specify the view box, only height and width
    // So we must translate so that the minimum y is 0
    context.translate(0., - formula_bbox.y_min);

    render_layout(&context, None, &formula_metrics, layout)?;
    Ok(formula_metrics)

}



fn load_font<'a>(file : &'a [u8]) -> TtfMathFont<'a> {
    let font = ttf_parser::Face::parse(file, 0).unwrap();
    TtfMathFont::new(font).unwrap()
}

