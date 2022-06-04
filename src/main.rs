use std::env::args_os;

mod pb_core;

#[cfg(feature = "gui")]
mod gui;

fn main() {
    #[cfg(feature = "gui")]
    {
        let native_options = eframe::NativeOptions::default();
        eframe::run_native(
            "My egui App",
            native_options,
            Box::new(|cc| Box::new(gui::PBGui::new(cc, args_os().nth(1).unwrap()))),
        );
    }

    #[cfg(not(feature = "gui"))]
    {
        println!("GUI not built! Exiting...")
    }
}
