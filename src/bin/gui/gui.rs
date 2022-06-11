use pixelbuster::pbcore::{parse_ops, process_multi, Space};

use std::path::Path;
use std::time::{Duration, Instant};

use eframe::egui;
use eframe::{
    egui::{
        containers::ScrollArea,
        panel::{CentralPanel, SidePanel},
        widgets::{DragValue, Slider},
        ColorImage, Context, Style, TextureHandle, Visuals,
    },
    App, Frame,
};

use image::{io::Reader, DynamicImage};

pub struct PBGui {
    code: String,
    data: Option<(DynamicImage, TextureHandle)>,
    preview: bool,
    t_pre: Duration,
    t_parse: Duration,
    t_proc: Duration,
    t_post: Duration,
    v_checks: [bool; 9],
    v_mins: [f32; 9],
    v_maxes: [f32; 9],
    vdefaults: [f32; 9],
}

impl App for PBGui {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        SidePanel::right("code edit").show(ctx, |ui| {
            ui.heading("Code go here");
            ScrollArea::vertical().show(ui, |ui| {
                ui.code_editor(&mut self.code);
                ui.horizontal(|ui| {
                    if ui.button("Update").clicked() {
                        self.process(ctx);
                    }

                    if ui
                        .checkbox(&mut self.preview, "Preview".to_string())
                        .clicked()
                    {
                        self.process(ctx);
                    }
                });

                ui.columns(4, |cols| {
                    for (n, c) in cols.iter_mut().enumerate() {
                        if c.checkbox(&mut self.v_checks[n], (n + 1).to_string())
                            .clicked()
                            || c.checkbox(&mut self.v_checks[n + 4], (n + 5).to_string())
                                .clicked()
                        {
                            self.process(ctx)
                        }
                    }
                });

                let mut proc = false;
                for (n, b) in self.v_checks.iter().enumerate() {
                    if *b {
                        ui.horizontal(|ui| {
                            let slider = Slider::new(
                                &mut self.vdefaults[n],
                                self.v_mins[n]..=self.v_maxes[n],
                            )
                            .prefix(format!("v{}: ", n + 1))
                            .smart_aim(true)
                            .clamp_to_range(false);
                            ui.add(DragValue::new(&mut self.v_mins[n]));
                            if ui.add(slider).drag_released() {
                                proc = true;
                            };
                            ui.add(DragValue::new(&mut self.v_maxes[n]));
                        });
                    }
                }
                if proc {
                    self.process(ctx);
                }

                ui.label(format!(
                    "PRE: {:.2}ms\nPARSE: {:.2}ms\nPROC: {:.2}ms\nPOST: {:.2}ms",
                    self.t_pre.as_secs_f64() * 1000.0,
                    self.t_parse.as_secs_f64() * 1000.0,
                    self.t_proc.as_secs_f64() * 1000.0,
                    self.t_post.as_secs_f64() * 1000.0
                ));
            });
        });
        CentralPanel::default()
            .frame(
                egui::containers::Frame::window(&ctx.style())
                    .inner_margin(0.0)
                    .outer_margin(0.0),
            )
            .show(ctx, |ui| {
                match ctx.input().raw.dropped_files.get(0) {
                    Some(f) => {
                        if let Some(p) = &f.path {
                            self.load(ctx, p)
                        }
                    }
                    None => (),
                }
                if let Some((img, tex)) = self.data.as_ref() {
                    let s = ctx.available_rect().size();
                    let (w, h) = (img.width() as f32, img.height() as f32);
                    let scale = (w / s.x).max(h / s.y);
                    ui.image(tex, &[w / scale, h / scale]);
                }
            });
    }
}

impl PBGui {
    pub fn new<P: AsRef<Path>>(cc: &eframe::CreationContext<'_>, path: Option<P>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        cc.egui_ctx.set_style(Style {
            visuals: Visuals {
                dark_mode: true,
                ..Default::default()
            },
            ..Default::default()
        });

        let mut result = Self {
            code: String::new(),
            data: None,
            preview: true,
            t_pre: Duration::default(),
            t_parse: Duration::default(),
            t_proc: Duration::default(),
            t_post: Duration::default(),
            v_mins: [-1.0; 9],
            v_maxes: [1.0; 9],
            v_checks: [false; 9],
            vdefaults: [0.0; 9],
        };

        if let Some(p) = path {
            result.load(&cc.egui_ctx, p)
        }

        result
    }

    fn load<P: AsRef<Path>>(&mut self, ctx: &Context, path: P) {
        if let Some(data) = Reader::open(path)
            .ok()
            .map(|r| r.with_guessed_format().ok())
            .flatten()
            .map(|r| r.decode().ok())
            .flatten()
            .map(move |img| {
                let ctx = ctx.load_texture(
                    "img",
                    ColorImage::from_rgba_unmultiplied(
                        [img.width() as usize, img.height() as usize],
                        &img.to_rgba8(),
                    ),
                );
                Some((img, ctx))
            })
        {
            self.data = data;
            self.process(ctx);
        }
    }

    // TODO: Half/Quarter res preview.
    fn process(&mut self, ctx: &Context) {
        if let Some((img, tex)) = self.data.as_mut() {
            if self.preview {
                // fetch data
                let i_pre = Instant::now();
                let mut pixels = img.to_rgba32f();
                let mut vdefaults = self.vdefaults;
                self.v_checks.iter().enumerate().for_each(|(n, v)| {
                    if !v {
                        vdefaults[n] = 0.0
                    }
                });
                self.t_pre = Instant::now() - i_pre;

                // parse into ops
                let i_parse = Instant::now();

                let ops = parse_ops(&self.code, Space::SRGB);

                self.t_parse = Instant::now() - i_parse;

                for er in ops.1 {
                    println!("{}", er);
                }
                // actually process
                let i_proc = Instant::now();

                process_multi(&ops.0, &mut pixels, Some(vdefaults));

                self.t_proc = Instant::now() - i_proc;

                // post process aka convert into texture readable data
                let i_post = Instant::now();
                let pixels = pixels
                    .into_iter()
                    .map(|p| (p * 255.0) as u8)
                    .collect::<Vec<u8>>();
                self.t_post = Instant::now() - i_post;

                *tex = ctx.load_texture(
                    "img",
                    ColorImage::from_rgba_unmultiplied(
                        [img.width() as usize, img.height() as usize],
                        pixels.as_ref(),
                    ),
                );
            } else {
                *tex = ctx.load_texture(
                    "img",
                    ColorImage::from_rgba_unmultiplied(
                        [img.width() as usize, img.height() as usize],
                        img.to_rgba8().as_ref(),
                    ),
                )
            }
        }
    }
}
