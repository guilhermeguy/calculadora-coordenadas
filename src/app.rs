use eframe::egui::{self, Align, Layout, RichText};

use crate::geo::{
    self,
    dms::{self, Axis},
    Converted, Coord, Datum,
};

pub const WIDTH: f32 = 460.0;
const FIELD_W: f32 = 170.0;
const SMALL_W: f32 = 52.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
    Dd,
    Dms,
    Utm,
}

impl Format {
    const ALL: [Format; 3] = [Format::Dd, Format::Dms, Format::Utm];

    fn label(self) -> &'static str {
        match self {
            Format::Dd => "Graus decimais (DD)",
            Format::Dms => "Graus, min, seg (GMS)",
            Format::Utm => "UTM",
        }
    }
}

/// Everything the user can edit. A result is shown only while the inputs
/// are identical to the ones it was computed from.
#[derive(Debug, Clone, PartialEq)]
struct Inputs {
    in_fmt: Format,
    out_fmt: Format,
    in_datum: Datum,
    out_datum: Datum,
    dd_lat: String,
    dd_lon: String,
    lat_d: String,
    lat_m: String,
    lat_s: String,
    lat_south: bool,
    lon_d: String,
    lon_m: String,
    lon_s: String,
    lon_west: bool,
    utm_zone: String,
    utm_south: bool,
    utm_e: String,
    utm_n: String,
    force_zone: bool,
    forced_zone: String,
}

impl Default for Inputs {
    fn default() -> Self {
        Self {
            in_fmt: Format::Dd,
            out_fmt: Format::Utm,
            in_datum: Datum::Sirgas2000,
            out_datum: Datum::Sirgas2000,
            dd_lat: String::new(),
            dd_lon: String::new(),
            lat_d: String::new(),
            lat_m: String::new(),
            lat_s: String::new(),
            lat_south: true,
            lon_d: String::new(),
            lon_m: String::new(),
            lon_s: String::new(),
            lon_west: true,
            utm_zone: String::new(),
            utm_south: true,
            utm_e: String::new(),
            utm_n: String::new(),
            force_zone: false,
            forced_zone: String::new(),
        }
    }
}

