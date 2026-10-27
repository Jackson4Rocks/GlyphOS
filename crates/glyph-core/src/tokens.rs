//! GlyphOS design tokens and theme system.
//!
//! Two canonical palettes, named as products, not accidents:
//!   • **AMOLED Black** — absolute black (`#000000`) surfaces, true off-pixel
//!     economics on OLED, subtle light reflections, restrained illumination.
//!   • **Crystal White** — near-white crystalline surfaces, black ink,
//!     soft gray separators, industrial clean contrast.
//!
//! Single source of truth for colors, glass materials, typography, spacing,
//! radii and motion. The shell generates GTK CSS from these; first-party apps
//! may load the same stylesheet via `stylesheet()`.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------- palette
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreset {
    /// Absolute-black AMOLED palette (the default dark experience).
    #[default]
    AmoledBlack,
    /// Crystal-white crystalline palette (the default light experience).
    CrystalWhite,
}

/// One complete surface palette for a mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Palette {
    /// Wallpaper-behind root color.
    pub bg_root: Rgba,
    /// Primary window/panel surface.
    pub surface: Rgba,
    /// Elevated sibling surface.
    pub surface_elevated: Rgba,
    /// Secondary raised surface.
    pub surface_raised: Rgba,
    /// Primary ink.
    pub ink: Rgba,
    /// Secondary ink.
    pub ink_dim: Rgba,
    /// Thin separator tint (used at low alpha).
    pub separator: Rgba,
    /// Glass border tint.
    pub glass_border: Rgba,
    /// Glass tint color (the blur-kissed surface).
    pub glass_tint: Rgba,
}

/// AMOLED Black — absolute black, no ambient glow.
pub fn amoled_black() -> Palette {
    Palette {
        bg_root: Rgba::hex("#000000"),
        surface: Rgba::hex("#0A0A0A"),
        surface_elevated: Rgba::hex("#111111"),
        surface_raised: Rgba::hex("#161616"),
        ink: Rgba::hex("#FFFFFF"),
        ink_dim: Rgba::hex("#ADADAD"),
        separator: Rgba::hex("#FFFFFF").with_alpha(0.08),
        glass_border: Rgba::hex("#FFFFFF").with_alpha(0.10),
        glass_tint: Rgba::hex("#000000"),
    }
}

/// Crystal White — clean crystalline surfaces, black ink.
pub fn crystal_white() -> Palette {
    Palette {
        bg_root: Rgba::hex("#FAFAF8"),
        surface: Rgba::hex("#FFFFFF"),
        surface_elevated: Rgba::hex("#FFFFFF"),
        surface_raised: Rgba::hex("#F2F2EF"),
        ink: Rgba::hex("#101010"),
        ink_dim: Rgba::hex("#5C5C5C"),
        separator: Rgba::hex("#101010").with_alpha(0.08),
        glass_border: Rgba::hex("#101010").with_alpha(0.08),
        glass_tint: Rgba::hex("#FFFFFF"),
    }
}

// ---------------------------------------------------------------- tokens
/// Which mode (as in day/night; each resolves to a palette).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AppearanceMode {
    /// AMOLED Black on all surfaces.
    #[default]
    Dark,
    /// Crystal White on all surfaces.
    Light,
    /// Follow a schedule; resolves into one of the two presets.
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GlassLevel {
    /// Full frosted-glass pipeline (blur + tint).
    #[default]
    Frosted,
    /// Reduced-effects mode for weaker GPUs: no blur, strong tint.
    Reduced,
    /// Accessibility / cutout mode: near-opaque, no blur, highest contrast.
    Opaque,
}

