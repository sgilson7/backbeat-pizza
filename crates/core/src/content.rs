//! The data files, read and checked. Every word and colour the game uses
//! comes from `data/`; this module turns them into values and refuses a file
//! the rules cannot use, with a message that names what is wrong.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

use crate::fx::ONE;
use crate::puzzle::{Clarity, Layout};

/// A colour as three Q16 channels.
pub type Rgb = [i32; 3];

/// `#rrggbb` to Q16 channels.
pub fn rgb(hex: &str) -> Result<Rgb, String> {
    let h = hex.strip_prefix('#').unwrap_or("");
    if h.len() != 6 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("\"{hex}\" is not a colour written as #rrggbb."));
    }
    let ch = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).map(|v| v as i32 * ONE / 255);
    Ok([ch(0).unwrap_or(0), ch(2).unwrap_or(0), ch(4).unwrap_or(0)])
}

/// The 3D piece an ingredient is drawn as (and the icon the page shows).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Shape {
    Thin,
    Thick,
    Stuffed,
    Sauce,
    Disc,
    Dome,
    Ring,
    Arc,
    Cube,
    Hoop,
    Blob,
    Fish,
}

impl Shape {
    pub fn name(&self) -> &'static str {
        match self {
            Shape::Thin => "thin",
            Shape::Thick => "thick",
            Shape::Stuffed => "stuffed",
            Shape::Sauce => "sauce",
            Shape::Disc => "disc",
            Shape::Dome => "dome",
            Shape::Ring => "ring",
            Shape::Arc => "arc",
            Shape::Cube => "cube",
            Shape::Hoop => "hoop",
            Shape::Blob => "blob",
            Shape::Fish => "fish",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawIngredient {
    name: String,
    color: String,
    shape: Shape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ingredient {
    pub name: String,
    pub css: String,
    pub color: Rgb,
    pub shape: Shape,
}

fn ingredient(r: RawIngredient) -> Result<Ingredient, String> {
    if r.name.trim().is_empty() {
        return Err("An ingredient in data/menu.json has no name.".into());
    }
    Ok(Ingredient {
        color: rgb(&r.color)?,
        css: r.color,
        name: r.name,
        shape: r.shape,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerDef {
    pub key: String,
    pub name: String,
    pub options: Vec<Ingredient>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Menu {
    /// in the order a pizza is stacked: crust before sauce
    pub layers: Vec<LayerDef>,
    pub toppings: Vec<Ingredient>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelDef {
    pub name: String,
    pub drummers: usize,
    /// indices into `Menu::layers`
    pub layers: Vec<usize>,
    pub layout: Layout,
    pub clarity: Clarity,
    pub patience: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drummer {
    pub name: String,
    pub step_ms: i64,
    pub shirt: Rgb,
    pub band: Rgb,
    pub skin: Rgb,
    pub shell: Rgb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Drum {
    Hat,
    Snare,
    Kick,
    TomHi,
    TomLo,
    Crash,
}

impl Drum {
    pub const ALL: [Drum; 6] = [
        Drum::Hat,
        Drum::Snare,
        Drum::Kick,
        Drum::TomHi,
        Drum::TomLo,
        Drum::Crash,
    ];
    pub fn key(&self) -> &'static str {
        match self {
            Drum::Hat => "hat",
            Drum::Snare => "snare",
            Drum::Kick => "kick",
            Drum::TomHi => "tom_hi",
            Drum::TomLo => "tom_lo",
            Drum::Crash => "crash",
        }
    }
}

/// One riff: its length in sixteenth notes and its hits, in step order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Riff {
    pub steps: u32,
    pub hits: Vec<(u32, Drum)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Riffs {
    pub perfect: Vec<Riff>,
    pub close: Vec<Riff>,
    pub miss: Vec<Riff>,
}

/// Everything read from `data/`.
#[derive(Debug, Clone)]
pub struct Content {
    pub menu: Menu,
    pub levels: Vec<LevelDef>,
    pub band: Vec<Drummer>,
    pub hints: BTreeMap<String, Vec<String>>,
    pub riffs: Riffs,
    pub copy: BTreeMap<String, Value>,
    pub scene: BTreeMap<String, Rgb>,
    pub masklings: Masklings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Masklings {
    pub mask: Rgb,
    pub hole: Rgb,
    pub robes: Vec<Rgb>,
}

fn masklings(text: &str) -> Result<Masklings, String> {
    #[derive(Deserialize)]
    struct Raw {
        mask: String,
        hole: String,
        robes: Vec<String>,
    }
    let raw: Raw = parse("masklings.json", text)?;
    let robes = raw.robes.iter().map(|h| rgb(h)).collect::<Result<Vec<_>, _>>()?;
    if robes.is_empty() {
        return Err("data/masklings.json lists no robes.".into());
    }
    Ok(Masklings {
        mask: rgb(&raw.mask)?,
        hole: rgb(&raw.hole)?,
        robes,
    })
}

/// The hint lists the game draws from; each must hold at least one line.
pub const HINT_KEYS: [&str; 9] = [
    "perfect",
    "close_one",
    "close_many",
    "close_vague",
    "miss_named_one",
    "miss_named_many",
    "miss_counted_one",
    "miss_counted_many",
    "miss_vague",
];

/// The copy keys the core fills; the page's own keys are its business.
pub const COPY_KEYS: [&str; 21] = [
    "level_label",
    "drummers_one",
    "drummers_many",
    "layers_none",
    "layers_some",
    "toppings_rule",
    "clarity_named",
    "clarity_counted",
    "clarity_vague",
    "patience_rule",
    "patience_value",
    "serve_to",
    "verdict_perfect",
    "verdict_close",
    "verdict_miss",
    "won_text",
    "lost_text",
    "plain",
    "tasting",
    "fed",
    "log_number",
];

/// The scene colours the core needs from `data/scene.json`.
pub const SCENE_KEYS: [&str; 17] = [
    "sky",
    "ink",
    "light",
    "ambient",
    "floor",
    "wall",
    "curtain",
    "riser",
    "counter",
    "counter_top",
    "chrome",
    "cymbal",
    "drumhead",
    "bulb",
    "speaker",
    "spot",
    "stick",
];

fn parse<T: for<'de> Deserialize<'de>>(name: &str, text: &str) -> Result<T, String> {
    serde_json::from_str(text).map_err(|e| format!("data/{name} could not be read: {e}"))
}

fn menu(text: &str) -> Result<Menu, String> {
    #[derive(Deserialize)]
    struct RawLayer {
        name: String,
        options: Vec<RawIngredient>,
    }
    #[derive(Deserialize)]
    struct Raw {
        layers: BTreeMap<String, RawLayer>,
        toppings: Vec<RawIngredient>,
    }
    let raw: Raw = parse("menu.json", text)?;
    let mut layers = Vec::new();
    // stacking order: crust under sauce; any other layer after them by name
    let mut keys: Vec<&String> = raw.layers.keys().collect();
    keys.sort_by_key(|k| match k.as_str() {
        "crust" => 0,
        "sauce" => 1,
        _ => 2,
    });
    for key in keys {
        let l = &raw.layers[key];
        let options = l
            .options
            .iter()
            .cloned()
            .map(ingredient)
            .collect::<Result<Vec<_>, _>>()?;
        layers.push(LayerDef {
            key: key.clone(),
            name: l.name.clone(),
            options,
        });
    }
    let toppings = raw
        .toppings
        .into_iter()
        .map(ingredient)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Menu { layers, toppings })
}

fn levels(text: &str, menu: &Menu, band: usize) -> Result<Vec<LevelDef>, String> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct RawLevel {
        name: String,
        drummers: usize,
        layers: Vec<String>,
        toppings: u8,
        min: u8,
        max: u8,
        clarity: Clarity,
        patience: u32,
    }
    #[derive(Deserialize)]
    struct Raw {
        levels: Vec<RawLevel>,
    }
    let raw: Raw = parse("levels.json", text)?;
    let mut out = Vec::new();
    for l in raw.levels {
        let mut layers = Vec::new();
        for key in &l.layers {
            let at = menu.layers.iter().position(|m| &m.key == key).ok_or_else(|| {
                format!(
                    "Level \"{}\" uses a layer \"{key}\" the menu does not have.",
                    l.name
                )
            })?;
            layers.push(at);
        }
        layers.sort();
        if l.toppings as usize > menu.toppings.len() {
            return Err(format!(
                "Level \"{}\" uses {} toppings; the menu has {}.",
                l.name,
                l.toppings,
                menu.toppings.len()
            ));
        }
        if l.drummers == 0 || l.drummers > band {
            return Err(format!(
                "Level \"{}\" has {} drummers; the band has {band}.",
                l.name, l.drummers
            ));
        }
        let layout = Layout {
            picks: layers
                .iter()
                .map(|&k| menu.layers[k].options.len() as u8)
                .collect(),
            toppings: l.toppings,
            min: l.min,
            max: l.max,
        };
        layout.check().map_err(|e| format!("Level \"{}\": {e}", l.name))?;
        out.push(LevelDef {
            name: l.name,
            drummers: l.drummers,
            layers,
            layout,
            clarity: l.clarity,
            patience: l.patience,
        });
    }
    if out.is_empty() {
        return Err("data/levels.json lists no levels.".into());
    }
    Ok(out)
}

fn band(text: &str) -> Result<Vec<Drummer>, String> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct RawDrummer {
        name: String,
        step_ms: i64,
        shirt: String,
        band: String,
        skin: String,
        shell: String,
    }
    #[derive(Deserialize)]
    struct Raw {
        drummers: Vec<RawDrummer>,
    }
    let raw: Raw = parse("band.json", text)?;
    raw.drummers
        .into_iter()
        .map(|d| {
            if !(40..=400).contains(&d.step_ms) {
                return Err(format!(
                    "{}'s step_ms is {}; it needs 40 to 400.",
                    d.name, d.step_ms
                ));
            }
            Ok(Drummer {
                shirt: rgb(&d.shirt)?,
                band: rgb(&d.band)?,
                skin: rgb(&d.skin)?,
                shell: rgb(&d.shell)?,
                step_ms: d.step_ms,
                name: d.name,
            })
        })
        .collect()
}

/// One grid riff: a row of x and . per drum, all the same length.
pub fn riff(rows: &BTreeMap<String, String>) -> Result<Riff, String> {
    let mut steps = None;
    let mut hits = Vec::new();
    for (key, row) in rows {
        let drum = Drum::ALL
            .iter()
            .find(|d| d.key() == key)
            .ok_or_else(|| format!("A riff names a drum \"{key}\" the kit does not have."))?;
        let n = row.chars().count() as u32;
        if *steps.get_or_insert(n) != n {
            return Err(format!(
                "A riff's \"{key}\" row is {n} steps; the others are not."
            ));
        }
        for (k, c) in row.chars().enumerate() {
            match c {
                'x' => hits.push((k as u32, *drum)),
                '.' => {}
                other => {
                    return Err(format!(
                        "A riff's \"{key}\" row holds \"{other}\"; use x or a dot."
                    ))
                }
            }
        }
    }
    hits.sort();
    Ok(Riff {
        steps: steps.unwrap_or(0),
        hits,
    })
}

fn riffs(text: &str) -> Result<Riffs, String> {
    #[derive(Deserialize)]
    struct Raw {
        perfect: Vec<BTreeMap<String, String>>,
        close: Vec<BTreeMap<String, String>>,
        miss: Vec<BTreeMap<String, String>>,
    }
    let raw: Raw = parse("riffs.json", text)?;
    let all = |v: &[BTreeMap<String, String>], kind: &str| -> Result<Vec<Riff>, String> {
        if v.is_empty() {
            return Err(format!("data/riffs.json has no {kind} riff."));
        }
        v.iter().map(riff).collect()
    };
    Ok(Riffs {
        perfect: all(&raw.perfect, "perfect")?,
        close: all(&raw.close, "close")?,
        miss: all(&raw.miss, "miss")?,
    })
}

fn hints(text: &str) -> Result<BTreeMap<String, Vec<String>>, String> {
    let raw: BTreeMap<String, Value> = parse("hints.json", text)?;
    let mut out = BTreeMap::new();
    for key in HINT_KEYS {
        let lines: Vec<String> = raw
            .get(key)
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|s| s.as_str().map(String::from)).collect())
            .unwrap_or_default();
        if lines.is_empty() {
            return Err(format!("data/hints.json has no lines under \"{key}\"."));
        }
        out.insert(key.to_string(), lines);
    }
    Ok(out)
}

fn copy(text: &str) -> Result<BTreeMap<String, Value>, String> {
    let raw: BTreeMap<String, Value> = parse("copy.json", text)?;
    for key in COPY_KEYS {
        if raw
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.trim().is_empty())
            .unwrap_or(true)
        {
            return Err(format!("data/copy.json has no \"{key}\"."));
        }
    }
    Ok(raw)
}

fn scene(text: &str) -> Result<BTreeMap<String, Rgb>, String> {
    let raw: BTreeMap<String, Value> = parse("scene.json", text)?;
    let mut out = BTreeMap::new();
    for key in SCENE_KEYS {
        let hex = raw
            .get(key)
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("data/scene.json has no colour \"{key}\"."))?;
        out.insert(key.to_string(), rgb(hex)?);
    }
    Ok(out)
}

/// The data files' text, by file name without `.json`.
pub struct Files<'a> {
    pub menu: &'a str,
    pub levels: &'a str,
    pub band: &'a str,
    pub hints: &'a str,
    pub riffs: &'a str,
    pub copy: &'a str,
    pub scene: &'a str,
    pub masklings: &'a str,
}

impl Content {
    pub fn load(f: &Files) -> Result<Content, String> {
        let menu = menu(f.menu)?;
        let band = band(f.band)?;
        let levels = levels(f.levels, &menu, band.len())?;
        Ok(Content {
            levels,
            band,
            menu,
            hints: hints(f.hints)?,
            riffs: riffs(f.riffs)?,
            copy: copy(f.copy)?,
            scene: scene(f.scene)?,
            masklings: masklings(f.masklings)?,
        })
    }

    /// A copy string with its placeholders filled.
    pub fn say(&self, key: &str, fills: &[(&str, String)]) -> String {
        let mut s = self
            .copy
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or(key)
            .to_string();
        for (k, v) in fills {
            s = s.replace(&format!("{{{k}}}"), v);
        }
        s
    }

    pub fn color(&self, key: &str) -> Rgb {
        self.scene.get(key).copied().unwrap_or([ONE; 3])
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn files() -> Files<'static> {
        Files {
            menu: include_str!("../../../data/menu.json"),
            levels: include_str!("../../../data/levels.json"),
            band: include_str!("../../../data/band.json"),
            hints: include_str!("../../../data/hints.json"),
            riffs: include_str!("../../../data/riffs.json"),
            copy: include_str!("../../../data/copy.json"),
            scene: include_str!("../../../data/scene.json"),
            masklings: include_str!("../../../data/masklings.json"),
        }
    }

    #[test]
    fn the_shipped_data_loads() {
        let c = Content::load(&files()).unwrap();
        assert_eq!(c.levels.len(), 10);
        assert_eq!(c.band.len(), 4);
    }

    #[test]
    fn colours_are_read_as_q16_channels() {
        assert_eq!(rgb("#ff0080").unwrap(), [ONE, 0, 128 * ONE / 255]);
        assert!(rgb("ff0080").is_err());
        assert!(rgb("#ff00").is_err());
    }

    #[test]
    fn a_riff_grid_becomes_hits_in_step_order() {
        let mut rows = BTreeMap::new();
        rows.insert("kick".to_string(), "x..x".to_string());
        rows.insert("snare".to_string(), ".x..".to_string());
        let r = riff(&rows).unwrap();
        assert_eq!(r.steps, 4);
        assert_eq!(r.hits, vec![(0, Drum::Kick), (1, Drum::Snare), (3, Drum::Kick)]);
    }

    #[test]
    fn a_riff_with_rows_of_different_lengths_or_an_unknown_drum_is_refused() {
        let mut rows = BTreeMap::new();
        rows.insert("kick".to_string(), "x..x".to_string());
        rows.insert("snare".to_string(), ".x.".to_string());
        assert!(riff(&rows).is_err());
        let mut rows = BTreeMap::new();
        rows.insert("cowbell".to_string(), "x".to_string());
        assert!(riff(&rows).unwrap_err().contains("cowbell"));
    }

    #[test]
    fn a_level_that_asks_for_more_drummers_than_the_band_has_is_refused() {
        let f = files();
        let bad = f.levels.replacen("\"drummers\": 1", "\"drummers\": 9", 1);
        assert_ne!(bad, f.levels);
        let e = Content::load(&Files { levels: &bad, ..f }).unwrap_err();
        assert!(e.contains("9 drummers"), "{e}");
    }

    #[test]
    fn a_missing_hint_list_is_named() {
        let f = files();
        let bad = f.hints.replacen("\"miss_vague\"", "\"miss_hazy\"", 1);
        assert_ne!(bad, f.hints);
        let e = Content::load(&Files { hints: &bad, ..f }).unwrap_err();
        assert!(e.contains("miss_vague"), "{e}");
    }

    #[test]
    fn placeholders_are_filled_by_name() {
        let c = Content::load(&files()).unwrap();
        assert_eq!(c.say("level_label", &[("n", "7".into())]), "Level 7");
    }
}