pub struct App {
    inp: Inputs,
    result: Option<(Inputs, Result<Converted, String>)>,
    last_height: f32,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self { inp: Inputs::default(), result: None, last_height: 0.0 }
    }

    fn run(&self) -> Result<Converted, String> {
        let i = &self.inp;
        let input = match i.in_fmt {
            Format::Dd => Coord::Geo {
                lat: dms::parse_angle(&i.dd_lat, Axis::Lat)?,
                lon: dms::parse_angle(&i.dd_lon, Axis::Lon)?,
            },
            Format::Dms => Coord::Geo {
                lat: dms::parse_dms_fields(&i.lat_d, &i.lat_m, &i.lat_s, i.lat_south, Axis::Lat)?,
                lon: dms::parse_dms_fields(&i.lon_d, &i.lon_m, &i.lon_s, i.lon_west, Axis::Lon)?,
            },
            Format::Utm => Coord::Utm {
                zone: parse_zone(&i.utm_zone)?,
                north: !i.utm_south,
                easting: dms::parse_number(&i.utm_e, "Easting (E)")?,
                northing: dms::parse_number(&i.utm_n, "Northing (N)")?,
            },
        };
        let forced = if i.out_fmt == Format::Utm && i.force_zone {
            Some(parse_zone(&i.forced_zone)?)
        } else {
            None
        };
        geo::convert(input, i.in_datum, i.out_datum, i.out_fmt == Format::Utm, forced)
    }

    fn selectors(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("selectors").num_columns(4).spacing([8.0, 6.0]).show(ui, |ui| {
            ui.label("Entrada:");
            format_combo(ui, "in_fmt", &mut self.inp.in_fmt);
            ui.label("Datum:");
            datum_combo(ui, "in_datum", &mut self.inp.in_datum);
            ui.end_row();

            ui.label("Saída:");
            format_combo(ui, "out_fmt", &mut self.inp.out_fmt);
            ui.label("Datum:");
            datum_combo(ui, "out_datum", &mut self.inp.out_datum);
            ui.end_row();
        });
    }

    fn input_fields(&mut self, ui: &mut egui::Ui) {
        let i = &mut self.inp;
        egui::Grid::new("inputs").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| match i.in_fmt {
            Format::Dd => {
                ui.label("Latitude:");
                text(ui, &mut i.dd_lat, FIELD_W, "-23,5505  ou  23,5505 S");
                ui.end_row();
                ui.label("Longitude:");
                text(ui, &mut i.dd_lon, FIELD_W, "-46,6333  ou  46,6333 W");
                ui.end_row();
            }
            Format::Dms => {
                ui.label("Latitude:");
                ui.horizontal(|ui| {
                    dms_row(ui, &mut i.lat_d, &mut i.lat_m, &mut i.lat_s);
                    hemi(ui, &mut i.lat_south, "N", "S");
                });
                ui.end_row();
                ui.label("Longitude:");
                ui.horizontal(|ui| {
                    dms_row(ui, &mut i.lon_d, &mut i.lon_m, &mut i.lon_s);
                    hemi(ui, &mut i.lon_west, "E", "W");
                });
                ui.end_row();
            }
            Format::Utm => {
                ui.label("Fuso:");
                ui.horizontal(|ui| {
                    text(ui, &mut i.utm_zone, SMALL_W, "23");
                    ui.add_space(8.0);
                    ui.label("Hemisfério:");
                    hemi(ui, &mut i.utm_south, "N", "S");
                });
                ui.end_row();
                ui.label("E (m):");
                text(ui, &mut i.utm_e, FIELD_W, "333288");
                ui.end_row();
                ui.label("N (m):");
                text(ui, &mut i.utm_n, FIELD_W, "7394588");
                ui.end_row();
            }
        });

        if i.out_fmt == Format::Utm {
            ui.horizontal(|ui| {
                ui.checkbox(&mut i.force_zone, "Forçar fuso de saída");
                if i.force_zone {
                    text(ui, &mut i.forced_zone, SMALL_W, "22");
                }
            });
        }
    }

    fn results(&self, ui: &mut egui::Ui) {
        let Some((inputs, res)) = &self.result else { return };
        ui.separator();
        let c = match res {
            Ok(c) => c,
            Err(e) => {
                ui.colored_label(ui.visuals().error_fg_color, format!("⚠ {e}"));
                return;
            }
        };

        let rows: Vec<(&str, String)> = match inputs.out_fmt {
            Format::Dd => vec![
                ("Latitude", dms::format_dd(c.lat)),
                ("Longitude", dms::format_dd(c.lon)),
            ],
            Format::Dms => vec![
                ("Latitude", dms::format_dms(c.lat, Axis::Lat)),
                ("Longitude", dms::format_dms(c.lon, Axis::Lon)),
            ],
            Format::Utm => {
                let u = c.utm.expect("UTM requested");
                let band = u.band.map(String::from).unwrap_or_default();
                let mc = geo::utm::central_meridian(u.zone);
                vec![
                    (
                        "Fuso",
                        format!(
                            "{}{band}  ({})  MC {mc:.0}°",
                            u.zone,
                            if u.north { "Norte" } else { "Sul" }
                        ),
                    ),
                    ("E", format!("{} m", dms::format_metres(u.easting))),
                    ("N", format!("{} m", dms::format_metres(u.northing))),
                    ("Convergência", format!("{:.6}°", u.convergence)),
                    ("Fator de escala", format!("{:.8}", u.scale)),
                ]
            }
        };

        ui.label(RichText::new(format!("Resultado ({})", inputs.out_datum.label())).strong());
        egui::Grid::new("results").num_columns(3).spacing([10.0, 4.0]).show(ui, |ui| {
            for (name, value) in &rows {
                ui.label(format!("{name}:"));
                ui.label(RichText::new(value).monospace().size(15.0));
                if ui.small_button("Copiar").clicked() {
                    ui.ctx().copy_text(copy_value(value));
                }
                ui.end_row();
            }
        });
        ui.horizontal(|ui| {
            if ui.button("Copiar tudo").clicked() {
                let all: Vec<String> = rows.iter().map(|(n, v)| format!("{n}: {v}")).collect();
                ui.ctx().copy_text(all.join("\n"));
            }
        });

        if inputs.in_datum != inputs.out_datum
            && (inputs.in_datum == Datum::Sad69 || inputs.out_datum == Datum::Sad69)
        {
            ui.label(
                RichText::new(
                    "Transformação SAD69 ↔ SIRGAS 2000 por 3 parâmetros (IBGE): precisão de alguns metros.",
                )
                .small()
                .weak(),
            );
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let enter = ctx.input(|i| i.key_pressed(egui::Key::Enter));

        // The panel itself always fills the window, so measure the content in
        // a nested layout; otherwise the auto-fit below feeds back on itself.
        let bottom = egui::CentralPanel::default()
            .show(ctx, |ui| ui.vertical(|ui| {
                self.selectors(ui);
                ui.separator();
                self.input_fields(ui);
                ui.add_space(4.0);

                let clicked = ui
                    .with_layout(Layout::top_down(Align::Center), |ui| {
                        ui.add_sized([160.0, 30.0], egui::Button::new(RichText::new("Converter").strong()))
                            .clicked()
                    })
                    .inner;
                if clicked || enter {
                    self.result = Some((self.inp.clone(), self.run()));
                }

                // Hide a stale result as soon as any input changes.
                if self.result.as_ref().is_some_and(|(snap, _)| *snap != self.inp) {
                    self.result = None;
                }
                self.results(ui);
            }).response.rect.bottom())
            .inner;

        // Grow / shrink the window to fit the content.
        let desired = (bottom + 10.0).ceil();
        if (desired - self.last_height).abs() > 1.0 {
            self.last_height = desired;
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(WIDTH, desired)));
        }
    }
}