/// The complete token set. Serialized in `glyphos.toml` under `[theme]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "kebab-case", default)]
pub struct Tokens {
    pub appearance: AppearanceMode,
    /// Explicit palette override; when unset the appearance decides
    /// (Dark → amoled_black, Light → crystal_white).
    pub preset: Option<ThemePreset>,
    pub accent: Rgba,
    pub glass: GlassLevel,
    /// Base surface opacity of glass shells (0..100).
    pub glass_opacity: u8,
    /// Background blur quality (0..100), compositor-dependent.
    pub blur_quality: u8,
    /// Global UI scale in percent (100 = 1x).
    pub scale_percent: u16,
    /// Overall animation intensity (0 = reduced motion).
    pub animation_intensity: u8,
    /// Panel/dock corner radius in px at 1x.
    pub corner_radius: u16,
    /// Preferred NDot-like font (user supplied, optional).
    pub display_font: Option<String>,
    /// Dot-matrix fallback shipped with the OS.
    pub fallback_display_font: String,
    /// Interface font.
    pub ui_font: String,
    /// Monospace font.
    pub mono_font: String,
}

impl Default for Tokens {
    fn default() -> Self {
        Self {
            appearance: AppearanceMode::Dark,
            preset: None,
            accent: Rgba::hex("#D71920"),
            glass: GlassLevel::Frosted,
            glass_opacity: 82,
            blur_quality: 70,
            scale_percent: 100,
            animation_intensity: 80,
            corner_radius: 14,
            display_font: None,
            fallback_display_font: "Doto".into(),
            ui_font: "Inter".into(),
            mono_font: "Space Mono".into(),
        }
    }
}

/// An 8-bit RGBA color that serializes as `#RRGGBB` (alpha optional).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub struct Rgba(pub [u8; 4]);

impl Rgba {
    pub fn hex(h: &str) -> Self {
        let h = h.trim_start_matches('#');
        let n = |s: &str| u8::from_str_radix(s, 16).unwrap_or(255);
        if h.len() == 6 {
            Self([n(&h[0..2]), n(&h[2..4]), n(&h[4..6]), 255])
        } else if h.len() == 8 {
            Self([n(&h[0..2]), n(&h[2..4]), n(&h[4..6]), n(&h[6..8])])
        } else {
            Self([255, 255, 255, 255])
        }
    }

    /// CSS rgba() string; may be used anywhere GTK CSS accepts a color.
    pub fn css(&self) -> String {
        let [r, g, b, a] = self.0;
        format!("rgba({r},{g},{b},{:.3})", a as f32 / 255.0)
    }

    /// Same color with a new alpha channel factor `t` (0..1). For a fully
    /// transparent tint pair such as glass surfaces this recolors alpha only.
    pub fn with_alpha(self, t: f32) -> Self {
        let [r, g, b, _] = self.0;
        Self([r, g, b, (255.0 * t).clamp(0.0, 255.0) as u8])
    }

    /// Compute a readable foreground color for this background.
    pub fn contrast_text(self) -> Self {
        let [r, g, b, _] = self.0;
        let lum = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
        if lum > 140.0 { Self::hex("#101010") } else { Self::hex("#FFFFFF") }
    }
}

impl From<Rgba> for String {
    fn from(c: Rgba) -> String {
        let [r, g, b, a] = c.0;
        if a == 255 {
            format!("#{r:02X}{g:02X}{b:02X}")
        } else {
            format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
        }
    }
}

impl TryFrom<String> for Rgba {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(Self::hex(&s))
    }
}

// ---------------------------------------------------------------- glass
/// Resolved glass material parameters for one shell surface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlassMaterial {
    /// Blur sigma in px (0 disables blur).
    pub blur_sigma: u8,
    /// Surface tint opacity 0..1.
    pub tint_opacity: f32,
    /// Border color used on opaque-free surfaces.
    pub border: Rgba,
    /// Tint color the glass fades toward.
    pub tint: Rgba,
    /// Dim applied to the desktop beneath the surface, 0..1.
    pub background_dim: f32,
}

impl Tokens {
    /// The display font actually used, resolving the fallback chain.
    pub fn display_font_resolved(&self) -> String {
        self.display_font.clone().unwrap_or_else(|| self.fallback_display_font.clone())
    }

    /// Palette actually in effect (appearance mode + optional preset pin).
    pub fn palette(&self) -> Palette {
        let dark = self.effective_appearance().0;
        match self.preset {
            Some(ThemePreset::AmoledBlack) => amoled_black(),
            Some(ThemePreset::CrystalWhite) => crystal_white(),
            None => {
                if dark { amoled_black() } else { crystal_white() }
            }
        }
    }

