use windows_reactor::*;

#[derive(Clone, Copy)]
pub struct Palette {
    pub app_bg: Color,
    pub sidebar_bg: Color,
    pub card_bg: Color,
    pub stroke: Color,
    pub accent: Color,
    pub accent_text: Color,
    pub text_primary: Color,
    pub text_muted: Color,
    pub tile_bg: Color,
}

pub const SIDEBAR_WIDTH: f64 = 220.0;
/// `Thickness::uniform` is not `const` in windows-reactor 0.100.
pub const PAGE_MARGIN: f64 = 24.0;
pub const CARD_RADIUS: f64 = 8.0;
pub const TITLE_SIZE: f64 = 28.0;
pub const SUBTITLE_SIZE: f64 = 14.0;
pub const ROW_NAME_SIZE: f64 = 16.0;

// Each SKU palette is authored here. The accent is the central identity color;
// the surrounding surfaces are tinted to support it while preserving contrast.
pub const SKU_PALETTES: [Palette; 5] = [
    Palette {
        // Retail: Midnight purple
        app_bg: Color::rgb(0x13, 0x12, 0x17),
        sidebar_bg: Color::rgb(0x19, 0x17, 0x1E),
        card_bg: Color::rgb(0x21, 0x1E, 0x27),
        stroke: Color::rgb(0x3A, 0x35, 0x43),
        accent: Color::rgb(0xA5, 0x8D, 0xC7),
        accent_text: Color::rgb(0x19, 0x14, 0x20),
        text_primary: Color::rgb(0xF1, 0xEF, 0xF3),
        text_muted: Color::rgb(0xA8, 0xA2, 0xAE),
        tile_bg: Color::rgb(0x2B, 0x27, 0x32),
    },
    Palette {
        // Mists of Pandaria: jade
        app_bg: Color::rgb(0x11, 0x17, 0x16),
        sidebar_bg: Color::rgb(0x16, 0x1E, 0x1C),
        card_bg: Color::rgb(0x1D, 0x29, 0x26),
        stroke: Color::rgb(0x34, 0x48, 0x42),
        accent: Color::rgb(0x78, 0xB7, 0x9F),
        accent_text: Color::rgb(0x0E, 0x1B, 0x17),
        text_primary: Color::rgb(0xEE, 0xF3, 0xF1),
        text_muted: Color::rgb(0x9F, 0xAE, 0xA9),
        tile_bg: Color::rgb(0x27, 0x34, 0x31),
    },
    Palette {
        // Classic: cool forged metal
        app_bg: Color::rgb(0x14, 0x16, 0x19),
        sidebar_bg: Color::rgb(0x1A, 0x1D, 0x21),
        card_bg: Color::rgb(0x22, 0x26, 0x2B),
        stroke: Color::rgb(0x3C, 0x42, 0x49),
        accent: Color::rgb(0xA9, 0xB0, 0xB8),
        accent_text: Color::rgb(0x18, 0x1A, 0x1D),
        text_primary: Color::rgb(0xEF, 0xF0, 0xF2),
        text_muted: Color::rgb(0x9F, 0xA4, 0xAA),
        tile_bg: Color::rgb(0x2C, 0x31, 0x36),
    },
    Palette {
        // Burning Crusade: fel green
        app_bg: Color::rgb(0x13, 0x17, 0x11),
        sidebar_bg: Color::rgb(0x19, 0x1E, 0x16),
        card_bg: Color::rgb(0x22, 0x2A, 0x1D),
        stroke: Color::rgb(0x3B, 0x49, 0x32),
        accent: Color::rgb(0x91, 0xB8, 0x76),
        accent_text: Color::rgb(0x14, 0x1B, 0x0F),
        text_primary: Color::rgb(0xEF, 0xF2, 0xEC),
        text_muted: Color::rgb(0xA4, 0xAE, 0x9D),
        tile_bg: Color::rgb(0x2C, 0x35, 0x26),
    },
    Palette {
        // Forever: sky blue (reserved for the upcoming SKU)
        app_bg: Color::rgb(0x11, 0x16, 0x1A),
        sidebar_bg: Color::rgb(0x16, 0x1D, 0x22),
        card_bg: Color::rgb(0x1C, 0x28, 0x2F),
        stroke: Color::rgb(0x32, 0x45, 0x50),
        accent: Color::rgb(0x79, 0xAC, 0xC8),
        accent_text: Color::rgb(0x0D, 0x18, 0x1E),
        text_primary: Color::rgb(0xED, 0xF1, 0xF3),
        text_muted: Color::rgb(0x9C, 0xAA, 0xB2),
        tile_bg: Color::rgb(0x26, 0x33, 0x3A),
    },
];

