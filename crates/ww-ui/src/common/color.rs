use gpui::Hsla;
use gpui::hsla;

#[allow(unused)]
/// Pure red: `hsl(0°, 100%, 50%)`
pub fn red() -> Hsla {
    hsla(0.0, 1.0, 0.5, 1.0)
}

#[allow(unused)]
/// Pure orange: `hsl(30°, 100%, 50%)`
pub fn orange() -> Hsla {
    hsla(0.0833, 1.0, 0.5, 1.0)
}

#[allow(unused)]
/// Pure yellow: `hsl(60°, 100%, 50%)`
pub fn yellow() -> Hsla {
    hsla(0.1667, 1.0, 0.5, 1.0)
}
#[allow(unused)]
/// Lime green: `hsl(75°, 100%, 50%)`
pub fn lime() -> Hsla {
    hsla(0.2083, 1.0, 0.5, 1.0)
}
#[allow(unused)]
/// Pure green: `hsl(120°, 100%, 50%)`
pub fn green() -> Hsla {
    hsla(0.3333, 1.0, 0.5, 1.0)
}
#[allow(unused)]
/// Teal: `hsl(160°, 100%, 50%)`
pub fn teal() -> Hsla {
    hsla(0.4444, 1.0, 0.5, 1.0)
}
#[allow(unused)]
/// Pure cyan: `hsl(180°, 100%, 50%)`
pub fn cyan() -> Hsla {
    hsla(0.5, 1.0, 0.5, 1.0)
}
#[allow(unused)]
/// Light blue: `hsl(200°, 100%, 50%)`
pub fn sky_blue() -> Hsla {
    hsla(0.5556, 1.0, 0.5, 1.0)
}
#[allow(unused)]
/// Pure blue: `hsl(240°, 100%, 50%)`
pub fn blue() -> Hsla {
    hsla(0.6667, 1.0, 0.5, 1.0)
}
#[allow(unused)]
/// Indigo: `hsl(260°, 100%, 50%)`
pub fn indigo() -> Hsla {
    hsla(0.7222, 1.0, 0.5, 1.0)
}
#[allow(unused)]
/// Purple: `hsl(280°, 100%, 50%)`
pub fn purple() -> Hsla {
    hsla(0.7778, 1.0, 0.5, 1.0)
}
#[allow(unused)]
/// Magenta / Fuchsia: `hsl(300°, 100%, 50%)`
pub fn magenta() -> Hsla {
    hsla(0.8333, 1.0, 0.5, 1.0)
}
#[allow(unused)]
/// Pink: `hsl(330°, 100%, 50%)`
pub fn pink() -> Hsla {
    hsla(0.9167, 1.0, 0.5, 1.0)
}
#[allow(unused)]
/// Rose: `hsl(350°, 100%, 50%)`
pub fn rose() -> Hsla {
    hsla(0.9722, 1.0, 0.5, 1.0)
}

// ── Neutral color functions ──────────────────────────────────────────
#[allow(unused)]
/// Pure white
pub fn white() -> Hsla {
    hsla(0.0, 0.0, 1.0, 1.0)
}
#[allow(unused)]
/// Pure black
pub fn black() -> Hsla {
    hsla(0.0, 0.0, 0.0, 1.0)
}

#[allow(unused)]
/// Transparent
pub fn transparent() -> Hsla {
    hsla(0.0, 0.0, 0.0, 0.0)
}

/// A grey with the given lightness (0.0 = black, 1.0 = white).
///
/// # Examples
/// ```
/// let light_grey = grey(0.8);  // near white
/// let mid_grey   = grey(0.5);  // middle grey
/// let dark_grey  = grey(0.2);  // near black
/// ```
///
#[allow(unused)]
pub fn grey(lightness: f32) -> Hsla {
    grey_with_alpha(lightness, 1.0)
}

#[allow(unused)]
pub fn grey_with_alpha(lightness: f32, a: f32) -> Hsla {
    hsla(0.0, 0.0, lightness.clamp(0.0, 1.0), a)
}

