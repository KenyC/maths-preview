use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use gtk4::prelude::{ActionMapExt, EditableExtManual, DrawingAreaExtManual};
use gtk4::prelude::{GtkApplicationExt, GtkWindowExt, EditableExt, WidgetExt, BoxExt};
use gtk4::gio::SimpleAction;
use gtk4::glib::clone;
use gtk4::{DrawingArea, glib, Statusbar, Entry};
use gtk4::{Application, ApplicationWindow};
use rex::font::backend::ttf_parser::TtfMathFont;


use crate::cli::{EXAMPLE_FORMULA, UI_FONT_SIZE};
use crate::render::draw_formula;
use crate::undo::{UndoStack, get_selection};


use crate::app::{save_to_output, AppContext};


struct Ui {
    window : ApplicationWindow, 
    draw_area : DrawingArea, 
    text_field : Entry, 
    status_bar : Statusbar,
}



pub fn build_ui(app : &Application, font : TtfMathFont<'static>, app_context : AppContext) {
    let AppContext { format, font_size, outfile, informula, metainfo, custom_cmd, .. } = app_context;
    let format     = format.get();
    let metainfo   = metainfo.get();
    let font_size  = font_size.get();
    let font = Rc::new(font);

    let Ui { window, draw_area, text_field, status_bar, } = construct_widgets(app, informula);




    let undo_stack = Rc::new(RefCell::new(UndoStack::new()));
    setup_undo_actions(app, undo_stack.clone(), text_field.clone());
    let last_ok_string = Rc::new(RefCell::new(EXAMPLE_FORMULA.to_string()));

    draw_area.set_draw_func(clone!(#[strong] font, #[strong] text_field, #[strong] last_ok_string, #[strong] status_bar, #[strong] custom_cmd, move |_area, context, width, height| {
        let text = text_field.text();
        context.set_source_rgb(0.0, 0.0, 0.0);

        let width  = width   as f64;
        let height = height  as f64; 

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
                eprintln!("{}", error);
                status_bar.push(0, &format!("{}", error));
                draw_formula(last_ok_string.borrow().as_str(), context, font.clone(), UI_FONT_SIZE, Some((width, height)), custom_cmd.borrow().deref()).unwrap_or(());
            },
        }
    }));


    text_field.connect_changed(clone!(#[weak] draw_area, move |_text_buffer| {
        draw_area.queue_draw()
    }));
    text_field.delegate().unwrap().connect_insert_text(clone!(#[strong] undo_stack, move |entry, text, pt| {
        let selection = get_selection(entry);
        undo_stack.borrow_mut().insert_text(text, *pt, selection);
    }));
    text_field.delegate().unwrap().connect_delete_text(clone!(#[strong] undo_stack, move |entry, start_pos, end_pos| {
        let deleted_text = entry.chars(start_pos, end_pos);
        let selection = get_selection(entry);
        undo_stack.borrow_mut().delete_text(deleted_text.as_str(), start_pos, end_pos, selection);
    }));


    window.connect_close_request(clone!(#[strong] text_field, #[strong] outfile, #[strong] font, #[strong] custom_cmd, move |_| {
        let text = text_field.text();
        // TODO: error handling
        // Can't really see how to set an exit status code once the app is running
        save_to_output(&text, outfile.borrow().deref(), format, font.clone(), font_size, metainfo, custom_cmd.borrow().deref()).unwrap();
        glib::signal::Propagation::Proceed
    }));

    window.show();
    
}

fn construct_widgets(app: &Application, informula: Rc<RefCell<String>>) -> Ui {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Math Preview")
        .default_width(350)
        .default_height(70)
        .build();

    let draw_area = DrawingArea::builder()
        .height_request(250)
        .vexpand(true)
        .margin_top(3)
        .margin_bottom(3)
        .margin_start(3)
        .margin_end(3)
        .build()
    ;

    let status_bar = Statusbar::builder()
        .build()
    ;
    status_bar.push(0, "Loading ...");

    let text_field = Entry::builder()
        .valign(gtk4::Align::Center)
        .build()
    ;
    text_field.select_region(0, text_field.selection_bound());
    text_field.grab_focus();
    text_field.set_text(informula.borrow().as_str());

    let scrolled_window = gtk4::ScrolledWindow::builder()
        .valign(gtk4::Align::Start)
        .build()
    ;
    scrolled_window.set_child(Some(&text_field));



    let vbox = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(0)
        .margin_start(0)
        .margin_end(0)
        .margin_top(0)
        .margin_bottom(0)
        .build()
    ;

    vbox.append(&scrolled_window);
    vbox.append(&draw_area);
    vbox.append(&status_bar);
    window.set_child(Some(&vbox));
    Ui { window, draw_area, text_field, status_bar }
}

fn setup_undo_actions(app: &Application, undo_stack : Rc<RefCell<UndoStack>>, text_field : Entry) {
    let undo_action = SimpleAction::new("undo", None);
    let redo_action = SimpleAction::new("redo", None);

    app.add_action(&undo_action);
    app.add_action(&redo_action);
    app.set_accels_for_action("app.undo", &["<Ctrl>Z"]);
    app.set_accels_for_action("app.redo", &["<Ctrl><Shift>Z"]);


    undo_action.connect_activate(clone!(#[strong] text_field, #[strong] undo_stack, move |_, _| {
        undo_stack.borrow_mut().undo(text_field.clone());
    }));

    redo_action.connect_activate(clone!(#[strong] text_field, #[strong] undo_stack, move |_, _| {
        undo_stack.borrow_mut().redo(text_field.clone());
    }));
}