pub fn palette(sku: usize) -> &'static Palette {
    SKU_PALETTES.get(sku).unwrap_or(&SKU_PALETTES[0])
}

pub fn sku_flair(sku: usize) -> View {
    let bytes = match sku {
        0 => Some(include_bytes!("../assets/generated/midnight-logo.png").as_slice()),
        4 => Some(include_bytes!("../assets/generated/forever-logo.png").as_slice()),
        _ => None,
    };

    match bytes {
        Some(bytes) => Image::new()
            .source_data(EncodedImage::from_static(bytes))
            .width(176.0)
            .height(142.0)
            .stretch(Stretch::Uniform)
            .horizontal_alignment(HorizontalAlignment::Center)
            .margin(Thickness::new(12.0, 8.0, 12.0, 12.0))
            .into(),
        None => Border::new().height(0.0).into(),
    }
}

/// Button builder that keeps `on_click` / `grid_column` chainable.
pub struct ThemedButton {
    button: Button,
    content: View,
}

impl ThemedButton {
    pub fn on_click(mut self, callback: impl IntoUnitCallback) -> Self {
        self.button = self.button.on_click(callback);
        self
    }

    pub fn grid_column(mut self, column: i32) -> Self {
        self.button = self.button.grid_column(column);
        self
    }

    pub fn vertical_alignment(mut self, value: VerticalAlignment) -> Self {
        self.button = self.button.vertical_alignment(value);
        self
    }

    pub fn enabled(mut self, value: bool) -> Self {
        self.button = self.button.is_enabled(value);
        self
    }

    pub fn min_width(mut self, value: f64) -> Self {
        self.button = self.button.min_width(value);
        self
    }
}

impl From<ThemedButton> for View {
    fn from(value: ThemedButton) -> Self {
        value.button.content(value.content)
    }
}

pub fn accent_button(palette: &Palette, label: impl Into<String>) -> ThemedButton {
    ThemedButton {
        button: Button::new()
            .style(ButtonStyle::Default)
            .resource_overrides(fill_resources(palette.accent, palette.accent_text)),
        content: TextBlock::new()
            .text(label.into())
            .font_weight(FontWeight::SEMI_BOLD)
            .foreground(palette.accent_text)
            .into(),
    }
}

pub fn outline_button(palette: &Palette, label: impl Into<String>) -> ThemedButton {
    ThemedButton {
        button: Button::new()
            .style(ButtonStyle::Default)
            .resource_overrides(outline_resources(palette)),
        content: TextBlock::new()
            .text(label.into())
            .font_weight(FontWeight::SEMI_BOLD)
            .foreground(palette.accent)
            .into(),
    }
}

pub fn nav_button(
    palette: &Palette,
    label: impl Into<String>,
    symbol: Symbol,
    selected: bool,
) -> ThemedButton {
    let (foreground, style) = if selected {
        (palette.accent_text, ButtonStyle::Default)
    } else {
        (palette.text_muted, ButtonStyle::Subtle)
    };
    let mut button = Button::new()
        .style(style)
        .horizontal_alignment(HorizontalAlignment::Stretch)
        .horizontal_content_alignment(HorizontalAlignment::Left)
        .vertical_content_alignment(VerticalAlignment::Center);
    if selected {
        button = button.resource_overrides(fill_resources(palette.accent, palette.accent_text));
    }
    ThemedButton {
        button,
        content: StackPanel::new()
            .orientation(Orientation::Horizontal)
            .spacing(12.0)
            .children((
                SymbolIcon::new().symbol(symbol),
                TextBlock::new()
                    .text(label.into())
                    .font_weight(FontWeight::SEMI_BOLD)
                    .foreground(foreground),
            )),
    }
}