// ── Degree-adjustable colour functions ───────────────────────────────
//
// Each function below accepts a `lightness` parameter (0.0 – 1.0) so
// callers can dial the colour up or down.  The *base* variant (e.g.
// `red()`) is equivalent to calling the adjustable version with 0.5.
#[allow(unused)]
/// Red with an adjustable lightness.
///
/// `lightness` 0.0 = black, 0.5 = pure red, 1.0 = white.
pub fn red_with(lightness: f32) -> Hsla {
    hsla(0.0, 1.0, lightness.clamp(0.0, 1.0), 1.0)
}

#[allow(unused)]
/// Orange with an adjustable lightness.
pub fn orange_with(lightness: f32) -> Hsla {
    hsla(0.0833, 1.0, lightness.clamp(0.0, 1.0), 1.0)
}

#[allow(unused)]
/// Yellow with an adjustable lightness.
pub fn yellow_with(lightness: f32) -> Hsla {
    hsla(0.1667, 1.0, lightness.clamp(0.0, 1.0), 1.0)
}

#[allow(unused)]
/// Lime with an adjustable lightness.
pub fn lime_with(lightness: f32) -> Hsla {
    hsla(0.2083, 1.0, lightness.clamp(0.0, 1.0), 1.0)
}

#[allow(unused)]
/// Green with an adjustable lightness.
pub fn green_with(lightness: f32) -> Hsla {
    hsla(0.3333, 1.0, lightness.clamp(0.0, 1.0), 1.0)
}

#[allow(unused)]
/// Teal with an adjustable lightness.
pub fn teal_with(lightness: f32) -> Hsla {
    hsla(0.4444, 1.0, lightness.clamp(0.0, 1.0), 1.0)
}

#[allow(unused)]
/// Cyan with an adjustable lightness.
pub fn cyan_with(lightness: f32) -> Hsla {
    hsla(0.5, 1.0, lightness.clamp(0.0, 1.0), 1.0)
}

#[allow(unused)]
/// Sky-blue with an adjustable lightness.
pub fn sky_blue_with(lightness: f32) -> Hsla {
    hsla(0.5556, 1.0, lightness.clamp(0.0, 1.0), 1.0)
}

#[allow(unused)]
/// Blue with an adjustable lightness.
pub fn blue_with(lightness: f32) -> Hsla {
    hsla(0.6667, 1.0, lightness.clamp(0.0, 1.0), 1.0)
}

#[allow(unused)]
/// Indigo with an adjustable lightness.
pub fn indigo_with(lightness: f32) -> Hsla {
    hsla(0.7222, 1.0, lightness.clamp(0.0, 1.0), 1.0)
}

#[allow(unused)]
/// Purple with an adjustable lightness.
pub fn purple_with(lightness: f32) -> Hsla {
    hsla(0.7778, 1.0, lightness.clamp(0.0, 1.0), 1.0)
}

#[allow(unused)]
/// Magenta with an adjustable lightness.
pub fn magenta_with(lightness: f32) -> Hsla {
    hsla(0.8333, 1.0, lightness.clamp(0.0, 1.0), 1.0)
}

#[allow(unused)]
/// Pink with an adjustable lightness.
pub fn pink_with(lightness: f32) -> Hsla {
    hsla(0.9167, 1.0, lightness.clamp(0.0, 1.0), 1.0)
}

#[allow(unused)]
/// Rose with an adjustable lightness.
pub fn rose_with(lightness: f32) -> Hsla {
    hsla(0.9722, 1.0, lightness.clamp(0.0, 1.0), 1.0)
}

#[allow(unused)]
/// Grey with an adjustable lightness (alias for [`grey`]).
pub fn grey_with(lightness: f32) -> Hsla {
    grey(lightness)
}

// ── Convenience: pre-baked shades for the most-used colours ──────────
//
// Naming follows a 50–900 scale roughly inspired by Tailwind.
//   50  = very light (near white)
//   100 = extra light
//   200 = light
//   300 = medium-light
//   400 = normal-light
//   500 = base   (same as calling the bare function, e.g. `blue()`)
//   600 = normal-dark
//   700 = dark
//   800 = extra dark
//   900 = very dark (near black)

pub mod shades {
    #![allow(unused)]

    use gpui::Hsla;
    use gpui::hsla;

    // ── Red ──────────────────────────────────────────────────────

