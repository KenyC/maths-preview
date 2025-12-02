

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    desktop::main()
}

#[cfg(target_arch = "wasm32")]
fn main() {
}


#[cfg(not(target_arch = "wasm32"))]
mod desktop {
    use gtk4::prelude::{ApplicationExt, ActionMapExt, ApplicationExtManual};
    use gtk4::prelude::{GtkApplicationExt, GtkWindowExt};
    use rex::font::backend::ttf_parser::TtfMathFont;

    use gtk4::gio::SimpleAction;
    use gtk4::glib::clone;
    use gtk4::glib;
    use gtk4::Application;
    use maths_preview::desktop::error::AppResult;
    use maths_preview::desktop::ui::build_ui;
    use maths_preview::desktop::app::AppContext;
    use maths_preview::desktop::cli;

    pub fn main() {
        
        let app_context = AppContext::default();


        let application = Application::builder()
            .application_id("com.example.MathPreview")
            .build();

        cli::setup_command_line(&application);



        application.connect_handle_local_options(clone!(
                #[strong] app_context, 
                move |_application, option| {
                    cli::handle_options(&app_context, option)
        }));
        application.connect_activate(clone!(#[strong] app_context, move |app| 
            match load_font(app_context.math_font.get()) {
                Ok(font) => build_ui(app, font, app_context.clone()),
                Err(e)   => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        ));



        let action_close = SimpleAction::new("quit", None);
        action_close.connect_activate(clone!(#[weak] application, move |_, _| {
            application.windows()[0].close();
            // application.quit(); <- QUIT does not call delete window
        }));
        application.add_action(&action_close);
        application.set_accels_for_action("app.quit", &["<Primary>Q", "Escape"]);
        

        application.run();
    }



    fn load_font<'a>(file : &'a [u8]) -> AppResult<TtfMathFont<'a>> {
        let font = ttf_parser::Face::parse(file, 0)?;
        Ok(TtfMathFont::new(font)?)
    }



}