    /// (is_dark, resolved mode)
    pub fn effective_appearance(&self) -> (bool, AppearanceMode) {
        let mode = if self.appearance == AppearanceMode::Auto { auto_daytime() } else { self.appearance };
        (mode == AppearanceMode::Dark, mode)
    }

    /// Glass material for large shell surfaces (panel, dock, command bar).
    pub fn glass_shell(&self) -> GlassMaterial {
        let p = self.palette();
        let opacity = self.glass_opacity as f32 / 100.0;
        let base = GlassMaterial {
            blur_sigma: match self.glass { GlassLevel::Frosted => scale_blur(self.blur_quality), _ => 0 },
            tint_opacity: opacity,
            border: p.glass_border,
            tint: p.glass_tint,
            background_dim: 0.06,
        };
        match self.glass {
            GlassLevel::Frosted => base,
            GlassLevel::Reduced => GlassMaterial { tint_opacity: 0.94, ..base },
            GlassLevel::Opaque => GlassMaterial {
                // AMOLED cutout: fully solid, zero luminance leakage.
                tint_opacity: 1.0,
                background_dim: 0.0,
                ..base
            },
        }
    }

    /// Glass material for popovers and menus (a notch stronger).
    pub fn glass_popover(&self) -> GlassMaterial {
        let m = self.glass_shell();
        GlassMaterial {
            tint_opacity: (m.tint_opacity + 0.06).min(1.0),
            ..m
        }
    }
}

fn scale_blur(q: u8) -> u8 {
    let sigma = (q as f32 / 100.0) * 42.0;
    sigma.clamp(0.0, 42.0) as u8
}

/// Resolves Auto appearance by a simple daylight curve (7:00–20:00 → light).
fn auto_daytime() -> AppearanceMode {
    use chrono::{DateTime, Local, Timelike};
    let now: DateTime<Local> = Local::now();
    let h = now.hour();
    if (7..20).contains(&h) { AppearanceMode::Light } else { AppearanceMode::Dark }
}