    pub fn red_50() -> Hsla {
        hsla(0.0, 1.0, 0.95, 1.0)
    }
    pub fn red_100() -> Hsla {
        hsla(0.0, 1.0, 0.85, 1.0)
    }
    pub fn red_200() -> Hsla {
        hsla(0.0, 1.0, 0.75, 1.0)
    }
    pub fn red_300() -> Hsla {
        hsla(0.0, 1.0, 0.65, 1.0)
    }
    pub fn red_400() -> Hsla {
        hsla(0.0, 1.0, 0.55, 1.0)
    }
    pub fn red_500() -> Hsla {
        hsla(0.0, 1.0, 0.50, 1.0)
    }
    pub fn red_600() -> Hsla {
        hsla(0.0, 1.0, 0.42, 1.0)
    }
    pub fn red_700() -> Hsla {
        hsla(0.0, 1.0, 0.33, 1.0)
    }
    pub fn red_800() -> Hsla {
        hsla(0.0, 1.0, 0.22, 1.0)
    }
    pub fn red_900() -> Hsla {
        hsla(0.0, 1.0, 0.12, 1.0)
    }

    // ── Blue ─────────────────────────────────────────────────────

    pub fn blue_50() -> Hsla {
        hsla(0.6667, 1.0, 0.95, 1.0)
    }
    pub fn blue_100() -> Hsla {
        hsla(0.6667, 1.0, 0.85, 1.0)
    }
    pub fn blue_200() -> Hsla {
        hsla(0.6667, 1.0, 0.75, 1.0)
    }
    pub fn blue_300() -> Hsla {
        hsla(0.6667, 1.0, 0.65, 1.0)
    }
    pub fn blue_400() -> Hsla {
        hsla(0.6667, 1.0, 0.55, 1.0)
    }
    pub fn blue_500() -> Hsla {
        hsla(0.6667, 1.0, 0.50, 1.0)
    }
    pub fn blue_600() -> Hsla {
        hsla(0.6667, 1.0, 0.42, 1.0)
    }
    pub fn blue_700() -> Hsla {
        hsla(0.6667, 1.0, 0.33, 1.0)
    }
    pub fn blue_800() -> Hsla {
        hsla(0.6667, 1.0, 0.22, 1.0)
    }
    pub fn blue_900() -> Hsla {
        hsla(0.6667, 1.0, 0.12, 1.0)
    }

    // ── Green ────────────────────────────────────────────────────

    pub fn green_50() -> Hsla {
        hsla(0.3333, 1.0, 0.95, 1.0)
    }
    pub fn green_100() -> Hsla {
        hsla(0.3333, 1.0, 0.85, 1.0)
    }
    pub fn green_200() -> Hsla {
        hsla(0.3333, 1.0, 0.75, 1.0)
    }
    pub fn green_300() -> Hsla {
        hsla(0.3333, 1.0, 0.65, 1.0)
    }
    pub fn green_400() -> Hsla {
        hsla(0.3333, 1.0, 0.55, 1.0)
    }
    pub fn green_500() -> Hsla {
        hsla(0.3333, 1.0, 0.50, 1.0)
    }
    pub fn green_600() -> Hsla {
        hsla(0.3333, 1.0, 0.42, 1.0)
    }
    pub fn green_700() -> Hsla {
        hsla(0.3333, 1.0, 0.33, 1.0)
    }
    pub fn green_800() -> Hsla {
        hsla(0.3333, 1.0, 0.22, 1.0)
    }
    pub fn green_900() -> Hsla {
        hsla(0.3333, 1.0, 0.12, 1.0)
    }

    // ── Yellow ───────────────────────────────────────────────────

    pub fn yellow_50() -> Hsla {
        hsla(0.1667, 1.0, 0.95, 1.0)
    }
    pub fn yellow_100() -> Hsla {
        hsla(0.1667, 1.0, 0.85, 1.0)
    }
    pub fn yellow_200() -> Hsla {
        hsla(0.1667, 1.0, 0.75, 1.0)
    }
    pub fn yellow_300() -> Hsla {
        hsla(0.1667, 1.0, 0.65, 1.0)
    }
    pub fn yellow_400() -> Hsla {
        hsla(0.1667, 1.0, 0.55, 1.0)
    }
    pub fn yellow_500() -> Hsla {
        hsla(0.1667, 1.0, 0.50, 1.0)
    }
    pub fn yellow_600() -> Hsla {
        hsla(0.1667, 1.0, 0.42, 1.0)
    }
    pub fn yellow_700() -> Hsla {
        hsla(0.1667, 1.0, 0.33, 1.0)
    }
    pub fn yellow_800() -> Hsla {
        hsla(0.1667, 1.0, 0.22, 1.0)
    }
    pub fn yellow_900() -> Hsla {
        hsla(0.1667, 1.0, 0.12, 1.0)
    }

