//! The search, sort and taxa filter card above the catalog.

use crate::components::{
    chip::Chip, icons, layout::split_row, search_field::SearchField, segmented::Segmented,
    tag::Tag, text,
};
use crate::config::theme::{colors, typography};
use crate::features::library::logic;
use crate::features::library::models::{LibraryState, Taxa};
use crate::services::localization::{tr, trf};
use crate::services::sample_data::library as sample;
use eframe::egui::{self, CornerRadius, Margin, Ui};

const TAXA_LABELS: [&str; 6] = [
    "library.taxa_broadleaf",
    "library.taxa_conifers",
    "library.taxa_palms",
    "library.taxa_grasses",
    "library.taxa_flowers",
    "library.taxa_north_america",
];
const SORT_LABELS: [&str; 4] = [
    "library.sort_volume",
    "library.sort_name",
    "library.sort_rating",
    "library.sort_height",
];

/// Draws the card; `shown` is the number of cards currently listed.
pub fn show(ui: &mut Ui, state: &mut LibraryState, shown: usize) {
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .corner_radius(CornerRadius::same(2))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            let sort_width = 330.0;
            split_row(
                ui,
                32.0,
                |ui| {
                    let badges = [
                        (
                            tr(TAXA_LABELS[state.taxa as usize]),
                            colors::SURFACE_CONTAINER,
                            colors::PRIMARY,
                        ),
                        (
                            sample::ACTIVE_FILTERS,
                            colors::SECONDARY_CONTAINER,
                            colors::ON_SECONDARY_CONTAINER,
                        ),
                    ];
                    SearchField::new(tr("library.search_hint"))
                        .width(ui.available_width() - sort_width)
                        .height(32.0)
                        .font(typography::body_md(), 18.0)
                        .fill(colors::SURFACE_LOWEST)
                        .badges(&badges)
                        .show(ui, &mut state.search);
                },
                |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    let mut layout = state.layout;
                    Segmented::icons(&[icons::GRID_VIEW, icons::VIEW_LIST])
                        .container(colors::SURFACE_LOWEST, 2.0, 0.0)
                        .selected(colors::SURFACE_HIGH, colors::PRIMARY)
                        .height(28.0)
                        .show(ui, &mut layout);
                    state.layout = layout;
                    ui.add_space(4.0);
                    if Tag::new(tr(SORT_LABELS[state.sort as usize]), colors::ON_SURFACE)
                        .trail_icon(icons::UNFOLD_MORE, 14.0, colors::OUTLINE)
                        .gap(4.0)
                        .fill(colors::SURFACE_CONTAINER)
                        .padding(8.0, 4.0)
                        .clickable()
                        .show(ui)
                        .clicked()
                    {
                        state.sort = state.sort.next();
                    }
                    text::label(
                        ui,
                        tr("library.sort"),
                        typography::label_sm(),
                        colors::OUTLINE,
                    );
                },
            );
            ui.add_space(4.0);
            split_row(
                ui,
                28.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    text::label(
                        ui,
                        text::caps(tr("library.taxa")),
                        typography::label_sm(),
                        colors::OUTLINE_VARIANT,
                    );
                    ui.add_space(4.0);
                    for (index, taxa) in Taxa::ALL.into_iter().enumerate() {
                        let selected = state.taxa == taxa;
                        let mut chip = Chip::new(tr(TAXA_LABELS[index]))
                            .count(sample::TAXA_COUNTS[index])
                            .selected(selected);
                        if taxa == Taxa::NorthAmerica {
                            chip = chip.icon(icons::PUBLIC, colors::OUTLINE);
                        } else if selected {
                            chip = chip.icon(icons::CHECK, colors::ON_PRIMARY);
                        }
                        if chip.show(ui).clicked() {
                            logic::toggle_taxa(state, taxa);
                        }
                    }
                },
                |ui| {
                    let total = sample::TOTAL_ASSETS.to_string();
                    text::label(
                        ui,
                        trf("library.showing", &[&shown.to_string(), &total]),
                        typography::label_sm(),
                        colors::OUTLINE,
                    );
                },
            );
        });
}
