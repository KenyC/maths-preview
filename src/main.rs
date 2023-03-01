use std::cell::Cell;
use std::rc::Rc;

use gtk::cairo::Context;
use gtk::gdk::keys::Key;
use gtk::gdk::{key, EventKey};
use gtk::glib::clone;
use gtk::{prelude::*, TextView, DrawingArea, glib};
use gtk::{Application, ApplicationWindow, Button};

fn main() {
    let application = Application::builder()
        .application_id("com.example.FirstGtkApp")
        .build();

    application.connect_activate(build_ui);

    application.run();
}

fn build_ui(app : &Application) {
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
    draw_area.connect_draw(clone!(@strong color => move |_area, context| {
        context.set_source_rgb(color.get(), 0.0, 0.0);
        context.rectangle(0., 0., 50., 50.);
        context.fill().unwrap();
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

fn draw_formula(formula : &str, context: &Context) {

}