    // ── Purple ───────────────────────────────────────────────────

    pub fn purple_50() -> Hsla {
        hsla(0.7778, 1.0, 0.95, 1.0)
    }
    pub fn purple_100() -> Hsla {
        hsla(0.7778, 1.0, 0.85, 1.0)
    }
    pub fn purple_200() -> Hsla {
        hsla(0.7778, 1.0, 0.75, 1.0)
    }
    pub fn purple_300() -> Hsla {
        hsla(0.7778, 1.0, 0.65, 1.0)
    }
    pub fn purple_400() -> Hsla {
        hsla(0.7778, 1.0, 0.55, 1.0)
    }
    pub fn purple_500() -> Hsla {
        hsla(0.7778, 1.0, 0.50, 1.0)
    }
    pub fn purple_600() -> Hsla {
        hsla(0.7778, 1.0, 0.42, 1.0)
    }
    pub fn purple_700() -> Hsla {
        hsla(0.7778, 1.0, 0.33, 1.0)
    }
    pub fn purple_800() -> Hsla {
        hsla(0.7778, 1.0, 0.22, 1.0)
    }
    pub fn purple_900() -> Hsla {
        hsla(0.7778, 1.0, 0.12, 1.0)
    }

    // ── Pink ─────────────────────────────────────────────────────

    pub fn pink_50() -> Hsla {
        hsla(0.9167, 1.0, 0.95, 1.0)
    }
    pub fn pink_100() -> Hsla {
        hsla(0.9167, 1.0, 0.85, 1.0)
    }
    pub fn pink_200() -> Hsla {
        hsla(0.9167, 1.0, 0.75, 1.0)
    }
    pub fn pink_300() -> Hsla {
        hsla(0.9167, 1.0, 0.65, 1.0)
    }
    pub fn pink_400() -> Hsla {
        hsla(0.9167, 1.0, 0.55, 1.0)
    }
    pub fn pink_500() -> Hsla {
        hsla(0.9167, 1.0, 0.50, 1.0)
    }
    pub fn pink_600() -> Hsla {
        hsla(0.9167, 1.0, 0.42, 1.0)
    }
    pub fn pink_700() -> Hsla {
        hsla(0.9167, 1.0, 0.33, 1.0)
    }
    pub fn pink_800() -> Hsla {
        hsla(0.9167, 1.0, 0.22, 1.0)
    }
    pub fn pink_900() -> Hsla {
        hsla(0.9167, 1.0, 0.12, 1.0)
    }

    // ── Grey / Neutral ───────────────────────────────────────────

    pub fn grey_50() -> Hsla {
        hsla(0.0, 0.0, 0.95, 1.0)
    }
    pub fn grey_100() -> Hsla {
        hsla(0.0, 0.0, 0.85, 1.0)
    }
    pub fn grey_200() -> Hsla {
        hsla(0.0, 0.0, 0.75, 1.0)
    }
    pub fn grey_300() -> Hsla {
        hsla(0.0, 0.0, 0.65, 1.0)
    }
    pub fn grey_400() -> Hsla {
        hsla(0.0, 0.0, 0.55, 1.0)
    }
    pub fn grey_500() -> Hsla {
        hsla(0.0, 0.0, 0.50, 1.0)
    }
    pub fn grey_600() -> Hsla {
        hsla(0.0, 0.0, 0.42, 1.0)
    }
    pub fn grey_700() -> Hsla {
        hsla(0.0, 0.0, 0.33, 1.0)
    }
    pub fn grey_800() -> Hsla {
        hsla(0.0, 0.0, 0.22, 1.0)
    }
    pub fn grey_900() -> Hsla {
        hsla(0.0, 0.0, 0.12, 1.0)
    }
}
