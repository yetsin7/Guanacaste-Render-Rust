//! Help screen: every control of the application grouped by what it is for, with a search box, and a small
//! diagnostics card with real facts about this session (version, graphics, threads, open project).

use super::ScreenContext;
use crate::components::icon_button::IconButton;
use crate::components::search_field::SearchField;
use crate::components::{icons, text};
use crate::config::theme::{colors, typography};
use crate::features::help::components::shortcut_group;
use crate::features::help::logic::controls;
use crate::features::help::models::{Diagnostics, HelpState, Shortcut};
use crate::models::navigation::Screen;
use crate::services::localization::{tr, trf};
use eframe::egui::{self, vec2, CornerRadius, Margin, ScrollArea, Ui};

const MAX_WIDTH: f32 = 1180.0;

/// State of the help screen.
#[derive(Default)]
pub struct HelpScreen {
    state: HelpState,
}

impl HelpScreen {
    /// Draws the screen; `diagnostics` are the facts of the running session.
    pub fn show(&mut self, ui: &mut Ui, context: &mut ScreenContext, diagnostics: &Diagnostics) {
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Frame::new()
                    .inner_margin(Margin::same(12))
                    .show(ui, |ui| {
                        let total = ui.available_width();
                        let width = total.min(MAX_WIDTH);
                        ui.horizontal_top(|ui| {
                            ui.add_space((total - width) / 2.0);
                            ui.vertical(|ui| {
                                ui.set_width(width);
                                ui.spacing_mut().item_spacing.y = 12.0;
                                self.header(ui, context);
                                self.groups(ui, width);
                                diagnostics_card(ui, diagnostics);
                            });
                        });
                    });
            });
    }

    fn header(&mut self, ui: &mut Ui, context: &mut ScreenContext) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            if IconButton::new(icons::CLOSE)
                .sizes(16.0, 28.0)
                .tooltip(tr("ctl.back"))
                .show(ui)
                .clicked()
            {
                *context.navigate = Some(Screen::Workspace(crate::models::navigation::Mode::Build));
            }
            text::label(
                ui,
                tr("ctl.title"),
                typography::headline_sm(),
                colors::ON_SURFACE,
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                SearchField::new(tr("ctl.search"))
                    .width(256.0)
                    .height(28.0)
                    .font(typography::body_sm(), 14.0)
                    .show(ui, &mut self.state.search);
            });
        });
        text::paragraph(
            ui,
            tr("ctl.subtitle"),
            typography::body_sm(),
            colors::OUTLINE,
            18.0,
        );
    }

    fn groups(&mut self, ui: &mut Ui, width: f32) {
        let gap = 12.0;
        let columns = if width < 760.0 { 1 } else { 2 };
        let column = (width - gap * (columns as f32 - 1.0)) / columns as f32;
        let cards: [(&'static str, char, Vec<Shortcut>); 4] = [
            ("ctl.group_navigation", icons::MOVE, controls::navigation()),
            ("ctl.group_cameras", icons::CAMERA, controls::cameras()),
            ("ctl.group_objects", icons::NATURE, controls::objects()),
            (
                "ctl.group_application",
                icons::KEYBOARD,
                controls::application(),
            ),
        ];
        let search = self.state.search.clone();
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing = vec2(gap, gap);
            for lane in 0..columns {
                ui.allocate_ui_with_layout(
                    vec2(column, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_width(column);
                        ui.spacing_mut().item_spacing.y = gap;
                        for (index, (title, glyph, rows)) in cards.iter().enumerate() {
                            if index % columns == lane {
                                let ink = if index % 2 == 0 {
                                    colors::PRIMARY
                                } else {
                                    colors::TERTIARY
                                };
                                shortcut_group::show(ui, title, ink, *glyph, rows, &search);
                            }
                        }
                    },
                );
            }
        });
    }
}

fn diagnostics_card(ui: &mut Ui, diagnostics: &Diagnostics) {
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 4.0;
            text::label(
                ui,
                text::caps(tr("ctl.diag")),
                typography::mono_medium(11.0),
                colors::SECONDARY,
            );
            let line = |ui: &mut Ui, value: String| {
                text::label(ui, value, typography::body_sm(), colors::ON_SURFACE);
            };
            line(ui, trf("ctl.diag_version", &[env!("CARGO_PKG_VERSION")]));
            line(
                ui,
                tr(if diagnostics.gpu {
                    "ctl.diag_gpu_yes"
                } else {
                    "ctl.diag_gpu_no"
                })
                .to_owned(),
            );
            line(
                ui,
                trf("ctl.diag_threads", &[&diagnostics.threads.to_string()]),
            );
            match &diagnostics.project {
                Some(name) => {
                    line(ui, trf("ctl.diag_project", &[name]));
                    line(
                        ui,
                        trf(
                            "ctl.diag_geometry",
                            &[
                                &diagnostics.meshes.to_string(),
                                &diagnostics.triangles.to_string(),
                            ],
                        ),
                    );
                    line(
                        ui,
                        trf(
                            "ctl.diag_objects",
                            &[
                                &diagnostics.objects.to_string(),
                                &diagnostics.plants.to_string(),
                            ],
                        ),
                    );
                    line(
                        ui,
                        trf("ctl.diag_cameras", &[&diagnostics.cameras.to_string()]),
                    );
                }
                None => line(ui, tr("ctl.diag_no_project").to_owned()),
            }
        });
}
