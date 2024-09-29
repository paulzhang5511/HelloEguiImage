// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::mpsc::{Receiver, Sender};

use eframe::{
    egui::{
        self, pos2, vec2, Align, CentralPanel, Color32, ColorImage, FontId, Image, Layout,
        RichText, Sense, ViewportBuilder, Visuals,
    },
    epaint::Hsva,
};
use image::{flat, ColorType, FlatSamples};
use log::{debug, error};
use tokio::runtime::Runtime;

fn main() -> Result<(), eframe::Error> {
    std::env::set_var("RUST_LOG", "hello_egui_image=debug");
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([360.0, 700.0]),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "hello-egui-image",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            setup_custom_style(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<MyAPP>::new(MyAPP::new()))
        }),
    );

    Ok(())
}

struct MyAPP {
    rt: Runtime,
    img_width: u32,
    img_height: u32,
    tx: Sender<Option<(u32, u32, String, ColorImage)>>,
    rx: Receiver<Option<(u32, u32, String, ColorImage)>>,
    texture_handle: Option<egui::TextureHandle>,
    loading: bool,
    init_data: bool,
    label: Option<String>,
}

impl MyAPP {
    fn new() -> Self {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            rt,
            tx,
            rx,
            img_width: 300,
            img_height: 300,
            texture_handle: None,
            loading: false,
            init_data: true,
            label: None,
        }
    }
}

impl eframe::App for MyAPP {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            if self.init_data {
                self.init_data = false;
                self.loading = true;
                self.load_image("D:/code/hello-egui-image/composeResources/drawable/1.jpg");
            }

            if let Ok(img) = self.rx.try_recv() {
                self.loading = false;
                if let Some((width, height, label, img)) = img {
                    self.img_width = width;
                    self.img_height = height;
                    self.label = Some(label);
                    if self.texture_handle.is_some() {
                        ctx.forget_all_images();
                        self.texture_handle
                            .clone()
                            .unwrap()
                            .set(img, Default::default());
                    } else {
                        let texture_handle =
                            ui.ctx().load_texture("my-image", img, Default::default());
                        self.texture_handle = Some(texture_handle);
                    }
                }
            }

            ui.horizontal(|ui| {
                if ui.button("Load Image").clicked() {
                    if !self.loading {
                        self.loading = true;
                        self.load_image("D:/code/hello-egui-image/composeResources/drawable/1.jpg");
                    }
                }
                ui.spacing();
                if ui.button("Load Other Image").clicked() {
                    if !self.loading {
                        self.loading = true;
                        self.load_image(
                            "D:/code/hello-egui-image/composeResources/drawable/Slice22.png",
                        );
                    }
                }
            });

            // 显示纹理
            if let Some(texture_handle) = &self.texture_handle {
                if let Some(label) = &self.label {
                    ui.label(label);
                }
                let img = Image::new((texture_handle.id(), texture_handle.size_vec2()));
                let img = img.shrink_to_fit();
                ui.add(img);
            }
        });
    }
}

impl MyAPP {
    fn load_image(&mut self, path: &str) {
        let tx = self.tx.clone();
        let path = path.to_string();
        self.rt.spawn(async move {
            debug!("加载图片: {}", path);
            match image::open(path) {
                Ok(img) => {
                    let img_with = img.width() as usize;
                    let img_height = img.height() as usize;
                    let color_type = img.color();
                    let (label, img) = if img.color() == ColorType::Rgba8 {
                        let img = img.to_rgba8();
                        let flat_samples = img.as_flat_samples();
                        let sample_layout = flat_samples.layout;
                        let width_stride = sample_layout.width_stride;
                        let height_stride = sample_layout.height_stride;

                        let s = format!(
                            "widthxheight={}\ncolorType: {:?}\nbounds: {:?}\nstrides_cwh: {:?}\nwidth_stride: {}\nheight_stride: {}",
                            format!("{}x{}", img_with, img_height),
                            color_type,
                            flat_samples.bounds(),
                            flat_samples.strides_cwh(),
                            width_stride,
                            height_stride
                        );
                        let img = ColorImage::from_rgba_unmultiplied(
                            [img_with, img_height],
                            flat_samples.as_slice(),
                        );
                        (s, img)
                    } else {
                        let img = img.to_rgb8();
                        let flat_samples = img.as_flat_samples();
                        let sample_layout = flat_samples.layout;
                        let width_stride = sample_layout.width_stride;
                        let height_stride = sample_layout.height_stride;

                        let s = format!(
                            "widthxheight={}\ncolorType: {:?}\nbounds: {:?}\nstrides_cwh: {:?}\nwidth_stride: {}\nheight_stride: {}",
                            format!("{}x{}", img_with, img_height),
                            color_type,
                            flat_samples.bounds(),
                            flat_samples.strides_cwh(),
                            width_stride,
                            height_stride
                        );
                        let img =
                            ColorImage::from_rgb([img_with, img_height], flat_samples.as_slice());
                        (s, img)
                    };
                    match tx.send(Some((img_with as u32, img_height as u32, label, img))) {
                        Ok(_) => {
                            debug!("发送图片数据成功");
                        }
                        Err(e) => {
                            error!("发送图片数据失败: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("加载图片失败: {:?}", e);
                    tx.send(None).unwrap();
                }
            }
        });
    }
}

fn setup_custom_style(ctx: &egui::Context) {
    ctx.style_mut(|style| {
        style.visuals = Visuals {
            dark_mode: true,
            ..Default::default()
        };
    });
    // let old = ctx.style().visuals.clone();
    // let widgets = ctx.style().visuals.widgets.clone();
    // let mut inactive = widgets.inactive.clone();
    // // inactive.weak_bg_fill = Color32::YELLOW;
    // let visuals = egui::Visuals {
    //     dark_mode: true,
    //     override_text_color: Some(Color32::RED),
    //     panel_fill: Color32::from_rgb(248, 248, 248),
    //     extreme_bg_color: Color32::GREEN,
    //     widgets: Widgets {
    //         inactive: inactive,
    //         ..widgets
    //     },
    //     ..old
    // };
    // ctx.set_visuals(visuals);
}

/// 自定义字体
fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::empty();
    fonts.font_data.insert(
        "NotoSerifSC-Regular".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../composeResources/font/NotoSerifSC-Regular.ttf"
        )),
    );
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "NotoSerifSC-Regular".to_owned());
    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .push("NotoSerifSC-Regular".to_owned());
    ctx.set_fonts(fonts);
}
