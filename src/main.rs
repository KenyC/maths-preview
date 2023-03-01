use std::cell::Cell;
use std::rc::Rc;

use gtk::cairo::Context;
use gtk::gdk::keys::Key;
use gtk::gdk::{key, EventKey};
use gtk::glib::clone;
use gtk::{prelude::*, TextView, DrawingArea, glib};
use gtk::{Application, ApplicationWindow, Button};
use rex::{Renderer, GraphicsBackend, FontBackend, Backend};
use rex::font::FontContext;
use rex::font::backend::ttf_parser::TtfMathFont;
use rex::parser::parse;

const EXAMPLE_FORMULA : &str = r"\iint \sqrt{1 + f^2(x,t,t)}\,\mathrm{d}x\mathrm{d}y\mathrm{d}t = \sum \xi(t)";
// const EXAMPLE_FORMULA : &str = r"\left.x^{x^{x^x_x}_{x^x_x}}_{x^{x^x_x}_{x^x_x}}\right\} \mathrm{wat?}";


fn main() {
    let math_font_file = Box::leak(std::fs::read("resources/rex-xits.otf").unwrap().into_boxed_slice());
    let font = Rc::new(load_font(math_font_file));


    let application = Application::builder()
        .application_id("com.example.FirstGtkApp")
        .build();

    application.connect_activate(clone!(@strong font => move |app| build_ui(app, font.clone())));

    application.run();
}

fn build_ui(app : &Application, font : Rc<TtfMathFont<'static>>) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("First GTK Program")
        .default_width(350)
        .default_height(70)
        .build();

    let mut color = Rc::new(Cell::new(0.5));



    let text_field = TextView::builder()
        .vexpand(true)
        .build()
    ;

    let draw_area = DrawingArea::builder()
        .height_request(150)
        .build()
    ;

    // let new_color = color.clone();
    draw_area.connect_draw(clone!(@strong color, @strong font => move |area, context| {
        // context.set_source_rgb(color.get(), 0.0, 0.0);
        // context.rectangle(0., 0., 50., 50.);
        // context.fill().unwrap();
        draw_formula(EXAMPLE_FORMULA, context, font.clone(), area.allocated_width() as f64);
        Inhibit(false)
    }));


    let button = Button::with_label("Click me!");
    button.connect_clicked(clone!(@weak color, @weak draw_area => move |_| {
        color.set(1.0);
        draw_area.queue_draw();
        eprintln!("Clicked!");
    }));


    let text_buffer = text_field.buffer().unwrap();
    text_buffer.set_text("\\frac{1}{2}");


    let vbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build()
    ;

    vbox.add(&draw_area);
    vbox.add(&button);
    vbox.add(&text_field);
    window.add(&vbox);

    window.connect_key_press_event(|window, key| {
        if key.keyval() == gtk::gdk::keys::constants::Escape {
            window.application().unwrap().quit();
        }
        Inhibit(false)
    });

    window.show_all();
    
}




fn draw_formula<'a>(formula : &str, context: &Context, font : Rc<TtfMathFont<'a>>, canvas_width : f64) {
    let font_context = FontContext::new(font.as_ref()).unwrap();
    let parse_node = parse(formula).unwrap();
    let layout_settings = rex::layout::LayoutSettings::new(&font_context, 10.0, rex::layout::Style::Display);
    let node = rex::layout::engine::layout(&parse_node, layout_settings).unwrap();

    let renderer = Renderer::new();

    let (x0, y0, x1, y1) = renderer.size(&node);
    let width   = x1 - x0;
    let height  = y1 - y0;
    context.save();
    context.translate(x0, y0);
    let min_scale = canvas_width / width;
    context.scale(min_scale, min_scale);
    context.translate(0., height);

    // let context = context.clone();
    // context.set_source_rgb(1., 0., 0.);
    // context.rectangle(10., 10., 10., 10.);
    // context.fill().unwrap();

    let mut backend = CairoBackend(context.clone());
    renderer.render(&node, &mut backend);

    context.set_source_rgb(1., 0., 0.);
    context.rectangle(x0, y0, width, height);
    context.stroke().unwrap();
    context.restore();
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

    fn begin_color(&mut self, color: rex::RGBA) {
        todo!()
    }

    fn end_color(&mut self) {
        todo!()
    }
}


impl<'a> FontBackend<TtfMathFont<'a>> for CairoBackend {
    fn symbol(&mut self, pos: rex::Cursor, gid: rex::font::common::GlyphId, scale: f64, ctx: &TtfMathFont<'a>) {
        use ttf_parser::OutlineBuilder;

        let context = &mut self.0;

        context.save().unwrap();
        context.translate(pos.x, pos.y);
        // context.scale(0.1, 0.1);
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
                println!("move_to {:?} {:?}", x, y);
                self.context.move_to(x.into(), y.into());
            }

            fn line_to(&mut self, x: f32, y: f32) {
                println!("line_to {:?} {:?}", x, y);
                self.context.line_to(x.into(), y.into());
            }

            fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
                println!("quad_to  {:?} {:?} {:?} {:?}", x1, y1, x, y);
                self.context.curve_to(x1.into(), y1.into(), x1.into(), y1.into(), x.into(), y.into(),)
            }

            fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
                println!("curve_to {:?} {:?} {:?} {:?} {:?} {:?}", x1, y1, x2, y2, x, y);
                self.context.curve_to(x1.into(), y1.into(), x2.into(), y2.into(), x.into(), y.into(),)
            }

            fn close(&mut self) {
                println!("close");
                self.context.close_path();
            }

        }

        let mut builder = Builder {
            context: context,
        };

        ctx.font().outline_glyph(gid.into(), &mut builder);
        builder.fill();
        // context.rectangle(0., 0., 10., 10.);
        // context.fill().unwrap();
        context.restore().unwrap();
    }
}