fn parse_zone(s: &str) -> Result<u8, String> {
    s.trim()
        .parse::<u8>()
        .ok()
        .filter(|z| (1..=60).contains(z))
        .ok_or_else(|| format!("Fuso \"{}\" inválido (número de 1 a 60).", s.trim()))
}

/// Value to put on the clipboard: drop the unit suffix for metres.
fn copy_value(v: &str) -> String {
    v.trim_end_matches(" m").to_string()
}

fn text(ui: &mut egui::Ui, s: &mut String, width: f32, hint: &str) {
    ui.add(egui::TextEdit::singleline(s).desired_width(width).hint_text(hint));
}

fn dms_row(ui: &mut egui::Ui, d: &mut String, m: &mut String, s: &mut String) {
    ui.spacing_mut().item_spacing.x = 3.0;
    text(ui, d, 40.0, "G");
    ui.label("°");
    text(ui, m, 34.0, "M");
    ui.label("'");
    text(ui, s, 64.0, "S");
    ui.label("\"");
    ui.add_space(6.0);
}

/// Two-state hemisphere toggle; `negative` is true for the second option.
fn hemi(ui: &mut egui::Ui, negative: &mut bool, pos: &str, neg: &str) {
    ui.selectable_value(negative, false, pos);
    ui.selectable_value(negative, true, neg);
}

fn format_combo(ui: &mut egui::Ui, id: &str, value: &mut Format) {
    egui::ComboBox::from_id_salt(id)
        .width(150.0)
        .selected_text(value.label())
        .show_ui(ui, |ui| {
            for f in Format::ALL {
                ui.selectable_value(value, f, f.label());
            }
        });
}

fn datum_combo(ui: &mut egui::Ui, id: &str, value: &mut Datum) {
    egui::ComboBox::from_id_salt(id)
        .width(100.0)
        .selected_text(value.label())
        .show_ui(ui, |ui| {
            for d in Datum::ALL {
                ui.selectable_value(value, d, d.label());
            }
        });
}
