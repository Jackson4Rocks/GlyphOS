//! GlyphOS design tokens and theme system.
//!
//! Single source of truth for colors, glass materials, typography, spacing,
//! radii and motion. The shell generates GTK CSS from these; first-party apps
//! may load the same tokens via `load_css()`.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------- palette
/// Raw palette primitives. Dark and light themes both derive from this.
pub mod palette {
    /// Absolute black (nyx).
    pub const BLACK: &str = "#000000";
    /// Deep background (nyx-1).
    pub const DEEP: &str = "#080808";
    /// Elevated dark surface (nyx-2).
    pub const ELEVATED: &str = "#141414";
    /// Secondary dark surface (nyx-3).
    pub const RAISED: &str = "#1C1C1C";
    /// Light background (lumen-1, warm neutral).
    pub const LIGHT_BG: &str = "#F2F2F0";
    /// White surface (lumen-2).
    pub const WHITE: &str = "#FFFFFF";
    /// Primary text on dark.
    pub const TEXT_DARK: &str = "#FFFFFF";
    /// Secondary text on dark.
    pub const TEXT_SECONDARY_DARK: &str = "#ADADAD";
    /// Primary text on light.
    pub const TEXT_LIGHT: &str = "#101010";
    /// Secondary text on light.
    pub const TEXT_SECONDARY_LIGHT: &str = "#626262";
    /// Signature red accent (pulse).
    pub const ACCENT_RED: &str = "#D71920";
}

// ---------------------------------------------------------------- tokens
/// Which appearance is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AppearanceMode {
    #[default]
    Dark,
    Light,
    /// Follow a schedule / sensor; resolved into Dark or Light at runtime.
    Auto,
}

/// Glass material strength presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GlassLevel {
    /// Full frosted-glass pipeline (blur + tint).
    #[default]
    Frosted,
    /// Reduced-effects mode for weaker GPUs.
    Reduced,
    /// Accessibility / cutout mode: near-opaque surfaces, no blur.
    Opaque,
}

/// The complete token set. Serialized in `glyphos.toml` under `[theme]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "kebab-case", default)]
pub struct Tokens {
    pub appearance: AppearanceMode,
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
            accent: Rgba::hex(palette::ACCENT_RED),
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

    /// Same color, mixed toward alpha `0` by factor `t`.
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
        if a == 255 { format!("#{r:02X}{g:02X}{b:02X}") } else { format!("#{r:02X}{g:02X}{b:02X}{a:02X}") }
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
    /// Dim applied to the desktop beneath the surface, 0..1.
    pub background_dim: f32,
}

impl Tokens {
    /// The display font actually used, resolving the fallback chain.
    pub fn display_font_resolved(&self) -> String {
        self.display_font.clone().unwrap_or_else(|| self.fallback_display_font.clone())
    }

    /// Glass material for large shell surfaces (panel, dock, command bar).
    pub fn glass_shell(&self) -> GlassMaterial {
        let opacity = self.glass_opacity as f32 / 100.0;
        match self.glass {
            GlassLevel::Frosted => GlassMaterial {
                blur_sigma: scale_blur(self.blur_quality),
                tint_opacity: opacity * 0.80,
                border: border_for(self),
                background_dim: 0.06,
            },
            GlassLevel::Reduced => GlassMaterial {
                blur_sigma: 0,
                tint_opacity: 0.94,
                border: border_for(self),
                background_dim: 0.05,
            },
            GlassLevel::Opaque => GlassMaterial {
                blur_sigma: 0,
                tint_opacity: 1.0,
                border: border_for(self),
                background_dim: 0.0,
            },
        }
    }

    /// Glass material for popovers and menus (slightly stronger dimming).
    pub fn glass_popover(&self) -> GlassMaterial {
        let m = self.glass_shell();
        GlassMaterial {
            tint_opacity: (m.tint_opacity + 0.06).min(1.0),
            ..m
        }
    }