// ---------------------------------------------------------------- css gen
/// Render the GlyphOS stylesheet. This is the canonical CSS for every
/// first-party surface. Mode-driven: AMOLED Black or Crystal White.
pub fn stylesheet(tokens: &Tokens) -> String {
    let p = tokens.palette();
    let (_, mode) = tokens.effective_appearance();
    let shell = tokens.glass_shell();
    let pop = tokens.glass_popover();

    let red = tokens.accent.css();
    let red_soft = tokens.accent.with_alpha(0.25).css();
    let ink = p.ink.css();
    let ink_dim = p.ink_dim.css();
    let border_ink_dim = ink_dim.clone();
    let card_tint = pop.tint.with_alpha((pop.tint_opacity * 0.55).clamp(0.05, 1.0)).css();
    let sep = p.separator.css();
    let surface = p.surface.css();
    let surface_raised = p.surface_raised.css();
    let bg_root = p.bg_root.css();
    let shell_border = shell.border.css();
    let pop_border = pop.border.css();
    let shell_tint = shell.tint.with_alpha(shell.tint_opacity).css();
    let pop_tint = pop.tint.with_alpha(pop.tint_opacity).css();

    let _ = shell.blur_sigma; // blur executes compositor-side (see ARCHITECTURE)
    let radius = tokens.corner_radius;
    let anim_len = (tokens.animation_intensity as f32 / 100.0 * 220.0).max(90.0) as u32;

    let display = tokens.display_font_resolved();
    let mono = &tokens.mono_font;
    let ui = &tokens.ui_font;
    // The header names the *palette actually in effect*, honoring preset pins.
    let mode_name = match tokens.preset {
        Some(ThemePreset::AmoledBlack) => "AMOLED BLACK",
        Some(ThemePreset::CrystalWhite) => "CRYSTAL WHITE",
        None => match mode {
            AppearanceMode::Dark => "AMOLED BLACK",
            AppearanceMode::Light => "CRYSTAL WHITE",
            AppearanceMode::Auto => "AUTO",
        },
    };

    format!(
        r#"/* == GlyphOS {mode_name} · generated · do not hand-edit == */
:root {{
    --bg-root: {bg_root};
    --surface: {surface};
    --surface-raised: {surface_raised};
    --ink: {ink};
    --ink-dim: {ink_dim};
    --sep: {sep};
    --red: {red};
    --red-soft: {red_soft};
    --shell-tint: {shell_tint};
    --shell-border: {shell_border};
    --pop-tint: {pop_tint};
    --pop-border: {pop_border};
    --radius: {radius}px;
    --anim: {anim_len}ms;
    --display-font: "{display}";
    --ui-font: "{ui}";
    --mono-font: "{mono}";
}}

* {{
    transition: background-color var(--anim) ease-out,
                box-shadow var(--anim) ease-out, opacity var(--anim) ease-out,
                transform var(--anim) ease-out;
}}

/* Entrance animations (run on map): flyout drops in from the bar. */
@keyframes drop-in {{
    from {{ opacity: 0; transform: translateY(-16px); }}
    to {{ opacity: 1; transform: translateY(0); }}
}}
@keyframes slide-up {{
    from {{ opacity: 0; transform: translateY(10px); }}
    to {{ opacity: 1; transform: translateY(0); }}
}}
.quick-shell {{ animation: drop-in var(--anim) cubic-bezier(0.2,0.9,0.3,1); }}
.cmdbar {{ animation: drop-in var(--anim) ease-out; }}

.glyph-root {{
    font-family: var(--ui-font);
    color: var(--ink);
    border-radius: var(--radius);
}}

/* ——— glass surfaces ——— */
.glass, .panel, .dock {{
    background-color: {shell_tint};
    border: 1px solid {shell_border};
}}
.glass-pop {{ background-color: {pop_tint}; border: 1px solid {pop_border}; }}
.cmdbar {{ box-shadow: 0 12px 32px rgba(0,0,0,0.42), 0 2px 8px rgba(0,0,0,0.26); }}

.window {{ background: {surface}; border-radius: calc(var(--radius) - 6px); }}

/* ——— typography roles ——— */
.h1 {{ font-size: 28px; font-weight: 600; letter-spacing: 0.5px; }}
.h2 {{ font-size: 20px; font-weight: 600; }}
.h3 {{ font-size: 16px; font-weight: 500; }}
.body {{ font-size: 14px; }}
.small {{ font-size: 12px; }}
.mono {{ font-family: var(--mono-font); font-size: 13px; }}
.dim {{ color: var(--ink-dim); }}
.ink {{ color: var(--ink); }}
.ndot {{ font-family: var(--display-font); font-weight: 600; letter-spacing: 2px; }}
.ndot-xs {{ font-size: 11px; }}
.ndot-sm {{ font-size: 13px; }}
.ndot-md {{ font-size: 18px; }}
.ndot-lg {{ font-size: 28px; }}
.ndot-xl {{ font-size: 44px; }}
.ndot-clock {{ font-size: 96px; }}

/* ——— app-grid mark (2×2 tiles, one accent tile) ——— */
.logo-tile {{ min-width: 9px; min-height: 9px; border-radius: 3px; margin: 1px; }}
.logo-tile.t1 {{ background-color: rgba(255,255,255,0.92); }}
.logo-tile.t2 {{ background-color: {red}; }}
.logo-tile.t3 {{ background-color: transparent; border: 1px solid {border_ink_dim}; }}
.logo-tile.t4 {{ background-color: {border_ink_dim}; }}

/* ——— quick flyout (clock) ——— */
.quick-shell {{ padding: 12px; border-radius: 16px; min-width: 640px; }}
.quick-card {{ padding: 14px 16px; border-radius: 12px; background-color: {card_tint}; min-width: 320px; }}
.quick-col {{ min-width: 360px; }}
.cal-day {{ font-family: var(--mono-font); font-size: 13px; padding: 3px 6px; border-radius: 8px; }}
.cal-day.today {{ background-color: {red_soft}; border: 1px solid {red}; color: {ink}; }}
.transport {{ min-width: 40px; min-height: 32px; }}

/* ——— desktop (bottom layer, owns the wallpaper) ——— */
.desktop {{ background-color: {bg_root}; }}
.desktop-mark {{ padding: 44px 72px; border-radius: 24px; }}

/* ——— panel / dock geometry ——— */
.panel {{ padding: 6px 12px; }}
.workspace-pill {{
    min-height: 22px; min-width: 22px; padding: 0 8px;
    border-radius: 11px; background: transparent; color: {ink_dim};
    font-family: var(--display-font);
}}
.workspace-pill.focused {{ background-color: {red_soft}; color: {ink}; box-shadow: 0 0 0 1px {red} inset; }}

/* ——— generic widgets ——— */
.pill {{
    border-radius: 14px;
    padding: 4px 10px;
    background-color: rgba(128,128,128,0.08);
}}
.pill:hover {{ background-color: rgba(128,128,128,0.16); }}
.pill:active {{ transform: scale(0.94); }}
.pill.focused {{ border-color: {red}; background-color: {red_soft}; }}

.list-row {{ padding: 10px 14px; border-radius: 10px; margin: 2px 0; }}
.list-row:hover {{ background-color: rgba(128,128,128,0.10); }}
.list-row.focused {{ background-color: rgba(128,128,128,0.16); }}
.notif-row {{ border-radius: 12px; }}
.notif-row:focus, .notif-row:hover {{ outline: none; box-shadow: none; }}

.cmdbar {{ padding: 12px; }}
.sensitive-entry {{ box-shadow: 0 2px 10px rgba(0,0,0,{entry_shadow}); font-size: 16px; }}

input:focus, entry:focus {{ outline: 1px solid {red}; }}
button:focus {{ outline: 1px solid {red}; outline-offset: -1px; }}

/* ——— subtle grid & dots signature (identity lines) ——— */
.dotgrid {{
    background-image: radial-gradient(circle, {sep} 1px, transparent 1px);
    background-size: 12px 12px;
    opacity: 0.35;
}}

scrollbar {{ min-width: 8px; min-height: 8px; background-color: transparent; }}
scrollbar slider {{ background-color: rgba(128,128,128,0.42); border-radius: 4px; }}
"#,
        bg_root = bg_root,
        surface = surface,
        surface_raised = surface_raised,
        ink = ink,
        ink_dim = ink_dim,
        sep = sep,
        red = red,
        red_soft = red_soft,
        shell_tint = shell_tint,
        shell_border = shell_border,
        pop_tint = pop_tint,
        pop_border = pop_border,
        radius = radius,
        anim_len = anim_len,
        display = display,
        ui = ui,
        mono = mono,
        mode_name = mode_name,
        entry_shadow = entry_shadow(tokens),
    )
}

