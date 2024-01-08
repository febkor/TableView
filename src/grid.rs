use std::path::Path;

use egui::{
    text::{LayoutJob, TextWrapping},
    Align, FontId, Layout, RichText, ScrollArea, Style, TextFormat, ViewportCommand,
};
use egui_extras::{Column, TableBuilder};
use polars::prelude::DataFrame;

use crate::reader;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
// #[derive(serde::Deserialize, serde::Serialize)]
// #[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    // to opt-out of serialization of a member
    // #[serde(skip)]
    data: Option<DataFrame>,

    source: Option<String>,

    input: String,

    error: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            data: None,
            source: None,
            input: "".to_string(),
            error: None,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        cc.egui_ctx.set_visuals(egui::Visuals::light());
        Default::default()
    }
}

impl eframe::App for App {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                });
            });
        });

        egui::TopBottomPanel::top("data_source").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Data Source:");

                ui.add(
                    egui::TextEdit::singleline(&mut self.input)
                        .cursor_at_end(true)
                        .hint_text("File Path")
                        .desired_width(400.0),
                );

                ui.with_layout(Layout::right_to_left(Align::RIGHT), |ui| {
                    if ui.button("Open").clicked() {
                        self.data = None;
                        self.source = Some(self.input.clone());
                    }
                });
            });
        });

        if self.source.is_none() || self.source.as_ref().is_some_and(|x| x.len() == 0) {
            egui::CentralPanel::default().show(ctx, |_ui| {});

            return;
        }

        if self.data.is_none() {
            let source = self.source.clone().expect("source should be set");
            let path = Path::new(&source);
            let result = reader::read(path);
            if let Ok(result) = result {
                self.data = Some(result);
            } else {
                self.error = Some(result.err().unwrap().to_string());
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::warn_if_debug_build(ui);

            if self.data.is_some() && self.error.is_none() {
                let data = self.data.as_ref().expect("data should be set");
                let scroll_area = ScrollArea::horizontal();
                scroll_area.show(ui, |ui| {
                    table_ui(ui, data);
                });
            } else {
                ui.label(self.error.clone().unwrap().clone());
            }
        });

        fn table_ui(ui: &mut egui::Ui, data: &DataFrame) {
            let col_names = data.get_column_names();
            let schema = data.schema();
            let col_types: Vec<&polars::prelude::DataType> = schema.iter_dtypes().collect();

            let builder = TableBuilder::new(ui)
                .striped(true)
                .auto_shrink([false, false])
                .cell_layout(Layout::default())
                .columns(Column::auto().at_least(20.0).resizable(true), data.width());

            const ROW_HEIGHT: f32 = 16.0;
            const ROW_HEIGHT_HEADER: f32 = ROW_HEIGHT * 2.0 + 2.0;
            const FONT_SIZE: f32 = 12.0;
            const FONT_SIZE_HEADER: f32 = 12.0 + 2.0;

            let layouts: Vec<Layout> = col_types
                .iter()
                .map(|dtype| {
                    Layout::default().with_cross_align(if dtype.is_numeric() {
                        Align::RIGHT
                    } else {
                        Align::LEFT
                    })
                })
                .collect();

            builder
                .header(ROW_HEIGHT_HEADER, |mut header| {
                    for i in 0..col_names.len() {
                        header.col(|ui| {
                            let label = format!("{}\n[{}]", &col_names[i], &col_types[i]);
                            ui.with_layout(
                                Layout::default().with_cross_align(Align::Center),
                                |ui| {
                                    ui.label(
                                        RichText::new(label)
                                            .font(egui::FontId::monospace(FONT_SIZE_HEADER))
                                            .strong(),
                                    )
                                },
                            );
                        });
                    }
                })
                .body(|body| {
                    let total_rows = usize::min(1000, data.height());

                    body.rows(ROW_HEIGHT, total_rows, |mut row| {
                        // TODO: avoid vec alloc
                        let row_idx = row.index();
                        let data_row = data.get_row(row_idx).expect("should get row");
                        let elements = data_row.0;

                        for col_idx in 0..elements.len() {
                            row.col(|ui| {
                                let label = elements[col_idx].to_string();
                                let color = Style::default().visuals.text_color();
                                ui.with_layout(layouts[col_idx], |ui| {
                                    let mut job = LayoutJob::single_section(
                                        label,
                                        TextFormat::simple(FontId::monospace(FONT_SIZE), color),
                                    );
                                    job.wrap = TextWrapping {
                                        max_rows: 1,
                                        break_anywhere: true,
                                        overflow_character: Some('…'),
                                        ..Default::default()
                                    };

                                    ui.label(job);
                                });
                            });
                        }
                    });
                });
        }
    }
}
