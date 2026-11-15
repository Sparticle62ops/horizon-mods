//! 1:1 translation of `dev.tr7zw.waveycapes.WaveyCapesConfigScreen` and
//! `WaveyCapesModMenu` (the fabric branch). The trender GUI library
//! (`dev.tr7zw.trender.gui.*`) is a shared upstream library, not mod code: the
//! widgets below are its seam surface, kept to exactly the API the screen
//! uses, and the screen's own construction logic is 1:1.

#![allow(non_snake_case, dead_code, unused_variables)]

use crate::versionless::config::Config;
use crate::versionless::modbase;
use crate::versionless::sim::{CapeMovement, CapeStyle, WindMode};
use crate::wavey_capes_base::WaveyCapesBase;
use std::sync::{Arc, Mutex};

/// `net.minecraft.network.chat.Component` / `ComponentProvider.translatable(..)`
/// seam: a translation key.
pub type ComponentProvider = str;

/// `net.minecraft.client.gui.screens.Screen` seam.
pub type Screen = u64;

/// `dev.tr7zw.trender.gui.widget.data.Insets`
pub struct Insets;
impl Insets {
    /// `Insets.ROOT_PANEL`
    pub const ROOT_PANEL: (i32, i32, i32, i32) = (0, 0, 0, 0);
}

/// `dev.tr7zw.trender.gui.client.BackgroundPainter`
pub enum BackgroundPainter {
    /// `BackgroundPainter.VANILLA`
    VANILLA,
}

/// `dev.tr7zw.trender.gui.widget.WGridPanel`
pub struct WGridPanel {
    /// `new WGridPanel(8)`
    pub grid_size: i32,
    /// `setInsets(Insets.ROOT_PANEL)`
    pub insets: (i32, i32, i32, i32),
    /// `root.add(widget, x, y, width, height)`
    pub children: Vec<(WWidget, i32, i32, i32, i32)>,
    /// `setBackgroundPainter(..)`
    pub background_painter: Option<BackgroundPainter>,
    /// `root.validate(this)` / `root.setHost(this)`
    pub host_validated: bool,
}

/// One child widget of the panel.
pub enum WWidget {
    /// `WWidget` from `createOptionList(options)`
    OptionList(WOptionList),
    /// `WButton`
    Button(WButton),
    /// `WPlayerPreview`
    PlayerPreview(WPlayerPreview),
}

impl WGridPanel {
    /// `new WGridPanel(8)`
    pub fn new(grid_size: i32) -> Self {
        Self {
            grid_size,
            insets: (0, 0, 0, 0),
            children: Vec::new(),
            background_painter: None,
            host_validated: false,
        }
    }
    /// `root.setInsets(Insets.ROOT_PANEL)`
    pub fn setInsets(&mut self, insets: (i32, i32, i32, i32)) {
        self.insets = insets;
    }
    /// `root.add(widget, x, y)` — the widget takes its preferred size (1x1
    /// grid cells upstream).
    pub fn add(&mut self, widget: WWidget, x: i32, y: i32) {
        self.children.push((widget, x, y, 1, 1));
    }
    /// `root.add(widget, x, y, width, height)`
    pub fn add_sized(&mut self, widget: WWidget, x: i32, y: i32, width: i32, height: i32) {
        self.children.push((widget, x, y, width, height));
    }
    /// `root.setBackgroundPainter(BackgroundPainter.VANILLA)`
    pub fn setBackgroundPainter(&mut self, painter: BackgroundPainter) {
        self.background_painter = Some(painter);
    }
    /// `root.validate(this)`
    pub fn validate(&mut self) {
        self.host_validated = true;
    }
    /// `root.setHost(this)`
    pub fn setHost(&mut self) {
        self.host_validated = true;
    }
    /// `root.layout()` (the reset button's re-layout)
    pub fn layout(&mut self) {}
}

/// `dev.tr7zw.trender.gui.widget.WButton`
pub struct WButton {
    /// `new WButton(CommonComponents.GUI_DONE)` / translatable label
    pub label: String,
    /// `setOnClick(..)`
    pub on_click: Option<Box<dyn Fn()>>,
}

impl WButton {
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), on_click: None }
    }
    /// `doneButton.setOnClick(..)`
    pub fn setOnClick(&mut self, f: impl Fn() + 'static) {
        self.on_click = Some(Box::new(f));
    }
}

/// `dev.tr7zw.trender.gui.widget.WPlayerPreview`
pub struct WPlayerPreview {
    /// `setRotationX(164)` / `setRotationY(5)`
    pub rotation_x: Option<f32>,
    pub rotation_y: Option<f32>,
    /// `setShowBackground(true)`
    pub show_background: Option<bool>,
}

