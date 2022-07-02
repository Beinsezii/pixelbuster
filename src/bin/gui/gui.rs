use pixelbuster::{
    pbcore::{parse_ops, process, OpError, Space},
    pixelbuster, HELP,
};

use std::path::Path;
use std::time::{Duration, Instant};

use eframe::egui;
use eframe::{
    egui::{
        containers::{ScrollArea, Window},
        panel::{CentralPanel, SidePanel},
        text::LayoutJob,
        widgets::{DragValue, Slider, TextEdit},
        Color32, ColorImage, Context, FontId, Stroke, Style, TextFormat, TextureHandle, Visuals,
    },
    App, Frame,
};

use image::{io::Reader, DynamicImage, ImageFormat};

use rfd::FileDialog;

pub struct PBGui {
    code: String,
    code_errs: Vec<usize>,
    data: Option<(DynamicImage, TextureHandle)>,
    help: bool,
    preview: bool,
    t_pre: Duration,
    t_parse: Duration,
    t_proc: Duration,
    t_post: Duration,
    v_checks: [bool; 9],
    v_mins: [f32; 9],
    v_maxes: [f32; 9],
    externals: [f32; 9],
}

// App {{{
impl App for PBGui {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        Window::new("Help")
            .open(&mut self.help)
            .vscroll(true)
            .hscroll(true)
            .show(ctx, |ui| ui.label(HELP));
        SidePanel::right("toolbox").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Update").clicked() {
                    self.process(ctx);
                }
                if ui.button("Open").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter(
                            "Images",
                            &[
                                "avif", "bmp", "dds", "exr", "ff", "gif", "ico", "jpg", "png",
                                "pnm", "tga", "tiff", "webp",
                            ],
                        )
                        .add_filter("All Files", &["*"])
                        .pick_file()
                    {
                        self.load(ctx, path)
                    }
                }
                ui.menu_button("Export", |ui| {
                    let formats: &[(&str, fn(DynamicImage, std::path::PathBuf))] = &[
                        ("png", |img, path| {
                            img.into_rgba8()
                                .save_with_format(path, ImageFormat::Png)
                                .unwrap()
                        }),
                        ("jpg", |img, path| {
                            img.into_rgb8()
                                .save_with_format(path, ImageFormat::Jpeg)
                                .unwrap()
                        }),
                        ("exr", |img, path| {
                            img.into_rgba32f()
                                .save_with_format(path, ImageFormat::OpenExr)
                                .unwrap()
                        }),
                    ];
                    for (ext, func) in formats {
                        if ui.button(ext.to_ascii_uppercase()).clicked() {
                            if let Some((img, _tex)) = &self.data {
                                if let Some(path) = FileDialog::new()
                                    .add_filter(
                                        &format!("{} Images", ext.to_ascii_uppercase()),
                                        &[ext],
                                    )
                                    .add_filter("All Files", &["*"])
                                    .set_file_name(&format!("out.{}", ext))
                                    .save_file()
                                {
                                    let mut newimg = img.to_rgba32f();
                                    pixelbuster(
                                        &self.code,
                                        Space::LRGB,
                                        &mut newimg,
                                        img.width() as usize,
                                        None,
                                    );
                                    func(DynamicImage::from(newimg), path);
                                }
                            }
                            ui.close_menu()
                        }
                    }
                });
                ui.toggle_value(&mut self.help, "Help");
            });
            ScrollArea::vertical().show(ui, |ui| {
                let mut highlighter = |ui: &egui::Ui, text: &str, width: f32| {
                    let mut job = LayoutJob::default();
                    let mut iter = text.split('\n').enumerate();
                    let mut cur = iter.next();
                    let mut nex = iter.next();
                    loop {
                        let (n, row) = if let Some(v) = cur { v } else { break };
                        let mut row = row.to_string();
                        if nex.is_some() {
                            row.push('\n')
                        }
                        job.append(
                            row.as_str(),
                            0.0,
                            TextFormat {
                                underline: if self.code_errs.contains(&(n + 1)) {
                                    Stroke {
                                        width: 1.0,
                                        color: Color32::RED,
                                    }
                                } else {
                                    Stroke::default()
                                },
                                font_id: FontId::monospace(14.0),
                                ..Default::default()
                            },
                        );
                        cur = nex;
                        nex = iter.next();
                    }
                    job.wrap.max_width = width;
                    ui.fonts().layout_job(job)
                };
                ui.add(
                    TextEdit::multiline(&mut self.code)
                        .code_editor()
                        .layouter(&mut highlighter),
                );
                ui.horizontal(|ui| {
                    if ui.toggle_value(&mut self.preview, "Preview").clicked() {
                        self.process(ctx);
                    }
                });

                ui.columns(4, |cols| {
                    for (n, c) in cols.iter_mut().enumerate() {
                        if c.toggle_value(&mut self.v_checks[n], format!("e{}", n + 1))
                            .clicked()
                            || c.toggle_value(&mut self.v_checks[n + 4], format!("e{}", n + 5))
                                .clicked()
                        {
                            self.process(ctx)
                        }
                    }
                });

                let mut proc = false;
                for (n, b) in self.v_checks.iter().enumerate() {
                    if *b {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.add(DragValue::new(&mut self.v_mins[n]));
                                ui.label("<=");
                                ui.strong(format!("e{}", n + 1));
                                ui.label("<=");
                                ui.add(DragValue::new(&mut self.v_maxes[n]));
                            });
                            let slider = Slider::new(
                                &mut self.externals[n],
                                self.v_mins[n]..=self.v_maxes[n],
                            )
                            .smart_aim(true)
                            .clamp_to_range(false);
                            if ui.add(slider).drag_released() {
                                proc = true;
                            };
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
// App }}}

// impl PBGui {{{
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
            code_errs: Vec::new(),
            data: None,
            preview: true,
            help: false,
            t_pre: Duration::default(),
            t_parse: Duration::default(),
            t_proc: Duration::default(),
            t_post: Duration::default(),
            v_mins: [-1.0; 9],
            v_maxes: [1.0; 9],
            v_checks: [false; 9],
            externals: [0.0; 9],
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
                let width = img.width() as usize;
                let mut externals = self.externals;
                self.v_checks.iter().enumerate().for_each(|(n, v)| {
                    if !v {
                        externals[n] = 0.0
                    }
                });
                self.t_pre = Instant::now() - i_pre;

                // parse into ops
                let i_parse = Instant::now();

                let ops = parse_ops(&self.code, Space::LRGB);

                self.t_parse = Instant::now() - i_parse;

                for er in ops.1.iter() {
                    println!("{}", er);
                }
                self.code_errs = ops
                    .1
                    .into_iter()
                    .map(|oe| match oe {
                        OpError::Partial { line, .. } => line,
                        OpError::Unknown { line } => line,
                    })
                    .collect();
                // actually process
                let i_proc = Instant::now();

                process(&ops.0, &mut pixels, width, Some(externals));

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
// impl PBGui }}}
