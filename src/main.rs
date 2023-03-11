use std::cell::{RefCell, Cell};
use std::io::Write;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use cairo::ffi::FONT_SLANT_OBLIQUE;
use cairo::glib::{VariantTy, VariantDict};
use gtk::cairo::Context;


use gtk::gio::{SimpleAction, ApplicationFlags};
use gtk::glib::clone;
use gtk::subclass::scrolled_window;
use gtk::{prelude::*, TextView, DrawingArea, glib, Button};
use gtk::{Application, ApplicationWindow};
use rex::error::{FontError, LayoutError};
use rex::layout::Grid;
use rex::{Renderer, GraphicsBackend, FontBackend, Backend};
use rex::font::FontContext;
use rex::font::backend::ttf_parser::TtfMathFont;
use rex::parser::parse;

// const EXAMPLE_FORMULA : &str = r"\iint \sqrt{1 + f^2(x,t,t)}\,\mathrm{d}x\mathrm{d}y\mathrm{d}t = \sum \xi(t)";
const EXAMPLE_FORMULA : &str = r"\left.x^{x^{x^x_x}_{x^x_x}}_{x^{x^x_x}_{x^x_x}}\right\} \mathrm{wat?}";

const SVG_PATH : &str = "example.svg";
const UI_FONT_SIZE : f64 = 10.0;
const DEFAULT_FONT : &[u8] = include_bytes!("../resources/rex-xits.otf");


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


#[derive(Debug,)]
enum AppError {
    CouldNotGetTextBuffer,
    ParseError,
    IOError(std::io::Error),
    CairoError(cairo::Error),
    FontError(FontError),
    LayoutError(LayoutError),
}

type AppResult<A> = Result<A, AppError>;

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self 
    { Self::IOError(err) }
}

impl From<cairo::Error> for AppError {
    fn from(err: cairo::Error) -> Self 
    { Self::CairoError(err) }
}

impl From<FontError> for AppError {
    fn from(err: FontError) -> Self 
    { Self::FontError(err) }
}

impl From<LayoutError> for AppError {
    fn from(err: LayoutError) -> Self 
    { Self::LayoutError(err) }
}