pub fn ribbon_button(
    palette: &Palette,
    label: impl Into<String>,
    icon: Option<&'static [u8]>,
    selected: bool,
) -> ThemedButton {
    let label = label.into();
    let icon_view: View = match icon {
        Some(bytes) => Image::new()
            .source_data(EncodedImage::from_static(bytes))
            .width(60.0)
            .height(60.0)
            .stretch(Stretch::UniformToFill)
            .into(),
        None => Border::new()
            .width(60.0)
            .height(60.0)
            .corner_radius(8.0)
            .background(palette.tile_bg)
            .content(
                TextBlock::new()
                    .text(label.chars().next().unwrap_or('?').to_string())
                    .font_weight(FontWeight::SEMI_BOLD)
                    .foreground(palette.accent)
                    .horizontal_alignment(HorizontalAlignment::Center)
                    .vertical_alignment(VerticalAlignment::Center),
            ),
    };
    let mut button = Button::new().style(ButtonStyle::Default);
    button = if selected {
        button.resource_overrides(fill_resources(palette.accent, palette.accent_text))
    } else {
        button.resource_overrides(outline_resources(palette))
    };
    ThemedButton {
        button: button.min_width(64.0),
        content: Grid::new().width(60.0).height(60.0).children((
            icon_view,
            Border::new()
                .vertical_alignment(VerticalAlignment::Bottom)
                .horizontal_alignment(HorizontalAlignment::Stretch)
                .background(Color::argb(0xD8, 0x08, 0x09, 0x0B))
                .padding(Thickness::xy(5.0, 4.0))
                .content(
                    TextBlock::new()
                        .text(label)
                        .font_size(10.0)
                        .font_weight(FontWeight::SEMI_BOLD)
                        .foreground(Color::rgb(0xFF, 0xFF, 0xFF))
                        .horizontal_alignment(HorizontalAlignment::Center),
                ),
        )),
    }
}

pub fn category_chip(palette: &Palette, category: &str) -> View {
    Border::new()
        .padding(Thickness::xy(10.0, 4.0))
        .corner_radius(4.0)
        .border_brush(palette.accent)
        .border_thickness(1.0)
        .background(palette.app_bg)
        .vertical_alignment(VerticalAlignment::Center)
        .content(
            TextBlock::new()
                .text(category.to_string())
                .font_size(11.0)
                .font_weight(FontWeight::SEMI_BOLD)
                .foreground(palette.accent),
        )
}

pub fn page_header(palette: &Palette, title: &str, subtitle: &str) -> View {
    StackPanel::new().spacing(4.0).children((
        TextBlock::new()
            .text(title.to_string())
            .font_size(TITLE_SIZE)
            .font_weight(FontWeight::SEMI_BOLD)
            .foreground(palette.text_primary),
        TextBlock::new()
            .text(subtitle.to_string())
            .font_size(SUBTITLE_SIZE)
            .foreground(palette.text_muted),
    ))
}

pub fn empty_state(palette: &Palette, title: &str, message: &str) -> View {
    Border::new()
        .background(palette.card_bg)
        .border_brush(palette.stroke)
        .border_thickness(1.0)
        .corner_radius(CARD_RADIUS)
        .padding(Thickness::uniform(32.0))
        .horizontal_alignment(HorizontalAlignment::Stretch)
        .content(
            StackPanel::new()
                .spacing(8.0)
                .horizontal_alignment(HorizontalAlignment::Center)
                .children((
                    TextBlock::new()
                        .text(title.to_string())
                        .font_size(18.0)
                        .font_weight(FontWeight::SEMI_BOLD)
                        .foreground(palette.text_primary)
                        .horizontal_alignment(HorizontalAlignment::Center),
                    TextBlock::new()
                        .text(message.to_string())
                        .foreground(palette.text_muted)
                        .horizontal_alignment(HorizontalAlignment::Center),
                )),
        )
}

pub fn installed_badge(palette: &Palette) -> View {
    Border::new()
        .padding(Thickness::xy(10.0, 4.0))
        .corner_radius(10.0)
        .background(palette.accent)
        .content(
            TextBlock::new()
                .text("✓  INSTALLED")
                .font_size(10.0)
                .font_weight(FontWeight::SEMI_BOLD)
                .foreground(palette.accent_text),
        )
}