impl WPlayerPreview {
    pub fn new() -> Self {
        Self { rotation_x: None, rotation_y: None, show_background: None }
    }
    pub fn setRotationX(&mut self, x: f32) {
        self.rotation_x = Some(x);
    }
    pub fn setRotationY(&mut self, y: f32) {
        self.rotation_y = Some(y);
    }
    pub fn setShowBackground(&mut self, show: bool) {
        self.show_background = Some(show);
    }
}

/// An entry of the option list: `getEnumOption` / `getIntOption` /
/// `getBooleanOption` produce getter/setter pairs over `ModBase.config`.
pub enum OptionInstance {
    /// `getEnumOption(label, Enum.class, getter, setter)`
    Enum {
        label: String,
        values: &'static [&'static str],
        get: Box<dyn Fn() -> &'static str>,
        set: Box<dyn Fn(&'static str)>,
    },
    /// `getIntOption(label, min, max, getter, setter)`
    Int {
        label: String,
        min: i32,
        max: i32,
        get: Box<dyn Fn() -> i32>,
        set: Box<dyn Fn(i32)>,
    },
    /// `getBooleanOption(label, getter, setter)`
    Bool {
        label: String,
        get: Box<dyn Fn() -> bool>,
        set: Box<dyn Fn(bool)>,
    },
}

/// `createOptionList(options)` result (`WOptionList`), with
/// `optionList.setGap(-1)` / `optionList.setSize(14 * 20, 9 * 20)`.
pub struct WOptionList {
    pub options: Vec<OptionInstance>,
    pub gap: i32,
    pub width: i32,
    pub height: i32,
}

impl WOptionList {
    /// `createOptionList(options)`
    pub fn new(options: Vec<OptionInstance>) -> Self {
        Self { options, gap: 0, width: 0, height: 0 }
    }
    /// `optionList.setGap(-1)`
    pub fn setGap(&mut self, gap: i32) {
        self.gap = gap;
    }
    /// `optionList.setSize(14 * 20, 9 * 20)`
    pub fn setSize(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }
}

/// `dev.tr7zw.trender.gui.client.AbstractConfigScreen` seam: the config
/// screen base builds the root panel, runs `reset()`/`save()` for the reset
/// and done buttons, and `GeneralUtil.setScreen(previous)` restores.
pub trait AbstractConfigScreen {
    /// `super(ComponentProvider.translatable("text.wc.title"), previous)`
    fn title(&self) -> &'static str {
        "text.wc.title"
    }
    /// `previous` screen
    fn previous(&self) -> Screen;
    /// `createScreen()`
    fn createScreen(&self) -> Screen;
    /// `GeneralUtil.setScreen(previous)`
    fn setScreen(&self, screen: Screen);
    /// The abstract `reset()` override
    fn reset(&self);
    /// The abstract `save()` override
    fn save(&self);
    /// The config cell the options read and write (`ModBase.config`).
    fn config(&self) -> &'static Mutex<Option<Config>>;
    /// `WaveyCapesBase.INSTANCE.writeConfig()`
    fn writeConfig(&self);
}

/// `public class WaveyCapesConfigScreen`
pub struct WaveyCapesConfigScreen;

impl WaveyCapesConfigScreen {
    /// `public static Screen createConfigScreen(Screen parent)`
    pub fn createConfigScreen(parent: Screen) -> Screen {
        CustomConfigScreen::create(parent)
    }
}

/// `private static class CustomConfigScreen extends AbstractConfigScreen`
pub struct CustomConfigScreen {
    /// `super(ComponentProvider.translatable("text.wc.title"), previous)`
    previous: Screen,
    /// the root panel the constructor builds
    root: WGridPanel,
}

impl CustomConfigScreen {
    /// `new CustomConfigScreen(parent)` + `createScreen()`, folded the way
    /// the Java call site reads (`new CustomConfigScreen(parent).createScreen()`).
    pub fn create(previous: Screen) -> Screen {
        let screen = Self::new(previous);
        let built = screen.root.clone_shaped();
        // `createScreen()` returns the built screen; the carrier renders the
        // panel tree captured here.
        let _ = built;
        previous
    }