fn entry_shadow(tokens: &Tokens) -> f32 {
    let (dark, _) = tokens.effective_appearance();
    if dark { 0.25 } else { 0.06 }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amoled_and_crystal_render_distinct_palettes() {
        let dark = Tokens { appearance: AppearanceMode::Dark, ..Default::default() };
        let light = Tokens { appearance: AppearanceMode::Light, ..Default::default() };
        let d = stylesheet(&dark);
        let l = stylesheet(&light);
        assert!(d.contains("AMOLED BLACK"), "dark header missing");
        assert!(l.contains("CRYSTAL WHITE"), "light header missing");
        assert_eq!(d, stylesheet(&dark));
        // AMOLED: absolute black root.
        assert!(d.contains("rgba(0,0,0,1.000)"), "background must be pure black: {}", {
            let i = d.find("--bg-root").unwrap();
            &d[i..i + 34]
        });
        // Crystal: near-white root, never pure #FFF glow.
        assert!(l.contains("rgba(250,250,248,1.000)"), "crystal root missing");
    }

    #[test]
    fn pinned_preset_overrides_appearance() {
        let t = Tokens {
            appearance: AppearanceMode::Dark,
            preset: Some(ThemePreset::CrystalWhite),
            ..Default::default()
        };
        assert!(stylesheet(&t).contains("CRYSTAL WHITE"));
    }
}