    /// Dark or light ink, resolving the Auto mode into a concrete appearance.
    pub fn effective_appearance(&self) -> (bool, AppearanceMode) {
        let mode = if self.appearance == AppearanceMode::Auto { auto_daytime() } else { self.appearance };
        (mode == AppearanceMode::Dark, mode)
    }
}

fn scale_blur(q: u8) -> u8 {
    let sigma = (q as f32 / 100.0) * 42.0;
    sigma.clamp(0.0, 42.0) as u8
}

fn border_for(tokens: &Tokens) -> Rgba {
    let is_dark = tokens.effective_appearance().0;
    if is_dark { Rgba::hex("#FFFFFF").with_alpha(0.10) } else { Rgba::hex("#000000").with_alpha(0.08) }
}

/// Resolves Auto appearance for the cases we can detect: simple daylight curve.
fn auto_daytime() -> AppearanceMode {
    use chrono::{DateTime, Local, Timelike};
    let now: DateTime<Local> = Local::now();
    let h = now.hour();
    if (7..20).contains(&h) { AppearanceMode::Light } else { AppearanceMode::Dark }
}

// ---------------------------------------------------------------- css gen
/// Render the GlyphOS stylesheet. This is the canonical CSS for every
/// first-party surface. Returns a fresh string; apps call `load_css()`.
pub fn stylesheet(tokens: &Tokens) -> String {
    let (is_dark, _) = tokens.effective_appearance();
    let shell = tokens.glass_shell();
    let pop = tokens.glass_popover();

    // Palette derived by mode.
    let (bg_root, ink, ink_dim, sep) = if is_dark {
        (palette::DEEP, palette::TEXT_DARK, palette::TEXT_SECONDARY_DARK, "#FFFFFF")
    } else {
        (palette::LIGHT_BG, palette::TEXT_LIGHT, palette::TEXT_SECONDARY_LIGHT, "#000000")
    };
    let surface = if is_dark { palette::ELEVATED } else { palette::WHITE };

    let red = tokens.accent.css();
    let red_soft = tokens.accent.with_alpha(0.25).css();

    let shell_alpha = shell.tint_opacity;
    let shell_alpha2 = (shell.tint_opacity * 0.92).clamp(0.0, 1.0);
    let shell_border_border = shell.border.css();
    let pop_border = pop.tint_opacity;
    let pop_border_b = pop.border.css();

    let blur_ = if shell.blur_sigma == 0 { "none" } else { "grayscale(1) blur(var(--blur-sigma))" };
    let radius = tokens.corner_radius;
    let anim_len = (tokens.animation_intensity as f32 / 100.0 * 220.0).max(90.0) as u32;
    let anim_ms = format!("{anim_len}ms");

    let display = tokens.display_font_resolved();
    let mono = &tokens.mono_font;
    let ui = &tokens.ui_font;

    format!(
        r#"
/* ==== GlyphOS stylesheet · generated from design tokens — do not hand-edit ==== */
:root {{
    --bg-root: {bg_root};
    --surface: {surface};
    --ink: {ink};
    --ink-dim: {ink_dim};
    --sep: {sep};
    --red: {red};
    --red-soft: {red_soft};
    --shell-alpha: {shell_alpha:.3};
    --shell-alpha-2: {shell_alpha2:.3};
    --shell-border: {shell_border_border};
    --pop-alpha: {pop_border:.3};
    --pop-border: {pop_border_b};
    --blur-sigma: {sig}px;
    --radius: {radius}px;
    --anim: {anim_ms};
    --display-font: "{display}";
    --ui-font: "{ui}";
    --mono-font: "{mono}";
    --shadow-lg: 0 12px 32px rgba(0,0,0,0.42), 0 2px 8px rgba(0,0,0,0.26);
}}

* {{
    transition: background-color var(--anim) ease-out,
                box-shadow var(--anim) ease-out, opacity var(--anim) ease-out;
    outline: 0 solid transparent;
}}

.glyph-root {{
    font-family: var(--ui-font);
    color: var(--ink);
    border-radius: var(--radius);
}}

/* ——— glass surfaces ——— */
.glass {{
    background: rgba(0,0,0,var(--shell-alpha));
    border-color: transparent;
}}
.glass-pop {{ background: rgba(0,0,0,var(--pop-alpha)); }}

.glyph-glass {{
    background-color: var(--shell-alpha-color, rgba(10,10,10,0.86));
    backdrop-filter: {blur_};
    border-radius: var(--radius);
    box-shadow: var(--shadow-lg);
}}

.window {{ background: var(--surface); border-radius: calc(var(--radius) - 6px); }}

/* ——— typography roles ——— */
.h1 {{ font-size: 28px; font-weight: 600; letter-spacing: 0.5px; }}
.h2 {{ font-size: 20px; font-weight: 600; }}
.h3 {{ font-size: 16px; font-weight: 500; }}
.body {{ font-size: 14px; }}
.small {{ font-size: 12px; }}
.mono {{ font-family: var(--mono-font); font-size: 13px; }}
.dim {{ color: var(--ink-dim); }}
.ndot {{ font-family: var(--display-font); font-weight: 600; letter-spacing: 2px; }}

/* ——— header / time ——— */
.clock {{ font-family: var(--display-font); font-weight: 600; font-size: 96px; color: var(--ink); letter-spacing: 4px; }}
.window-label {{ font-family: var(--ui-font); font-size: 12px; }}
.workspace-pill {{ border-radius: 12px; padding: 0 10px; font-family: var(--display-font); font-size: 12px; }}

/* ——— generic widgets ——— */
.pill {{
    border-radius: 14px;
    padding: 4px 10px;
    background-color: transparent;
    border: 1px solid var(--shell-border);
}}
.pill:hover {{ border-color: rgba(255,255,255,0.22); }}
.pill.accent {{ background: var(--red-soft); border-color: transparent; }}
.pill.focused {{ border-color: var(--red); }}

.hoverable:hover {{ background-color: rgba(255,255,255,0.05); }}
.hoverable:active {{ background-color: rgba(255,255,255,0.09); }}
.dock-item {{ border-radius: 10px; padding: 4px 8px; }}
.dock-item:focus {{ border-color: var(--red); }}

.list-row {{
    padding: 10px 14px;
    border-radius: 10px;
    margin: 2px 0;
}}
.list-row:hover {{ background-color: rgba(255,255,255,0.06); }}
.list-row.focused {{ background-color: rgba(255,255,255,0.12); }}

.cmdbar {{
    background: transparent;
    border-radius: 18px;
    padding: 12px;
    box-shadow: var(--shadow-lg);
}}

.sensitive-entry {{ box-shadow: 0 2px 10px rgba(0,0,0,0.25); font-size: 16px; }}

input:focus, entry:focus, button:focus {{ outline: 2px solid var(--red); }}

.vm-indicator {{
    min-width: 12px; border-radius: 3px; background-color: rgba(255,255,255,0.14);
    transition: all var(--anim) ease-out;
}}
.vm-indicator.focused {{ background-color: var(--red); }}

.checkpoint {{
    min-height: 34px; min-width: 34px; border-radius: 17px;
    border: 1px solid var(--shell-border);
}}

/* scrollbars */
scrollbar {{ min-width: 8px; min-height: 8px; background-color: transparent; }}
scrollbar slider {{ background-color: rgba(128,128,128,0.42); border-radius: 4px; }}
"#,
        bg_root = bg_root,
        surface = surface,
        ink = ink,
        ink_dim = ink_dim,
        sep = sep,
        red = red,
        red_soft = red_soft,
        shell_alpha = shell_alpha,
        shell_alpha2 = shell_alpha2,
        shell_border_border = shell_border_border,
        pop_border = pop_border,
        pop_border_b = pop_border_b,
        sig = 24,
        radius = radius,
        anim_ms = anim_ms,
        display = display,
        ui = ui,
        mono = mono,
        blur_ = blur_,
    )
}
