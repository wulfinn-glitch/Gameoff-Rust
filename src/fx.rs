//! Central colour palette for the procedural effects, plus a live egui menu.
//!
//! Press `F2` to toggle the menu. Every colour here is the *base* colour; the
//! optional hue cycle applies a shared rotation on top without destroying the
//! values you picked.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

#[derive(Resource)]
pub struct FxPalette {
    /// Base colour behind everything (drives both the clear colour and the
    /// dark base of the background shader).
    pub background: Color,
    /// Low-energy grid lines in the background shader.
    pub grid_line: Color,
    /// High-energy shockwave/impact colour in the background shader.
    pub grid_impact: Color,
    pub attack: Color,
    pub parry: Color,
    pub spark: Color,
    pub deflect: Color,
    pub hit_flash: Color,
    pub reticle: Color,

    /// Slowly rotate every effect colour through the hue wheel.
    pub hue_cycle: bool,
    /// Full hue revolutions per second.
    pub hue_speed: f32,
    /// Current offset in `[0, 1)`.
    hue: f32,

    pub menu_open: bool,
}

impl Default for FxPalette {
    fn default() -> Self {
        Self {
            background: Color::srgb(0.045, 0.05, 0.06),
            grid_line: Color::hsl(40.0, 0.7, 0.45),
            grid_impact: Color::hsl(45.0, 0.9, 0.78),
            attack: Color::srgb(1.0, 0.83, 0.50),
            parry: Color::srgb(0.42, 1.0, 0.82),
            spark: Color::srgb(1.0, 0.94, 0.75),
            deflect: Color::srgb(0.55, 1.0, 0.95),
            hit_flash: Color::srgb(1.0, 1.0, 1.0),
            reticle: Color::srgb(0.42, 1.0, 0.92),
            hue_cycle: false,
            hue_speed: 0.05,
            hue: 0.0,
            menu_open: true,
        }
    }
}

impl FxPalette {
    fn shifted(&self, color: Color) -> Color {
        if !self.hue_cycle || self.hue.abs() < 1e-6 {
            return color;
        }

        let hsla = Hsla::from(color);
        let hue = (hsla.hue + self.hue * 360.0).rem_euclid(360.0);
        Color::Hsla(Hsla::new(hue, hsla.saturation, hsla.lightness, hsla.alpha))
    }

    pub fn background_color(&self) -> Color {
        self.shifted(self.background)
    }

    pub fn grid_line_color(&self) -> Color {
        self.shifted(self.grid_line)
    }

    pub fn grid_impact_color(&self) -> Color {
        self.shifted(self.grid_impact)
    }

    pub fn attack_color(&self) -> Color {
        self.shifted(self.attack)
    }

    pub fn parry_color(&self) -> Color {
        self.shifted(self.parry)
    }

    pub fn spark_color(&self) -> Color {
        self.shifted(self.spark)
    }

    pub fn deflect_color(&self) -> Color {
        self.shifted(self.deflect)
    }

    pub fn hit_flash_color(&self) -> Color {
        self.shifted(self.hit_flash)
    }

    pub fn reticle_color(&self) -> Color {
        self.shifted(self.reticle)
    }
}

/// Advance the shared hue offset while the cycle is enabled.
pub fn advance_hue(time: Res<Time>, mut palette: ResMut<FxPalette>) {
    if palette.hue_cycle {
        palette.hue = (palette.hue + time.delta_secs() * palette.hue_speed).rem_euclid(1.0);
    } else {
        palette.hue = 0.0;
    }
}

/// Push the (possibly hue-shifted) background colour into the renderer.
pub fn apply_background(palette: Res<FxPalette>, mut clear: ResMut<ClearColor>) {
    clear.0 = palette.background_color();
}

pub fn toggle_fx_menu(keys: Res<ButtonInput<KeyCode>>, mut palette: ResMut<FxPalette>) {
    if keys.just_pressed(KeyCode::F2) {
        palette.menu_open = !palette.menu_open;
    }
}

pub fn fx_debug_menu(mut contexts: EguiContexts, mut palette: ResMut<FxPalette>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    if !palette.menu_open {
        return;
    }

    egui::Window::new("FX Colours")
        .default_pos(egui::pos2(12.0, 320.0))
        .default_width(230.0)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Background shader");
            color_row(ui, "Base", &mut palette.background);
            color_row(ui, "Grid lines", &mut palette.grid_line);
            color_row(ui, "Impact wave", &mut palette.grid_impact);

            ui.separator();
            ui.label("Combat impacts");
            color_row(ui, "Attack slash", &mut palette.attack);
            color_row(ui, "Parry arc", &mut palette.parry);
            color_row(ui, "Hit spark", &mut palette.spark);
            color_row(ui, "Deflect", &mut palette.deflect);
            color_row(ui, "Enemy flash", &mut palette.hit_flash);
            color_row(ui, "Reticle", &mut palette.reticle);

            ui.separator();
            ui.checkbox(&mut palette.hue_cycle, "Slow hue shift");
            ui.add_enabled(
                palette.hue_cycle,
                egui::Slider::new(&mut palette.hue_speed, 0.0..=0.25).text("cycles/s"),
            );

            ui.separator();
            if ui.button("Reset defaults").clicked() {
                let open = palette.menu_open;
                *palette = FxPalette::default();
                palette.menu_open = open;
            }
        });
}

fn color_row(ui: &mut egui::Ui, label: &str, color: &mut Color) {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut c = to_color32(*color);
        if ui.color_edit_button_srgba(&mut c).changed() {
            *color = from_color32(c);
        }
    });
}

fn to_color32(color: Color) -> egui::Color32 {
    let c = color.to_srgba();
    let to_u8 = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    egui::Color32::from_rgba_unmultiplied(
        to_u8(c.red),
        to_u8(c.green),
        to_u8(c.blue),
        to_u8(c.alpha),
    )
}

fn from_color32(color: egui::Color32) -> Color {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    Color::srgba(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hue_cycle_is_identity_when_off_and_shifts_when_on() {
        let mut palette = FxPalette::default();

        // Off: base colour passes through untouched.
        assert_eq!(palette.attack_color(), palette.attack);

        // On: a quarter turn produces a different colour.
        palette.hue_cycle = true;
        palette.hue = 0.25;
        assert_ne!(palette.attack_color(), palette.attack);

        // A full turn returns to the base colour.
        palette.hue = 1.0;
        let a = palette.attack_color().to_srgba();
        let b = palette.attack.to_srgba();
        assert!((a.red - b.red).abs() < 1e-3);
        assert!((a.green - b.green).abs() < 1e-3);
        assert!((a.blue - b.blue).abs() < 1e-3);
    }
}
