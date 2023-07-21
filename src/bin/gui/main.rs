use std::env::args_os;

#[cfg(feature = "gui")]
mod gui;

fn main() -> eframe::Result<()> {
    #[cfg(feature = "gui")]
    {
        let native_options = eframe::NativeOptions::default();
        eframe::run_native(
            "Pixelbuster GUI",
            native_options,
            Box::new(|cc| Box::new(gui::PBGui::new(cc, args_os().nth(1)))),
        )
    }

    #[cfg(not(feature = "gui"))]
    {
        println!("GUI not built! Exiting...")
    }
}