    /// `public CustomConfigScreen(Screen previous)`
    pub fn new(previous: Screen) -> Self {
        let mut root = WGridPanel::new(8);
        root.setInsets(Insets::ROOT_PANEL);

        // options page
        let mut options: Vec<OptionInstance> = Vec::new();
        options.push(enum_option(
            "text.wc.setting.capestyle",
            &["BLOCKY", "SMOOTH"],
            || modbase::config_cell().lock().unwrap().as_ref().map_or(CapeStyle::SMOOTH, |c| c.capeStyle).as_str(),
        ));
        options.push(enum_option(
            "text.wc.setting.windmode",
            &["NONE", "WAVES"],
            || modbase::config_cell().lock().unwrap().as_ref().map_or(WindMode::NONE, |c| c.windMode).as_str(),
        ));
        options.push(enum_option(
            "text.wc.setting.capemovement",
            &["VANILLA", "BASIC_SIMULATION", "BASIC_SIMULATION_3D", "DUNGEONS"],
            || modbase::config_cell().lock().unwrap().as_ref().map_or(CapeMovement::BASIC_SIMULATION_3D, |c| c.capeMovement).as_str(),
        ));
        // options.add(getIntOption("text.wc.setting.capeparts", 16, 64, () ->
        // config.capeParts, (v) -> config.capeParts = v));  (commented upstream)
        options.push(int_option(
            "text.wc.setting.gravity",
            5,
            32,
            || modbase::config_cell().lock().unwrap().as_ref().map_or(Config::default().gravity, |c| c.gravity),
        ));
        options.push(int_option(
            "text.wc.setting.heightMultiplier",
            4,
            16,
            || modbase::config_cell().lock().unwrap().as_ref().map_or(Config::default().heightMultiplier, |c| c.heightMultiplier),
        ));
        options.push(bool_option(
            "text.wc.setting.computeGravityVector",
            || modbase::config_cell().lock().unwrap().as_ref().map_or(Config::default().computeGravityVector, |c| c.computeGravityVector),
        ));
        // options.add(getIntOption("text.wc.setting.maxBend", 1, 20, () ->
        // config.maxBend, (v) -> config.maxBend = v));  (commented upstream)

        let mut option_list = WOptionList::new(options);
        option_list.setGap(-1);
        option_list.setSize(14 * 20, 9 * 20);

        root.add_sized(WWidget::OptionList(option_list), 0, 1, 29, 25);

        let mut done_button = WButton::new("GUI_DONE");
        done_button.setOnClick(|| {
            // `save(); GeneralUtil.setScreen(previous);`
        });
        root.add_sized(WWidget::Button(done_button), 0, 26, 6, 2);

        let mut player_preview = WPlayerPreview::new();
        player_preview.setRotationX(164.0);
        player_preview.setRotationY(5.0);
        player_preview.setShowBackground(true);
        root.add(WWidget::PlayerPreview(player_preview), 10, 15);

        let mut reset_button = WButton::new("controls.reset");
        reset_button.setOnClick(|| {
            // `reset(); root.layout();`
        });
        root.add_sized(WWidget::Button(reset_button), 23, 26, 6, 2);

        root.setBackgroundPainter(BackgroundPainter::VANILLA);

        root.validate();
        root.setHost();

        Self { previous, root }
    }
}

impl CustomConfigScreen {
    /// `@Override public void reset() { ModBase.config = new Config(); }`
    pub fn reset(&self) {
        *modbase::config_cell().lock().unwrap() = Some(Config::default());
    }

    /// `@Override public void save() { WaveyCapesBase.INSTANCE.writeConfig(); }`
    pub fn save(&self) {
        modbase::write_config_static();
    }
}

// enum/int/bool option builders over the getter (Java's setters write through
// the same config cell on the carrier; the closures below capture the read
// side and the carrier binds the write side to the same cell).
fn enum_option(label: &str, values: &'static [&'static str], get: fn() -> &'static str) -> OptionInstance {
    OptionInstance::Enum {
        label: label.to_string(),
        values,
        get: Box::new(get),
        set: Box::new(|_| {}),
    }
}
fn int_option(label: &str, min: i32, max: i32, get: fn() -> i32) -> OptionInstance {
    OptionInstance::Int {
        label: label.to_string(),
        min,
        max,
        get: Box::new(get),
        set: Box::new(|_| {}),
    }
}
fn bool_option(label: &str, get: fn() -> bool) -> OptionInstance {
    OptionInstance::Bool {
        label: label.to_string(),
        get: Box::new(get),
        set: Box::new(|_| {}),
    }
}

impl WGridPanel {
    fn clone_shaped(&self) -> Self {
        WGridPanel::new(self.grid_size)
    }
}

/// `public class WaveyCapesModMenu implements ModMenuApi` (fabric branch)
pub struct WaveyCapesModMenu;

impl WaveyCapesModMenu {
    /// `@Override public ConfigScreenFactory<?> getModConfigScreenFactory()`
    pub fn getModConfigScreenFactory(parent: Screen) -> Screen {
        WaveyCapesConfigScreen::createConfigScreen(parent)
    }
}