pub fn stat_chip(palette: &Palette, text: impl Into<String>) -> View {
    Border::new()
        .padding(Thickness::xy(10.0, 5.0))
        .corner_radius(12.0)
        .background(palette.tile_bg)
        .content(
            TextBlock::new()
                .text(text.into())
                .font_size(12.0)
                .font_weight(FontWeight::SEMI_BOLD)
                .foreground(palette.text_muted),
        )
}

pub fn addon_icon_tile(palette: &Palette, name: &str) -> View {
    let initial: String = name
        .split_whitespace()
        .filter_map(|word| word.chars().next())
        .take(2)
        .collect();
    let initial = if initial.is_empty() {
        "?".to_string()
    } else {
        initial.to_uppercase()
    };
    Border::new()
        .width(48.0)
        .height(48.0)
        .corner_radius(CARD_RADIUS)
        .background(palette.tile_bg)
        .content(
            TextBlock::new()
                .text(initial)
                .font_size(18.0)
                .font_weight(FontWeight::SEMI_BOLD)
                .foreground(palette.accent)
                .horizontal_alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::Center),
        )
}

pub fn combo_box_resources(palette: &Palette) -> ResourceOverrides {
    ResourceOverrides::new()
        .set("ComboBoxBackground", palette.card_bg)
        .set("ComboBoxBackgroundPointerOver", palette.tile_bg)
        .set("ComboBoxBackgroundPressed", palette.tile_bg)
        .set("ComboBoxForeground", palette.text_primary)
        .set("ComboBoxForegroundPointerOver", palette.text_primary)
        .set("ComboBoxForegroundPressed", palette.text_primary)
        .set("ComboBoxBorderBrush", palette.stroke)
        .set("ComboBoxBorderBrushPointerOver", palette.accent)
        .set("ComboBoxBorderBrushPressed", palette.accent)
        .set("ComboBoxDropDownBackground", palette.card_bg)
        .set("ComboBoxDropDownBorderBrush", palette.stroke)
        .set("ComboBoxDropDownForeground", palette.text_primary)
        .set("ComboBoxItemBackground", palette.card_bg)
        .set("ComboBoxItemBackgroundPointerOver", palette.tile_bg)
        .set("ComboBoxItemBackgroundPressed", palette.app_bg)
        .set("ComboBoxItemBackgroundSelected", palette.accent)
        .set("ComboBoxItemBackgroundSelectedPointerOver", palette.accent)
        .set("ComboBoxItemBackgroundSelectedPressed", palette.accent)
        .set("ComboBoxItemForeground", palette.text_primary)
        .set("ComboBoxItemForegroundPointerOver", palette.text_primary)
        .set("ComboBoxItemForegroundPressed", palette.text_primary)
        .set("ComboBoxItemForegroundSelected", palette.accent_text)
        .set(
            "ComboBoxItemForegroundSelectedPointerOver",
            palette.accent_text,
        )
        .set("ComboBoxItemForegroundSelectedPressed", palette.accent_text)
}

fn fill_resources(background: Color, foreground: Color) -> ResourceOverrides {
    ResourceOverrides::new()
        .set("ButtonBackground", background)
        .set("ButtonBackgroundPointerOver", background)
        .set("ButtonBackgroundPressed", background)
        .set("ButtonForeground", foreground)
        .set("ButtonForegroundPointerOver", foreground)
        .set("ButtonForegroundPressed", foreground)
}

fn outline_resources(palette: &Palette) -> ResourceOverrides {
    ResourceOverrides::new()
        .set("ButtonBackground", palette.card_bg)
        .set("ButtonBackgroundPointerOver", palette.tile_bg)
        .set("ButtonBackgroundPressed", palette.app_bg)
        .set("ButtonForeground", palette.accent)
        .set("ButtonForegroundPointerOver", palette.accent)
        .set("ButtonForegroundPressed", palette.accent)
        .set("ButtonBorderBrush", palette.accent)
        .set("ButtonBorderBrushPointerOver", palette.accent)
        .set("ButtonBorderBrushPressed", palette.accent)
}