fn main() {
    let math_font_file : Rc<Cell<& 'static [u8]>> = Rc::new(Cell::new(DEFAULT_FONT)); 
    let format         : Rc<Cell<Format>> = Rc::new(Cell::default()); 
    let font_size      : Rc<Cell<f64>>    = Rc::new(Cell::new(UI_FONT_SIZE)); 
    let outfile : Rc<RefCell<Output>> = Rc::new(RefCell::default());


    let application = Application::builder()
        .application_id("com.example.MathPreview")
        .build();

    application.add_main_option(
        "mathfont", 
        gtk::glib::Char(b'm' as i8), 
        gtk::glib::OptionFlags::IN_MAIN & gtk::glib::OptionFlags::OPTIONAL_ARG, 
        gtk::glib::OptionArg::Filename, 
        "The OpenType maths font to use for render", 
        None,
    );

    application.add_main_option(
        "outfile", 
        gtk::glib::Char(b'o' as i8), 
        gtk::glib::OptionFlags::IN_MAIN & gtk::glib::OptionFlags::OPTIONAL_ARG, 
        gtk::glib::OptionArg::Filename, 
        "Output file ; if left unspecified, output is directed to stdout", 
        None,
    );

    application.add_main_option(
        "format",
        gtk::glib::Char(b'f' as i8),
        gtk::glib::OptionFlags::IN_MAIN & gtk::glib::OptionFlags::OPTIONAL_ARG, 
        gtk::glib::OptionArg::String, 
        "Format of 'outfile' ('svg', 'tex') ; defaults to 'tex'", 
        None,
    );

    application.add_main_option(
        "fontsize",
        gtk::glib::Char(b's' as i8),
        gtk::glib::OptionFlags::IN_MAIN & gtk::glib::OptionFlags::OPTIONAL_ARG, 
        gtk::glib::OptionArg::Double, 
        "Size of font in the SVG output (default: 10)", 
        None,
    );



    application.connect_handle_local_options(clone!(@strong math_font_file, @strong outfile, @strong font_size, @strong format, => move |_application, option| {
        if let Some(font_file) = parse_path(option) {
            math_font_file.set(font_file);
        }
        *outfile.borrow_mut() = parse_outfile(option);
        if let Some(option_format) = parse_format(option) {
            format.set(option_format);
        } 
        if let Some(font_size_arg) = parse_font_size(option) {
            font_size.set(font_size_arg);
        } 
        -1
    }));
    application.connect_activate(clone!(@strong outfile => move |app| 
        build_ui(app, load_font(math_font_file.get()), font_size.get(), format.get(), outfile.clone())
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

fn parse_path(option : &VariantDict) -> Option<& 'static [u8]> {
    let mathfont = option.lookup_value("mathfont", None)?;
    let path : PathBuf = mathfont.try_get().ok()?;

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


fn parse_font_size(option : &VariantDict) -> Option<f64> {
    let outfile = option.lookup_value("fontsize", None)?;
    let result = outfile.try_get::<f64>().unwrap();
    Some(result)
}


fn build_ui(app : &Application, font : TtfMathFont<'static>, font_size : f64, format : Format, outfile : Rc<RefCell<Output>>) {
    dbg!(font_size);
    dbg!(format);
    let font = Rc::new(font);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Math Preview")
        .default_width(350)
        .default_height(70)
        .build();



    let text_field = TextView::builder()
        .vexpand(true)
        .build()
    ;

    let draw_area = DrawingArea::builder()
        .height_request(150)
        .build()
    ;

    let text_buffer = text_field.buffer().unwrap();
    text_buffer.set_text(EXAMPLE_FORMULA);
    text_buffer.select_range(&text_buffer.start_iter(), &text_buffer.end_iter());
    text_field.grab_focus();

    let last_ok_string = Rc::new(RefCell::new(EXAMPLE_FORMULA.to_string()));

    draw_area.connect_draw(clone!(@strong font, @strong text_buffer, @strong last_ok_string => move |area, context| {
        context.set_source_rgb(0.0, 0.0, 0.0);

        if let Some(text) = text_buffer.text(&text_buffer.start_iter(), &text_buffer.end_iter(), false) {
            let width  = area.allocated_width()  as f64;
            let height = area.allocated_height() as f64; 
            dbg!(
                &text,
                area.allocated_width()  as f64,
                area.allocated_height() as f64,
            );

            let result = draw_formula(text.as_str(), context, font.clone(), UI_FONT_SIZE, Some((width, height)));
            if result.is_ok() {
                let mut str_ref = last_ok_string.borrow_mut();
                str_ref.clear();
                str_ref.push_str(text.as_str());
            }
            else {
                eprintln!("error!");
                draw_formula(last_ok_string.borrow().as_str(), context, font.clone(), UI_FONT_SIZE, Some((width, height)));
            }
        }
        Inhibit(false)
    }));



    text_buffer.connect_changed(clone!(@weak draw_area => move |_text_buffer| {
        draw_area.queue_draw()
    }));

    const canvas_size : (f64, f64) = (500., 200.);

    let button = Button::with_label("Save to SVG");
    button.connect_clicked(clone!(@weak text_buffer, @strong text_buffer, @strong font, => move |_| {
        if let Some(text) = text_buffer.text(&text_buffer.start_iter(), &text_buffer.end_iter(), false) {
            let result = save_svg(&Output::Path(PathBuf::from(SVG_PATH)), text.as_str(), font.clone(), font_size);
        }
    }));

    let vbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(3)
        .margin(10)
        .build()
    ;

    let scrolled_window = gtk::ScrolledWindow::builder()
        .build()
    ;
    scrolled_window.add(&text_field);
    // \oint_C \vec{E} \cdot \mathrm{d} \vec \ell= - \frac{\mathrm{d}}{\mathrm{d}t} \left( \int_S \vec{B}\cdot\mathrm{d} \vec{S} \right)

    vbox.add(&draw_area);
    vbox.add(&scrolled_window);
    vbox.add(&button);
    window.add(&vbox);

    // window.connect_delete_event(move |_, _| {
    //     Inhibit(false)
    // });
    window.connect_delete_event(clone!(@strong text_buffer, @strong outfile, @strong font, => move |_, _| {
        save_to_output(&text_buffer, outfile.borrow().deref(), format, font.clone(), font_size).unwrap();
        Inhibit(false)
    }));

    window.show_all();
    
}

fn save_to_output(text_buffer: &gtk::TextBuffer, outfile: &Output, format : Format, font : Rc<TtfMathFont>, font_size : f64) -> AppResult<()> {
    let text = text_buffer.text(&text_buffer.start_iter(), &text_buffer.end_iter(), false).ok_or(AppError::CouldNotGetTextBuffer)?;
    eprintln!("Saving to {:?}", outfile);
    match format {
        Format::Svg => save_svg(outfile, &text, font, font_size),
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


fn save_svg(path : &Output, formula : &str, font : Rc<TtfMathFont>, font_size : f64,) -> AppResult<()> {
    let (layout, renderer, formula_bbox) = layout_and_size(font.as_ref(), font_size, formula)?;

    eprintln!("Saving to SVG!");
    let width  = formula_bbox.2;
    let height = formula_bbox.3;
    let svg_surface = gtk::cairo::SvgSurface::for_stream(width, height, path.stream()?)?;
    let context = Context::new(svg_surface)?;

    render_layout(&context, None, formula_bbox, renderer, layout)

}


fn draw_formula<'a>(formula : &str, context: &Context, font : Rc<TtfMathFont<'a>>, font_size : f64, canvas_size : Option<(f64, f64)>,) -> AppResult<()> {
    let (layout, renderer, formula_bbox) = layout_and_size(font.as_ref(), font_size, formula,)?;

    render_layout(context, canvas_size, formula_bbox, renderer, layout)
}

fn render_layout(context: &Context, canvas_size: Option<(f64, f64)>, formula_bbox: (f64, f64, f64, f64), renderer: Renderer, layout: rex::layout::Layout<TtfMathFont>) -> Result<(), AppError> {
    // let (x0, y0, x1, y1) = renderer.size(&node);
    context.save()?;
    if let Some(canvas_size) = canvas_size {
        scale_and_center(formula_bbox, context, canvas_size);
    }

    let mut backend = CairoBackend(context.clone());
    renderer.render(&layout, &mut backend);



    context.restore()?;
    Ok(())
}

fn layout_and_size<'a, 'f>(font: &'f TtfMathFont<'a>, font_size : f64, formula: &str) -> Result<(rex::layout::Layout<'f, TtfMathFont<'a>>, Renderer, (f64, f64, f64, f64)), AppError> {
    let font_context = FontContext::new(font)?;
    let parse_node = parse(formula).map_err(|_| AppError::ParseError)?;
    let layout_settings = rex::layout::LayoutSettings::new(&font_context, font_size, rex::layout::Style::Display);
    let node = rex::layout::engine::layout(&parse_node, layout_settings)?;
    let mut grid = Grid::new();
    grid.insert(0, 0, node.as_node());
    let mut layout = rex::layout::Layout::new();
    layout.add_node(grid.build());
    let renderer = Renderer::new();
    let formula_bbox = renderer.size(&layout);
    Ok((layout, renderer, formula_bbox))
}

fn scale_and_center(bbox: (f64, f64, f64, f64), context: &Context, canvas_size: (f64, f64)) {
    let (x0, y0, x1, y1) = bbox;
    let (canvas_width, canvas_height) = canvas_size;
    let width   = x1 - x0;
    let height  = y1 - y0;
    let midx = 0.5 * (x0 + x1);
    let midy = 0.5 * (y0 + y1);

    let fit_to_width  = canvas_width / width;
    let fit_to_height = canvas_height / height;
    let scale = f64::min(fit_to_width, fit_to_height);

    let tx = - (midx - 0.5 *  canvas_width / scale);
    let ty = - (midy - 0.5 *  canvas_height / scale);
    context.scale(scale, scale);
    context.translate(tx, ty);

    // draw_bbox(context, x0, y0, width, height, x1, y1);
}

fn draw_bbox(context: &Context, x0: f64, y0: f64, width: f64, height: f64, x1: f64, y1: f64) {
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

fn load_font<'a>(file : &'a [u8]) -> TtfMathFont<'a> {
    let font = ttf_parser::Face::parse(file, 0).unwrap();
    TtfMathFont::new(font).unwrap()
}


pub struct CairoBackend(Context);

impl CairoBackend {
    pub fn new(context: &Context) -> Self {
        context.set_source_rgb(1., 0., 0.);
        Self(context.clone())
    }
}

impl<'a> Backend<TtfMathFont<'a>> for CairoBackend {}


impl GraphicsBackend for CairoBackend {
    fn rule(&mut self, pos: rex::Cursor, width: f64, height: f64) {
        let context = &mut self.0;
        context.rectangle(pos.x, pos.y, width, height);
        context.fill().unwrap();
    }


    // We preliminary don't support color changes
    fn begin_color(&mut self, color: rex::RGBA) {
        unimplemented!()
    }

    fn end_color(&mut self) {
        unimplemented!()
    }
}


impl<'a> FontBackend<TtfMathFont<'a>> for CairoBackend {
    fn symbol(&mut self, pos: rex::Cursor, gid: rex::font::common::GlyphId, scale: f64, ctx: &TtfMathFont<'a>) {
        use ttf_parser::OutlineBuilder;

        let context = &mut self.0;

        context.save().unwrap();
        context.translate(pos.x, pos.y);
        context.scale(scale, -scale);
        context.scale(ctx.font_matrix().sx.into(), ctx.font_matrix().sy.into(),);
        context.set_fill_rule(gtk::cairo::FillRule::EvenOdd);
        context.new_path();

        struct Builder<'a> { 
            // path   : Path,
            // paint  : Paint,
            context : &'a mut Context,
        }

        impl<'a> Builder<'a> {
            fn fill(self) {
                self.context.fill().unwrap();
            }
        }

        impl<'a> OutlineBuilder for Builder<'a> {
            fn move_to(&mut self, x: f32, y: f32) {
                // eprintln!("move_to {:?} {:?}", x, y);
                self.context.move_to(x.into(), y.into());
            }

            fn line_to(&mut self, x: f32, y: f32) {
                // eprintln!("line_to {:?} {:?}", x, y);
                self.context.line_to(x.into(), y.into());
            }

            fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
                // eprintln!("quad_to  {:?} {:?} {:?} {:?}", x1, y1, x, y);
                self.context.curve_to(x1.into(), y1.into(), x1.into(), y1.into(), x.into(), y.into(),)
            }

            fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
                // eprintln!("curve_to {:?} {:?} {:?} {:?} {:?} {:?}", x1, y1, x2, y2, x, y);
                self.context.curve_to(x1.into(), y1.into(), x2.into(), y2.into(), x.into(), y.into(),)
            }

            fn close(&mut self) {
                // eprintln!("close");
                self.context.close_path();
            }

        }

        let mut builder = Builder {
            context: context,
        };

        ctx.font().outline_glyph(gid.into(), &mut builder);
        builder.fill();
        context.restore().unwrap();
    }
}