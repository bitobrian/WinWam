#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::{Path, PathBuf},
};

use windows_reactor::*;

mod directory;
mod loadout;
mod logging;
mod persist;
mod scan;
mod theme;

use directory::{Addon, AddonSourceList, empty_source_list, load_source_list};
use loadout::{Loadout, LoadoutDraft, LoadoutPrompt};

#[cfg(debug_assertions)]
const LOCAL_TEST_ROOT: &str = "local-test";

#[derive(Clone, Copy)]
struct GameFlavor {
    label: &'static str,
    slug: &'static str,
    #[allow(dead_code)]
    schema_value: &'static str,
}

const GAME_FLAVORS: [GameFlavor; 5] = [
    GameFlavor {
        label: "Retail",
        slug: "retail",
        schema_value: "Retail",
    },
    GameFlavor {
        label: "Mists of Pandaria Classic",
        slug: "mop-classic",
        schema_value: "MoPClassic",
    },
    GameFlavor {
        label: "Classic",
        slug: "classic",
        schema_value: "Classic",
    },
    GameFlavor {
        label: "Burning Crusade Anniversary",
        slug: "bc-anniversary",
        schema_value: "BCAnniversary",
    },
    GameFlavor {
        label: "Forever",
        slug: "forever",
        schema_value: "Forever",
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Page {
    Discover,
    Installed,
    Loadouts,
    Workshop,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorkshopScreen {
    Overview,
    Anatomy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum UpdateStatus {
    Idle,
    Checking,
    Current,
    Failed(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct CatalogSurfaceState {
    query: String,
    category: Option<String>,
}

enum Message {
    Navigate(Page),
    Search(String),
    CheckForUpdates,
    DirectoryRefreshFinished(usize, Result<AddonSourceList, String>),
    ShowWorkshopScreen(WorkshopScreen),
    SelectFlavor(Option<usize>),
    SelectCategory(Option<usize>),
    OpenHttpsUrl(String),
    SourceUrlChanged(String),
    WowFolderChanged(String),
    BrowseWowFolder,
    ToggleUpdates(bool),
    ApplySettings,
    InstallAddon(String),
    InstallFinished(String, Result<(), String>),
    UninstallAddon(String),
    ToggleAddonDetails(String),
    CloseAddonDetails,
    DismissSupportBanner,
    RestoreSupportBanner,
    ShowSupportAuthors,
    CloseSupportAuthors,
    ToggleHood,
    SelectTelemetryLevel(Option<usize>),
    NewLoadout,
    EditLoadout(usize),
    RequestDeleteLoadout(usize),
    RequestApplyLoadout(usize),
    ConfirmLoadoutPrompt,
    DismissLoadoutPrompt,
    LoadoutNameChanged(String),
    ToggleLoadoutAddon(String, bool),
    SaveLoadout,
    CancelLoadout,
}

struct WinWam {
    source_list: AddonSourceList,
    page: Page,
    selected_flavor: usize,
    directory_error: Option<String>,
    directory_stale: bool,
    source_base_url: String,
    pending_source_base_url: String,
    wow_folder: String,
    wow_folder_missing: bool,
    check_for_updates: bool,
    installed_addon_ids: BTreeSet<String>,
    installed_addons: Vec<scan::InstalledAddon>,
    expanded_addon_id: Option<String>,
    installing_addon_ids: BTreeSet<String>,
    hood_open: bool,
    telemetry_level: usize,
    telemetry_events: VecDeque<String>,
    loadouts: Vec<Loadout>,
    loadout_draft: Option<LoadoutDraft>,
    loadout_prompt: Option<LoadoutPrompt>,
    last_pages: BTreeMap<String, String>,
    support_banner_dismissed: bool,
    support_authors_open: bool,
    toast: Option<String>,
    update_status: UpdateStatus,
    workshop_screen: WorkshopScreen,
    discover_surfaces: [CatalogSurfaceState; 5],
    installed_surfaces: [CatalogSurfaceState; 5],
}

impl Component for WinWam {
    type Input = ();
    type Message = Message;

    fn create(_input: &(), _context: &ComponentContext<Self>) -> Self {
        let mut settings = persist::load();
        let selected_flavor = flavor_index_from_slug(&settings.selected_flavor);
        settings.selected_flavor = GAME_FLAVORS[selected_flavor].slug.to_string();
        loadout::assign_missing_flavors(&mut settings.loadouts, &settings.selected_flavor);

        let (wow_folder, wow_folder_missing) = if !settings.wow_folder.is_empty() {
            let path = PathBuf::from(&settings.wow_folder);
            (
                settings.wow_folder.clone(),
                !is_wow_folder_for_flavor(&path, selected_flavor),
            )
        } else {
            match detect_wow_folder(selected_flavor) {
                Some(path) => (path.to_string_lossy().into_owned(), false),
                None => (String::new(), true),
            }
        };

        let source_base_url = settings.source_base_url.clone();
        let (mut source_list, mut directory_error, mut directory_stale) = adopt_directory_load(
            selected_flavor,
            &source_base_url,
            empty_source_list(selected_flavor),
        );
        if source_list.addons.is_empty()
            && let Some(error) = directory_error.as_ref()
            && let Some(development) = directory::development_source_list()
        {
            source_list = development;
            directory_error = Some(format!(
                "Could not refresh the directory: {error}. Showing development data."
            ));
            directory_stale = false;
        }
        let (installed_addons, installed_addon_ids) = scan_installed(
            Some(Path::new(wow_folder.trim())),
            selected_flavor,
            &source_list.addons,
        );
        let app = Self {
            source_list,
            page: initial_page(
                wow_folder_missing,
                settings
                    .last_pages
                    .get(GAME_FLAVORS[selected_flavor].slug)
                    .map(String::as_str),
            ),
            selected_flavor,
            directory_error,
            directory_stale,
            pending_source_base_url: source_base_url.clone(),
            source_base_url,
            wow_folder,
            wow_folder_missing,
            check_for_updates: settings.check_for_updates,
            installed_addon_ids,
            installed_addons,
            expanded_addon_id: None,
            installing_addon_ids: BTreeSet::new(),
            hood_open: false,
            telemetry_level: settings.telemetry_level.min(3),
            telemetry_events: VecDeque::from(["WinWam telemetry stream ready".to_string()]),
            loadouts: settings.loadouts,
            loadout_draft: None,
            loadout_prompt: None,
            last_pages: settings.last_pages,
            support_banner_dismissed: settings.support_banner_dismissed,
            support_authors_open: false,
            toast: None,
            update_status: UpdateStatus::Idle,
            workshop_screen: WorkshopScreen::Overview,
            discover_surfaces: std::array::from_fn(|_| CatalogSurfaceState::default()),
            installed_surfaces: std::array::from_fn(|_| CatalogSurfaceState::default()),
        };
        app.persist_settings();
        app
    }

    fn update(&mut self, message: Message, context: &ComponentContext<Self>) {
        if self.telemetry_level >= 2 || (self.telemetry_level == 1 && message_is_error(&message)) {
            self.record_telemetry(message_telemetry(&message));
        }
        match message {
            Message::ToggleHood => self.hood_open = !self.hood_open,
            Message::SelectTelemetryLevel(Some(level)) if level <= 3 => {
                self.telemetry_level = level;
                self.record_telemetry(format!("Telemetry level changed to {level}"));
                self.persist_settings();
            }
            Message::SelectTelemetryLevel(_) => {}
            Message::NewLoadout => {
                self.loadout_prompt = None;
                self.loadout_draft = Some(LoadoutDraft {
                    editing_index: None,
                    name: String::new(),
                    addon_ids: self.installed_addon_ids.clone(),
                });
            }
            Message::EditLoadout(index) => {
                if let Some(loadout) = self.loadouts.get(index) {
                    self.loadout_prompt = None;
                    self.loadout_draft = Some(LoadoutDraft {
                        editing_index: Some(index),
                        name: loadout.name.clone(),
                        addon_ids: loadout.addon_ids.clone(),
                    });
                }
            }
            Message::RequestDeleteLoadout(index) => {
                if let Some(loadout) = self.loadouts.get(index) {
                    self.loadout_draft = None;
                    self.loadout_prompt = Some(LoadoutPrompt::ConfirmDelete {
                        index,
                        name: loadout.name.clone(),
                    });
                }
            }
            Message::RequestApplyLoadout(index) => {
                if let Some(loadout) = self.loadouts.get(index).cloned() {
                    self.loadout_draft = None;
                    let removable = self.managed_addon_ids();
                    let plan = loadout::plan(&loadout, &self.installed_addon_ids, &removable);
                    self.loadout_prompt = Some(if plan.is_noop() {
                        LoadoutPrompt::Result(loadout::LoadoutApplyResult {
                            name: plan.name,
                            installed: Vec::new(),
                            uninstalled: Vec::new(),
                            failures: Vec::new(),
                        })
                    } else {
                        LoadoutPrompt::ConfirmApply(plan)
                    });
                }
            }
            Message::ConfirmLoadoutPrompt => self.confirm_loadout_prompt(),
            Message::DismissLoadoutPrompt => self.loadout_prompt = None,
            Message::LoadoutNameChanged(name) => {
                if let Some(draft) = &mut self.loadout_draft {
                    draft.name = name;
                }
            }
            Message::ToggleLoadoutAddon(id, enabled) => {
                if let Some(draft) = &mut self.loadout_draft {
                    if enabled {
                        draft.addon_ids.insert(id);
                    } else {
                        draft.addon_ids.remove(&id);
                    }
                }
            }
            Message::SaveLoadout => {
                if let Some(draft) = self.loadout_draft.take() {
                    let name = draft.name.trim();
                    if !name.is_empty() {
                        let flavor = draft
                            .editing_index
                            .and_then(|index| {
                                self.loadouts
                                    .get(index)
                                    .map(|loadout| loadout.flavor.clone())
                            })
                            .filter(|flavor| !flavor.is_empty())
                            .unwrap_or_else(|| self.current_flavor_slug().to_string());
                        let loadout = Loadout {
                            name: name.to_string(),
                            flavor,
                            addon_ids: draft.addon_ids,
                        };
                        if let Some(index) = draft.editing_index {
                            if index < self.loadouts.len() {
                                self.loadouts[index] = loadout;
                            }
                        } else {
                            self.loadouts.push(loadout);
                        }
                        self.persist_settings();
                    } else {
                        self.loadout_draft = Some(draft);
                    }
                }
            }
            Message::CancelLoadout => self.loadout_draft = None,
            Message::Navigate(page) => {
                if page == Page::Settings {
                    self.pending_source_base_url = self.source_base_url.clone();
                }
                self.page = page;
                self.remember_current_page();
            }
            Message::Search(query) => {
                if let Some(surface) = self.active_catalog_surface_mut() {
                    surface.query = query;
                }
            }
            Message::CheckForUpdates => {
                if self.wow_folder_missing || matches!(self.update_status, UpdateStatus::Checking) {
                    return;
                }
                self.update_status = UpdateStatus::Checking;
                let flavor = self.selected_flavor;
                let source_base_url = self.source_base_url.clone();
                context.spawn_background(move |_| {
                    let result = match load_source_list(flavor, &source_base_url) {
                        Ok(source_list) => Ok(source_list),
                        Err(error) => Err(error.to_string()),
                    };
                    Message::DirectoryRefreshFinished(flavor, result)
                });
            }
            Message::DirectoryRefreshFinished(flavor, result) => {
                if flavor != self.selected_flavor {
                    return;
                }
                let (source_list, directory_error, directory_stale) = adopt_directory_result(
                    result,
                    directory::load_cached_directory(GAME_FLAVORS[flavor].slug),
                    self.source_list.clone(),
                );
                self.source_list = source_list;
                self.directory_error = directory_error.clone();
                self.directory_stale = directory_stale;
                self.refresh_installed();
                self.update_status = match &directory_error {
                    None => UpdateStatus::Current,
                    Some(error) => UpdateStatus::Failed(error.clone()),
                };
                if let Some(id) = &self.expanded_addon_id
                    && self.lookup_overlay_addon(id).is_none()
                {
                    self.expanded_addon_id = None;
                }
            }
            Message::ShowWorkshopScreen(screen) => self.workshop_screen = screen,
            Message::SelectFlavor(Some(index)) if index < GAME_FLAVORS.len() => {
                if index == self.selected_flavor || !self.installing_addon_ids.is_empty() {
                    return;
                }
                self.remember_current_page();
                self.selected_flavor = index;
                self.loadout_draft = None;
                self.loadout_prompt = None;
                self.wow_folder_missing =
                    !is_wow_folder_for_flavor(Path::new(self.wow_folder.trim()), index);
                self.page = initial_page(
                    self.wow_folder_missing,
                    self.last_pages
                        .get(GAME_FLAVORS[index].slug)
                        .map(String::as_str),
                );
                let (source_list, directory_error, directory_stale) =
                    adopt_directory_load(index, &self.source_base_url, empty_source_list(index));
                self.source_list = source_list;
                self.directory_error = directory_error;
                self.directory_stale = directory_stale;
                if let Some(id) = &self.expanded_addon_id
                    && !self.source_list.addons.iter().any(|addon| addon.id == *id)
                {
                    self.expanded_addon_id = None;
                }
                self.refresh_installed();
                self.persist_settings();
            }
            Message::SelectFlavor(Some(_)) => {}
            Message::SelectFlavor(None) => {}
            Message::SelectCategory(Some(index)) => {
                let items = self.catalog_items();
                let query = self
                    .active_catalog_surface()
                    .map(|surface| surface.query.to_lowercase())
                    .unwrap_or_default();
                let searched: Vec<Addon> = items
                    .into_iter()
                    .filter(|addon| matches_query(addon, &query))
                    .collect();
                let categories = directory::visible_categories(&searched);
                if let Some(surface) = self.active_catalog_surface_mut() {
                    surface.category = categories.get(index).cloned();
                }
            }
            Message::SelectCategory(None) => {}
            Message::OpenHttpsUrl(url) => match directory::validated_https_url(&url) {
                Ok(url) => {
                    let host = url
                        .trim()
                        .strip_prefix("https://")
                        .and_then(|rest| rest.split(['/', '?', '#']).next())
                        .unwrap_or("site");
                    self.toast = Some(format!("Opening {host}"));
                    if let Err(error) = directory::open_https_url(&url) {
                        logging::error(&format!("Could not open {url}: {error}"));
                        self.toast = Some(format!("Could not open {host}"));
                    }
                }
                Err(error) => logging::error(&format!("Blocked URL {url}: {error}")),
            },
            Message::SourceUrlChanged(url) => self.pending_source_base_url = url,
            Message::WowFolderChanged(path) => {
                self.wow_folder_missing =
                    !is_wow_folder_for_flavor(Path::new(path.trim()), self.selected_flavor);
                self.wow_folder = path;
                self.refresh_installed();
            }
            Message::BrowseWowFolder => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Select the World of Warcraft folder")
                    .pick_folder()
                {
                    self.wow_folder_missing =
                        !is_wow_folder_for_flavor(&path, self.selected_flavor);
                    self.wow_folder = path.to_string_lossy().into_owned();
                    self.refresh_installed();
                    self.persist_settings();
                }
            }
            Message::ToggleUpdates(enabled) => {
                self.check_for_updates = enabled;
                self.persist_settings();
            }
            Message::ToggleAddonDetails(id) => {
                self.expanded_addon_id = if self.expanded_addon_id.as_deref() == Some(&id) {
                    None
                } else {
                    Some(id)
                };
                self.support_authors_open = false;
            }
            Message::CloseAddonDetails => self.expanded_addon_id = None,
            Message::DismissSupportBanner => {
                self.support_banner_dismissed = true;
                self.persist_settings();
            }
            Message::RestoreSupportBanner => {
                self.support_banner_dismissed = false;
                self.persist_settings();
            }
            Message::ShowSupportAuthors => {
                self.support_authors_open = true;
                self.expanded_addon_id = None;
            }
            Message::CloseSupportAuthors => self.support_authors_open = false,
            Message::InstallAddon(id) => {
                if self.installing_addon_ids.insert(id.clone()) {
                    let wow_folder = PathBuf::from(self.wow_folder.trim());
                    let flavor = self.selected_flavor;
                    context.spawn_background(move |_| {
                        #[cfg(debug_assertions)]
                        std::thread::sleep(std::time::Duration::from_millis(650));
                        let result = install_test_addon(&wow_folder, flavor, &id)
                            .map_err(|error| error.to_string());
                        Message::InstallFinished(id, result)
                    });
                }
            }
            Message::InstallFinished(id, result) => {
                self.installing_addon_ids.remove(&id);
                match result {
                    Ok(()) => {
                        self.refresh_installed();
                        self.directory_error = None;
                        self.directory_stale = false;
                    }
                    Err(error) => {
                        logging::error(&format!("Could not install addon: {error}"));
                        self.directory_error = Some(format!("Could not install addon: {error}"));
                        self.directory_stale = false;
                    }
                }
            }
            Message::UninstallAddon(id) => {
                match uninstall_test_addon(
                    Path::new(self.wow_folder.trim()),
                    self.selected_flavor,
                    &id,
                ) {
                    Ok(()) => {
                        self.refresh_installed();
                        self.directory_error = None;
                        self.directory_stale = false;
                    }
                    Err(error) => {
                        logging::error(&format!("Could not uninstall addon: {error}"));
                        self.directory_error = Some(format!("Could not uninstall addon: {error}"));
                        self.directory_stale = false;
                    }
                }
            }
            Message::ApplySettings => {
                let url = self.pending_source_base_url.trim().trim_end_matches('/');
                if url.starts_with("https://") {
                    self.source_base_url = url.to_string();
                    let (source_list, directory_error, directory_stale) = adopt_directory_load(
                        self.selected_flavor,
                        &self.source_base_url,
                        self.source_list.clone(),
                    );
                    self.source_list = source_list;
                    self.directory_error = directory_error;
                    self.directory_stale = directory_stale;
                    self.refresh_installed();
                    self.persist_settings();
                } else {
                    logging::error("Rejected a non-HTTPS addon directory source");
                    self.directory_error =
                        Some("The addon source must be an HTTPS URL.".to_string());
                    self.directory_stale = false;
                    self.persist_settings();
                }
            }
        }
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        let palette = theme::palette(self.selected_flavor);
        context.window_title("WinWam");
        context.window_visuals(
            WindowVisuals::new()
                .client_size(
                    theme::REFERENCE_CLIENT_WIDTH,
                    theme::REFERENCE_CLIENT_HEIGHT,
                )
                .constraints(WindowConstraints {
                    min_width: Some(theme::MIN_CLIENT_WIDTH),
                    min_height: Some(theme::MIN_CLIENT_HEIGHT),
                    max_width: None,
                    max_height: None,
                })
                .backdrop(WindowBackdrop::None)
                .theme(WindowTheme::Dark),
        );

        // Keep this node stable: remounting TitleBar hits a windows-reactor 0.100
        // ClearWindowTitleBar / PreferredHeightOption ERROR_INVALID_STATE crash.
        let title_bar = TitleBar::new()
            .title("WinWam")
            .is_back_button_visible(false)
            .is_pane_toggle_button_visible(false)
            .slots([SlotView::new(
                TitleBarSlot::Content,
                Grid::new()
                    .columns([GridLength::Pixel(theme::GAME_PANEL_WIDTH), GridLength::STAR])
                    .children((
                        StackPanel::new()
                            .orientation(Orientation::Horizontal)
                            .spacing(13.0)
                            .vertical_alignment(VerticalAlignment::Center)
                            .margin(Thickness::new(16.0, 0.0, 0.0, 0.0))
                            .children((
                                Image::new()
                                    .source_data(EncodedImage::from_static(include_bytes!(
                                        "../assets/icon.png"
                                    )))
                                    .width(44.0)
                                    .height(44.0)
                                    .stretch(Stretch::Uniform),
                                TextBlock::new()
                                    .text("WinWAM")
                                    .font_size(theme::BRAND_SIZE)
                                    .font_weight(FontWeight::BOLD)
                                    .foreground(palette.text_primary)
                                    .vertical_alignment(VerticalAlignment::Center),
                            )),
                        StackPanel::new()
                            .grid_column(1)
                            .orientation(Orientation::Horizontal)
                            .spacing(32.0)
                            .margin(Thickness::new(24.0, 0.0, 0.0, 0.0))
                            .children((
                                theme::topnav_button(
                                    palette,
                                    "DISCOVER",
                                    self.page == Page::Discover,
                                )
                                .automation_name("Discover")
                                .on_click(context.callback(|_| Message::Navigate(Page::Discover))),
                                theme::topnav_button(
                                    palette,
                                    "MY ADDONS",
                                    self.page == Page::Installed,
                                )
                                .automation_name("My Addons")
                                .on_click(context.callback(|_| Message::Navigate(Page::Installed))),
                                theme::topnav_button(
                                    palette,
                                    "LOADOUTS",
                                    self.page == Page::Loadouts,
                                )
                                .automation_name("Loadouts")
                                .on_click(context.callback(|_| Message::Navigate(Page::Loadouts))),
                                theme::topnav_button(
                                    palette,
                                    "ADDON WORKSHOP",
                                    self.page == Page::Workshop,
                                )
                                .automation_name("Addon Workshop")
                                .on_click(context.callback(|_| Message::Navigate(Page::Workshop))),
                            )),
                    )),
            )]);

        let content = match self.page {
            Page::Discover => self.browse_view(context),
            Page::Installed => self.installed_view(context),
            Page::Loadouts => self.loadouts_view(context),
            Page::Workshop => self.workshop_view(context),
            Page::Settings => self.settings_view(context),
        };

        let page = Border::new()
            .background(palette.app_bg)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .vertical_alignment(VerticalAlignment::Stretch)
            .content(content);

        let hood: View = if self.hood_open {
            Border::new()
                .width(380.0)
                .horizontal_alignment(HorizontalAlignment::Right)
                .vertical_alignment(VerticalAlignment::Stretch)
                .content(self.hood_view())
        } else {
            Border::new().width(0.0).into()
        };

        let overlay = self.shell_overlay(context, palette);
        let content_column = Grid::new()
            .grid_column(1)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .vertical_alignment(VerticalAlignment::Stretch)
            .children((page, hood, overlay));

        Grid::new()
            .rows([GridLength::Auto, GridLength::STAR])
            .background(palette.app_bg)
            .keyed_children([
                KeyedView::new(
                    "title-bar",
                    Border::new()
                        .grid_row(0)
                        .background(palette.app_bg)
                        .content(title_bar),
                ),
                KeyedView::new(
                    format!("body-theme-{}", self.selected_flavor),
                    Grid::new()
                        .grid_row(1)
                        .columns([GridLength::Pixel(theme::GAME_PANEL_WIDTH), GridLength::STAR])
                        .children((self.game_panel_view(context), content_column)),
                ),
            ])
    }
}

impl WinWam {
    fn current_flavor_slug(&self) -> &'static str {
        GAME_FLAVORS[self.selected_flavor].slug
    }

    fn remember_current_page(&mut self) {
        if let Some(page) = page_slug(self.page) {
            self.last_pages
                .insert(self.current_flavor_slug().to_string(), page.to_string());
            self.persist_settings();
        }
    }

    fn active_catalog_surface_mut(&mut self) -> Option<&mut CatalogSurfaceState> {
        active_catalog_surface_mut(
            self.page,
            self.selected_flavor,
            &mut self.discover_surfaces,
            &mut self.installed_surfaces,
        )
    }

    fn update_status_text(&self) -> String {
        match &self.update_status {
            UpdateStatus::Idle => String::new(),
            UpdateStatus::Checking => "Checking…".to_string(),
            UpdateStatus::Current => "Last checked: just now".to_string(),
            UpdateStatus::Failed(_) => "Could not check for updates".to_string(),
        }
    }

    fn game_panel_view(&self, context: &ViewContext<Self>) -> View {
        let palette = theme::palette(self.selected_flavor);
        let checking = matches!(self.update_status, UpdateStatus::Checking);
        let update_label = if checking {
            "Checking…"
        } else {
            "Check for Updates"
        };
        let update_enabled = !self.wow_folder_missing && !checking;
        let availability: View = if self.directory_error.is_none() {
            TextBlock::new()
                .text(format!(
                    "{} addons available.",
                    self.source_list.addons.len()
                ))
                .font_size(theme::META_SIZE)
                .foreground(palette.status_ok)
                .into()
        } else {
            Border::new().height(0.0).into()
        };
        let accent_glow = Color::argb(0x2E, palette.accent.r, palette.accent.g, palette.accent.b);
        Border::new()
            .grid_column(0)
            .background(palette.sidebar_bg)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .vertical_alignment(VerticalAlignment::Stretch)
            .content(
                Grid::new()
                    .rows([GridLength::STAR, GridLength::Auto])
                    .children((
                        Grid::new().grid_row(0).children((
                            Border::new()
                                .background(palette.app_bg)
                                .horizontal_alignment(HorizontalAlignment::Stretch)
                                .vertical_alignment(VerticalAlignment::Stretch),
                            Border::new()
                                .width(280.0)
                                .height(280.0)
                                .corner_radius(140.0)
                                .background(accent_glow)
                                .horizontal_alignment(HorizontalAlignment::Center)
                                .vertical_alignment(VerticalAlignment::Top)
                                .margin(Thickness::new(0.0, 24.0, 0.0, 0.0)),
                            Border::new()
                                .height(180.0)
                                .background(palette.sidebar_bg)
                                .vertical_alignment(VerticalAlignment::Bottom)
                                .horizontal_alignment(HorizontalAlignment::Stretch),
                            theme::sku_flair(self.selected_flavor),
                        )),
                        StackPanel::new()
                            .grid_row(1)
                            .spacing(8.0)
                            .margin(Thickness::new(26.0, 18.0, 26.0, 26.0))
                            .children((
                                TextBlock::new()
                                    .text("GAME VERSION")
                                    .font_size(theme::META_SIZE)
                                    .font_weight(FontWeight::EXTRA_BOLD)
                                    .foreground(palette.text_muted),
                                Button::new()
                                    .style(ButtonStyle::Subtle)
                                    .resource_overrides(theme::combo_box_resources(palette))
                                    .horizontal_alignment(HorizontalAlignment::Stretch)
                                    .content(
                                        ComboBox::new()
                                            .horizontal_alignment(HorizontalAlignment::Stretch)
                                            .height(theme::CONTROL_HEIGHT)
                                            .items_source(GAME_FLAVORS.map(|flavor| flavor.label))
                                            .selected_index(self.selected_flavor)
                                            .is_enabled(self.installing_addon_ids.is_empty())
                                            .on_selection_changed(
                                                context.callback(Message::SelectFlavor),
                                            ),
                                    ),
                                Grid::new()
                                    .columns([
                                        GridLength::STAR,
                                        GridLength::Pixel(theme::SETTINGS_BUTTON_SIZE),
                                    ])
                                    .column_spacing(8.0)
                                    .children((
                                        theme::accent_button(palette, update_label)
                                            .height(theme::CONTROL_HEIGHT)
                                            .horizontal_alignment(HorizontalAlignment::Stretch)
                                            .enabled(update_enabled)
                                            .on_click(
                                                context.callback(|_| Message::CheckForUpdates),
                                            ),
                                        theme::icon_outline_button(palette, Symbol::Setting)
                                            .grid_column(1)
                                            .automation_name("Settings")
                                            .on_click(
                                                context.callback(|_| {
                                                    Message::Navigate(Page::Settings)
                                                }),
                                            ),
                                    )),
                                TextBlock::new()
                                    .text(self.update_status_text())
                                    .font_size(theme::META_SIZE)
                                    .foreground(palette.text_muted),
                                availability,
                            )),
                    )),
            )
    }

    fn workshop_view(&self, context: &ViewContext<Self>) -> View {
        let palette = theme::palette(self.selected_flavor);
        let lesson: View = match self.workshop_screen {
            WorkshopScreen::Overview => StackPanel::new()
                .spacing(12.0)
                .children((
                    TextBlock::new()
                        .text("START BUILDING")
                        .font_size(theme::EYEBROW_SIZE)
                        .font_weight(FontWeight::EXTRA_BOLD)
                        .foreground(palette.accent),
                    TextBlock::new()
                        .text("Create your first addon.")
                        .font_size(36.0)
                        .font_weight(FontWeight::BOLD)
                        .foreground(palette.text_primary)
                        .text_wrapping(TextWrapping::Wrap),
                    TextBlock::new()
                        .text("Turn an idea into a working World of Warcraft addon. Learn what each file does, build useful features one step at a time, and test every change in-game.")
                        .font_size(15.0)
                        .text_wrapping(TextWrapping::Wrap)
                        .foreground(palette.text_muted),
                    theme::accent_button(palette, "Start learning")
                        .height(theme::CONTROL_HEIGHT)
                        .on_click(context.callback(|_| {
                            Message::ShowWorkshopScreen(WorkshopScreen::Anatomy)
                        })),
                )),
            WorkshopScreen::Anatomy => StackPanel::new()
                .spacing(12.0)
                .children((
                    TextBlock::new()
                        .text("Anatomy lesson arrives in a later milestone.")
                        .font_size(15.0)
                        .text_wrapping(TextWrapping::Wrap)
                        .foreground(palette.text_primary),
                    theme::outline_button(palette, "Back to overview")
                        .on_click(context.callback(|_| {
                            Message::ShowWorkshopScreen(WorkshopScreen::Overview)
                        })),
                )),
        };
        ScrollViewer::new()
            .horizontal_scroll_bar_visibility(ScrollBarVisibility::Disabled)
            .content(
                StackPanel::new()
                    .spacing(16.0)
                    .margin(Thickness::uniform(theme::PAGE_MARGIN))
                    .children((
                        lesson,
                        Border::new()
                            .background(palette.card_bg)
                            .border_brush(palette.stroke)
                            .border_thickness(1.0)
                            .corner_radius(theme::CARD_RADIUS)
                            .padding(Thickness::xy(13.0, 9.0))
                            .content(
                                StackPanel::new()
                                    .orientation(Orientation::Horizontal)
                                    .spacing(12.0)
                                    .children((
                                        TextBlock::new()
                                            .text("Using AI assistance")
                                            .font_weight(FontWeight::SEMI_BOLD)
                                            .foreground(palette.accent),
                                        TextBlock::new()
                                            .text("Share the target game version, relevant API documentation, and exact errors. Ask for small explained changes, review the code, and test each step in-game. Never share account or personal information.")
                                            .font_size(11.0)
                                            .text_wrapping(TextWrapping::Wrap)
                                            .foreground(palette.text_muted),
                                    )),
                            ),
                    )),
            )
    }

    fn managed_addon_ids(&self) -> BTreeSet<String> {
        self.installed_addons
            .iter()
            .filter(|addon| addon.managed)
            .map(|addon| addon.id.clone())
            .collect()
    }

    fn persist_settings(&self) {
        let settings = persist::AppSettings {
            schema_version: persist::SETTINGS_SCHEMA_VERSION,
            source_base_url: self.source_base_url.clone(),
            wow_folder: self.wow_folder.clone(),
            selected_flavor: self.current_flavor_slug().to_string(),
            check_for_updates: self.check_for_updates,
            telemetry_level: self.telemetry_level,
            loadouts: self.loadouts.clone(),
            last_pages: self.last_pages.clone(),
            support_banner_dismissed: self.support_banner_dismissed,
        };
        if let Err(error) = persist::save(&settings) {
            logging::error(&format!("Could not save settings: {error}"));
        }
    }

    fn refresh_installed(&mut self) {
        let (installed_addons, installed_addon_ids) = scan_installed(
            Some(Path::new(self.wow_folder.trim())),
            self.selected_flavor,
            &self.source_list.addons,
        );
        self.installed_addons = installed_addons;
        self.installed_addon_ids = installed_addon_ids;
    }

    fn addon_display_name(&self, id: &str) -> String {
        self.source_list
            .addons
            .iter()
            .find(|addon| addon.id == id)
            .map(|addon| addon.name.clone())
            .or_else(|| {
                self.installed_addons
                    .iter()
                    .find(|addon| addon.id == id)
                    .map(|addon| addon.title.clone())
            })
            .unwrap_or_else(|| id.to_string())
    }

    fn format_addon_names(&self, ids: &[String]) -> String {
        if ids.is_empty() {
            return "None".to_string();
        }
        ids.iter()
            .map(|id| self.addon_display_name(id))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn confirm_loadout_prompt(&mut self) {
        match self.loadout_prompt.take() {
            Some(LoadoutPrompt::ConfirmDelete { index, name }) => {
                if index < self.loadouts.len() {
                    self.loadouts.remove(index);
                    self.persist_settings();
                    self.record_telemetry(format!("Loadout deleted · {name}"));
                }
            }
            Some(LoadoutPrompt::ConfirmApply(plan)) => {
                let result = self.apply_loadout_plan(plan);
                self.record_telemetry(format!(
                    "Loadout applied · {} · {} installed · {} removed · {} failed",
                    result.name,
                    result.installed.len(),
                    result.uninstalled.len(),
                    result.failures.len()
                ));
                self.loadout_prompt = Some(LoadoutPrompt::Result(result));
            }
            Some(LoadoutPrompt::Result(_)) | None => {}
        }
    }

    fn apply_loadout_plan(&mut self, plan: loadout::LoadoutPlan) -> loadout::LoadoutApplyResult {
        let wow_folder = Path::new(self.wow_folder.trim());
        let mut installed = Vec::new();
        let mut uninstalled = Vec::new();
        let mut failures = Vec::new();
        for id in plan.uninstall {
            match uninstall_test_addon(wow_folder, self.selected_flavor, &id) {
                Ok(()) => uninstalled.push(id),
                Err(error) => failures.push(loadout::LoadoutFailure {
                    id,
                    action: "uninstall",
                    error: error.to_string(),
                }),
            }
        }
        for id in plan.install {
            match install_test_addon(wow_folder, self.selected_flavor, &id) {
                Ok(()) => installed.push(id),
                Err(error) => failures.push(loadout::LoadoutFailure {
                    id,
                    action: "install",
                    error: error.to_string(),
                }),
            }
        }
        self.refresh_installed();
        loadout::LoadoutApplyResult {
            name: plan.name,
            installed,
            uninstalled,
            failures,
        }
    }

    fn record_telemetry(&mut self, event: impl Into<String>) {
        if self.telemetry_level == 0 {
            return;
        }
        self.telemetry_events.push_back(event.into());
        while self.telemetry_events.len() > 250 {
            self.telemetry_events.pop_front();
        }
    }

    fn hood_view(&self) -> View {
        let palette = theme::palette(self.selected_flavor);
        let entries = self
            .telemetry_events
            .iter()
            .rev()
            .enumerate()
            .map(|(index, event)| {
                KeyedView::new(
                    format!("event-{index}"),
                    TextBlock::new()
                        .text(event.clone())
                        .font_size(12.0)
                        .text_wrapping(TextWrapping::Wrap)
                        .foreground(palette.text_primary),
                )
            })
            .collect::<Vec<_>>();
        Border::new()
            .background(palette.sidebar_bg)
            .border_brush(palette.stroke)
            .border_thickness(Thickness::new(1.0, 0.0, 0.0, 0.0))
            .padding(Thickness::uniform(16.0))
            .content(
                Grid::new()
                    .rows([GridLength::Auto, GridLength::STAR])
                    .children((
                        StackPanel::new().spacing(4.0).children((
                            TextBlock::new()
                                .text("Under the Hood")
                                .font_size(18.0)
                                .font_weight(FontWeight::SEMI_BOLD)
                                .foreground(palette.text_primary),
                            TextBlock::new()
                                .text("Live in-app telemetry")
                                .font_size(12.0)
                                .foreground(palette.text_muted),
                        )),
                        ScrollViewer::new()
                            .grid_row(1)
                            .horizontal_scroll_bar_visibility(ScrollBarVisibility::Disabled)
                            .content(
                                StackPanel::new()
                                    .spacing(8.0)
                                    .margin(Thickness::new(0.0, 12.0, 0.0, 0.0))
                                    .keyed_children(entries),
                            ),
                    )),
            )
    }

    fn catalog_items(&self) -> Vec<Addon> {
        match self.page {
            Page::Installed => self
                .installed_addons
                .iter()
                .map(|installed| {
                    self.source_list
                        .addons
                        .iter()
                        .find(|addon| addon.id == installed.id)
                        .cloned()
                        .unwrap_or_else(|| Addon {
                            id: installed.id.clone(),
                            name: installed.title.clone(),
                            author: String::new(),
                            category: String::new(),
                            ..Addon::default()
                        })
                })
                .collect(),
            _ => self.source_list.addons.clone(),
        }
    }

    fn active_catalog_surface(&self) -> Option<&CatalogSurfaceState> {
        match self.page {
            Page::Discover => self.discover_surfaces.get(self.selected_flavor),
            Page::Installed => self.installed_surfaces.get(self.selected_flavor),
            _ => None,
        }
    }

    fn catalog_surface_view(&self, context: &ViewContext<Self>, discover: bool) -> View {
        let palette = theme::palette(self.selected_flavor);
        let surface = if discover {
            &self.discover_surfaces[self.selected_flavor]
        } else {
            &self.installed_surfaces[self.selected_flavor]
        };
        let items = if discover {
            self.source_list.addons.clone()
        } else {
            self.installed_addons
                .iter()
                .map(|installed| {
                    self.source_list
                        .addons
                        .iter()
                        .find(|addon| addon.id == installed.id)
                        .cloned()
                        .unwrap_or_else(|| Addon {
                            id: installed.id.clone(),
                            name: installed.title.clone(),
                            author: String::new(),
                            category: String::new(),
                            ..Addon::default()
                        })
                })
                .collect()
        };
        let query = surface.query.to_lowercase();
        let searched: Vec<Addon> = items
            .iter()
            .filter(|addon| matches_query(addon, &query))
            .cloned()
            .collect();
        let categories = directory::visible_categories(&searched);
        let selected_category = surface
            .category
            .as_ref()
            .filter(|category| categories.iter().any(|item| item == *category))
            .cloned()
            .or_else(|| categories.first().cloned());
        let addons: Vec<Addon> = searched
            .into_iter()
            .filter(|addon| match selected_category.as_ref() {
                Some(category) if !addon.category.is_empty() => addon.category == *category,
                _ => true,
            })
            .collect();

        let mut children = vec![KeyedView::new(
            "header",
            theme::catalog_header(
                palette,
                if discover {
                    "COMMUNITY PICKS"
                } else {
                    "YOUR COLLECTION"
                },
                if discover {
                    "Featured Addons"
                } else {
                    "My Addons"
                },
            ),
        )];
        if let Some(error) = &self.directory_error {
            children.push(KeyedView::new(
                "error",
                InfoBar::new()
                    .is_open(true)
                    .is_closable(false)
                    .severity(if self.directory_stale {
                        InfoBarSeverity::Warning
                    } else {
                        InfoBarSeverity::Error
                    })
                    .title(if self.directory_stale {
                        "Saved directory"
                    } else {
                        "Directory unavailable"
                    })
                    .message(error.clone()),
            ));
        }
        children.push(KeyedView::new(
            "search",
            TextBox::new()
                .text(surface.query.clone())
                .placeholder_text("Search addons")
                .on_text_changed(context.callback(Message::Search)),
        ));
        if !categories.is_empty() {
            let selected = selected_category.clone();
            let ribbon = categories
                .iter()
                .enumerate()
                .map(|(index, category)| {
                    KeyedView::new(
                        category.clone(),
                        theme::ribbon_button(
                            palette,
                            directory::category_label(category),
                            category_ribbon_icon(category),
                            selected.as_deref() == Some(category.as_str()),
                        )
                        .on_click(context.callback(move |_| Message::SelectCategory(Some(index)))),
                    )
                })
                .collect::<Vec<_>>();
            children.push(KeyedView::new(
                "ribbon",
                ScrollViewer::new()
                    .horizontal_scroll_bar_visibility(ScrollBarVisibility::Auto)
                    .vertical_scroll_bar_visibility(ScrollBarVisibility::Disabled)
                    .content(
                        StackPanel::new()
                            .orientation(Orientation::Horizontal)
                            .spacing(8.0)
                            .keyed_children(ribbon),
                    ),
            ));
        }
        if !self.support_banner_dismissed {
            children.push(KeyedView::new(
                "support-banner",
                self.support_banner_view(palette, context),
            ));
        }

        let empty_catalog = items.is_empty();
        let loading = matches!(self.update_status, UpdateStatus::Checking) && empty_catalog;
        if loading {
            children.push(KeyedView::new(
                "loading",
                StackPanel::new()
                    .spacing(12.0)
                    .horizontal_alignment(HorizontalAlignment::Center)
                    .children((
                        ProgressRing::new()
                            .width(32.0)
                            .height(32.0)
                            .is_indeterminate(true)
                            .is_active(true),
                        TextBlock::new()
                            .text("Loading addons")
                            .foreground(palette.text_muted),
                    )),
            ));
        } else if empty_catalog {
            let (title, message) = if let Some(error) = &self.directory_error {
                ("Could not load addons", error.as_str())
            } else {
                (
                    "No addons yet",
                    if discover {
                        "The directory for this game version is empty."
                    } else {
                        "Install an addon from Discover to start your collection."
                    },
                )
            };
            children.push(KeyedView::new(
                "empty",
                theme::empty_state(palette, title, message),
            ));
        } else if addons.is_empty() {
            children.push(KeyedView::new(
                "empty",
                theme::empty_state(
                    palette,
                    "No addons found in this view.",
                    "Try another search or category.",
                ),
            ));
        } else {
            let cards = addons
                .iter()
                .map(|addon| {
                    let installed = self.installed_addon_ids.contains(&addon.id);
                    let managed = self
                        .installed_addons
                        .iter()
                        .find(|item| item.id == addon.id)
                        .map(|item| item.managed)
                        .unwrap_or(false);
                    KeyedView::new(
                        addon.id.clone(),
                        addon_card(
                            addon,
                            palette,
                            context,
                            installed,
                            managed,
                            self.expanded_addon_id.as_deref() == Some(addon.id.as_str()),
                            self.installing_addon_ids.contains(&addon.id),
                        ),
                    )
                })
                .collect::<Vec<_>>();
            children.push(KeyedView::new(
                "results",
                VariableSizedWrapGrid::new()
                    .orientation(Orientation::Horizontal)
                    .item_width(theme::catalog_card_width())
                    .item_height(theme::CATALOG_CARD_HEIGHT)
                    .horizontal_alignment(HorizontalAlignment::Left)
                    .keyed_children(cards),
            ));
        }

        ScrollViewer::new()
            .horizontal_scroll_bar_visibility(ScrollBarVisibility::Disabled)
            .content(
                StackPanel::new()
                    .spacing(16.0)
                    .margin(Thickness::uniform(theme::PAGE_MARGIN))
                    .keyed_children(children),
            )
    }

    fn support_banner_view(&self, palette: &theme::Palette, context: &ViewContext<Self>) -> View {
        Border::new()
            .background(palette.card_bg)
            .border_brush(palette.stroke)
            .border_thickness(1.0)
            .corner_radius(theme::CARD_RADIUS)
            .padding(Thickness::xy(16.0, 12.0))
            .content(
                Grid::new()
                    .columns([
                        GridLength::Auto,
                        GridLength::STAR,
                        GridLength::Auto,
                        GridLength::Auto,
                    ])
                    .column_spacing(12.0)
                    .children((
                        SymbolIcon::new().symbol(Symbol::Favorite),
                        StackPanel::new().grid_column(1).spacing(2.0).children((
                            TextBlock::new()
                                .text("DID YOU KNOW?")
                                .font_size(theme::EYEBROW_SIZE)
                                .font_weight(FontWeight::EXTRA_BOLD)
                                .foreground(palette.accent),
                            TextBlock::new()
                                .text("Enjoy an addon? Consider supporting its author.")
                                .font_weight(FontWeight::SEMI_BOLD)
                                .foreground(palette.text_primary)
                                .text_wrapping(TextWrapping::Wrap),
                            TextBlock::new()
                                .text("Many authors accept donations through GitHub Sponsors, Ko-fi, Patreon, or their project page. A contribution, star, or kind note all help.")
                                .font_size(theme::META_SIZE)
                                .foreground(palette.text_muted)
                                .text_wrapping(TextWrapping::Wrap),
                        )),
                        theme::accent_button(palette, "Support authors")
                            .grid_column(2)
                            .vertical_alignment(VerticalAlignment::Center)
                            .on_click(context.callback(|_| Message::ShowSupportAuthors)),
                        theme::outline_button(palette, "Dismiss")
                            .grid_column(3)
                            .vertical_alignment(VerticalAlignment::Center)
                            .automation_name("Dismiss author-support message")
                            .on_click(context.callback(|_| Message::DismissSupportBanner)),
                    )),
            )
    }

    fn shell_overlay(&self, context: &ViewContext<Self>, palette: &theme::Palette) -> View {
        let dialog: Option<View> = if self.support_authors_open {
            Some(self.support_authors_overlay(palette, context))
        } else if let Some(id) = self.expanded_addon_id.as_deref() {
            self.lookup_overlay_addon(id)
                .map(|addon| self.addon_details_overlay(&addon, palette, context))
        } else {
            None
        };
        let toast: View = match &self.toast {
            Some(message) => Border::new()
                .horizontal_alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::Bottom)
                .margin(Thickness::new(0.0, 0.0, 0.0, 24.0))
                .background(palette.card_bg)
                .border_brush(palette.stroke)
                .border_thickness(1.0)
                .corner_radius(theme::CARD_RADIUS)
                .padding(Thickness::xy(16.0, 10.0))
                .content(
                    TextBlock::new()
                        .text(message.clone())
                        .foreground(palette.text_primary),
                ),
            None => Border::new().height(0.0).into(),
        };
        match dialog {
            Some(dialog) => Grid::new().children((
                Border::new()
                    .background(palette.overlay)
                    .horizontal_alignment(HorizontalAlignment::Stretch)
                    .vertical_alignment(VerticalAlignment::Stretch),
                Border::new()
                    .width(theme::MODAL_WIDTH)
                    .horizontal_alignment(HorizontalAlignment::Center)
                    .vertical_alignment(VerticalAlignment::Center)
                    .content(dialog),
                toast,
            )),
            None => toast,
        }
    }

    fn lookup_overlay_addon(&self, id: &str) -> Option<Addon> {
        self.source_list
            .addons
            .iter()
            .find(|addon| addon.id == id)
            .cloned()
            .or_else(|| {
                self.installed_addons
                    .iter()
                    .find(|item| item.id == id)
                    .map(|installed| Addon {
                        id: installed.id.clone(),
                        name: installed.title.clone(),
                        author: String::new(),
                        category: String::new(),
                        version: installed.version.clone(),
                        ..Addon::default()
                    })
            })
    }

    fn addon_details_overlay(
        &self,
        addon: &Addon,
        palette: &theme::Palette,
        context: &ViewContext<Self>,
    ) -> View {
        let installed = self.installed_addon_ids.contains(&addon.id);
        let managed = self
            .installed_addons
            .iter()
            .find(|item| item.id == addon.id)
            .map(|item| item.managed)
            .unwrap_or(false);
        let installing = self.installing_addon_ids.contains(&addon.id);
        let status = if installing {
            "Installing"
        } else if installed && managed {
            "Installed"
        } else if installed {
            "Unmanaged"
        } else {
            "Not installed"
        };
        let version = addon
            .latest_release
            .as_ref()
            .map(|release| release.version.as_str())
            .or(addon.version.as_deref());
        let updated = addon
            .latest_release
            .as_ref()
            .and_then(|release| parse_published_at(&release.published_at))
            .map(format_age);
        let source = directory::resolved_source_url(addon);
        let support = addon.support_urls.first().map(|item| item.url.clone());
        let mut sidebar = vec![KeyedView::new("status", theme::stat_chip(palette, status))];
        if let Some(version) = version {
            sidebar.push(KeyedView::new(
                "version",
                meta_row(palette, "Version", version),
            ));
        }
        sidebar.push(KeyedView::new(
            "sku",
            meta_row(
                palette,
                "Game version",
                GAME_FLAVORS[self.selected_flavor].label,
            ),
        ));
        if let Some(count) = addon.download_count {
            sidebar.push(KeyedView::new(
                "downloads",
                meta_row(palette, "Downloads", &directory::compact_count(count)),
            ));
        }
        if let Some(updated) = updated.as_deref() {
            sidebar.push(KeyedView::new(
                "updated",
                meta_row(palette, "Updated", updated),
            ));
        }
        let action: View = if installing {
            ProgressRing::new()
                .width(28.0)
                .height(28.0)
                .is_indeterminate(true)
                .is_active(true)
                .into()
        } else if !installed {
            theme::accent_button(palette, "Install")
                .automation_name(format!("Install {}", addon.name))
                .on_click({
                    let id = addon.id.clone();
                    context.callback(move |_| Message::InstallAddon(id.clone()))
                })
                .into()
        } else {
            theme::outline_button(palette, "Remove")
                .enabled(managed)
                .automation_name(if managed {
                    format!("Remove {}", addon.name)
                } else {
                    "WinWAM did not install this addon, so it will not remove it.".to_string()
                })
                .on_click({
                    let id = addon.id.clone();
                    context.callback(move |_| Message::UninstallAddon(id.clone()))
                })
                .into()
        };
        sidebar.push(KeyedView::new("action", action));
        if let Some(url) = source {
            sidebar.push(KeyedView::new(
                "source",
                theme::outline_button(palette, "View source project")
                    .automation_name(format!("Open {} source project", addon.name))
                    .on_click(context.callback(move |_| Message::OpenHttpsUrl(url.clone()))),
            ));
        }
        if let Some(url) = support {
            sidebar.push(KeyedView::new(
                "support",
                theme::outline_button(palette, "Support this author")
                    .automation_name(format!("Support {}", addon.author))
                    .on_click(context.callback(move |_| Message::OpenHttpsUrl(url.clone()))),
            ));
        }
        if let Some(error) = &self.directory_error {
            sidebar.push(KeyedView::new(
                "error",
                InfoBar::new()
                    .is_open(true)
                    .is_closable(false)
                    .severity(InfoBarSeverity::Error)
                    .title("Action failed")
                    .message(error.clone()),
            ));
        }
        let mut main_children: Vec<View> = vec![
            TextBlock::new()
                .text("About this addon")
                .font_size(theme::SECTION_TITLE_SIZE)
                .font_weight(FontWeight::SEMI_BOLD)
                .foreground(palette.text_primary)
                .into(),
            TextBlock::new()
                .text(
                    addon
                        .description
                        .clone()
                        .unwrap_or_else(|| addon.summary.clone()),
                )
                .text_wrapping(TextWrapping::Wrap)
                .foreground(palette.text_primary)
                .into(),
        ];
        if !addon.features.is_empty() {
            main_children.push(
                TextBlock::new()
                    .text("What it adds")
                    .font_size(theme::SECTION_TITLE_SIZE)
                    .font_weight(FontWeight::SEMI_BOLD)
                    .foreground(palette.text_primary)
                    .into(),
            );
            for feature in &addon.features {
                main_children.push(
                    TextBlock::new()
                        .text(format!("• {feature}"))
                        .text_wrapping(TextWrapping::Wrap)
                        .foreground(palette.text_muted)
                        .into(),
                );
            }
        }
        Border::new()
            .background(palette.card_bg)
            .border_brush(palette.stroke)
            .border_thickness(1.0)
            .corner_radius(theme::CARD_RADIUS)
            .padding(Thickness::uniform(20.0))
            .content(
                StackPanel::new().spacing(16.0).children((
                    Grid::new()
                        .columns([GridLength::STAR, GridLength::Auto])
                        .children((
                            Grid::new()
                                .columns([GridLength::Auto, GridLength::STAR])
                                .column_spacing(12.0)
                                .children((
                                    theme::addon_icon_tile_at(palette, &addon.name, 58.0),
                                    StackPanel::new().grid_column(1).spacing(4.0).children((
                                        theme::category_chip(
                                            palette,
                                            directory::category_label(&addon.category),
                                        ),
                                        TextBlock::new()
                                            .text(addon.name.clone())
                                            .font_size(theme::ROW_NAME_SIZE)
                                            .font_weight(FontWeight::BOLD)
                                            .foreground(palette.text_primary),
                                        TextBlock::new()
                                            .text(format!("by {}", addon.author))
                                            .foreground(palette.text_muted),
                                    )),
                                )),
                            theme::outline_button(palette, "Close")
                                .grid_column(1)
                                .automation_name("Close")
                                .on_click(context.callback(|_| Message::CloseAddonDetails)),
                        )),
                    Grid::new()
                        .columns([
                            GridLength::STAR,
                            GridLength::Pixel(theme::MODAL_SIDEBAR_WIDTH),
                        ])
                        .column_spacing(20.0)
                        .children((
                            StackPanel::new().spacing(10.0).keyed_children(
                                main_children
                                    .into_iter()
                                    .enumerate()
                                    .map(|(index, child)| {
                                        KeyedView::new(format!("main-{index}"), child)
                                    })
                                    .collect::<Vec<_>>(),
                            ),
                            StackPanel::new()
                                .grid_column(1)
                                .spacing(10.0)
                                .keyed_children(sidebar),
                        )),
                )),
            )
    }

    fn support_authors_overlay(
        &self,
        palette: &theme::Palette,
        context: &ViewContext<Self>,
    ) -> View {
        Border::new()
            .background(palette.card_bg)
            .border_brush(palette.stroke)
            .border_thickness(1.0)
            .corner_radius(theme::CARD_RADIUS)
            .padding(Thickness::uniform(20.0))
            .content(
                StackPanel::new().spacing(12.0).children((
                    Grid::new()
                        .columns([GridLength::STAR, GridLength::Auto])
                        .children((
                            StackPanel::new()
                                .orientation(Orientation::Horizontal)
                                .spacing(8.0)
                                .children((
                                    theme::addon_icon_tile(palette, "Support"),
                                    TextBlock::new()
                                        .text("Support authors")
                                        .font_size(theme::SECTION_TITLE_SIZE)
                                        .font_weight(FontWeight::BOLD)
                                        .foreground(palette.text_primary)
                                        .vertical_alignment(VerticalAlignment::Center),
                                )),
                            theme::outline_button(palette, "Close")
                                .grid_column(1)
                                .automation_name("Close")
                                .on_click(context.callback(|_| Message::CloseSupportAuthors)),
                        )),
                    TextBlock::new()
                        .text("GitHub Sponsors, Ko-fi, Patreon, and project pages are the usual homes for financial support.")
                        .text_wrapping(TextWrapping::Wrap)
                        .foreground(palette.text_primary),
                    TextBlock::new()
                        .text("Stars, issue reports, and a kind note help even when you are not donating.")
                        .text_wrapping(TextWrapping::Wrap)
                        .foreground(palette.text_muted),
                    TextBlock::new()
                        .text("Never share account credentials or personal information when contacting an author.")
                        .text_wrapping(TextWrapping::Wrap)
                        .foreground(palette.text_muted),
                )),
            )
    }

    fn browse_view(&self, context: &ViewContext<Self>) -> View {
        self.catalog_surface_view(context, true)
    }

    fn installed_view(&self, context: &ViewContext<Self>) -> View {
        self.catalog_surface_view(context, false)
    }

    fn loadouts_view(&self, context: &ViewContext<Self>) -> View {
        let palette = theme::palette(self.selected_flavor);
        let flavor = self.current_flavor_slug();
        let visible = loadout::flavor_indices(&self.loadouts, flavor);
        let mut children = vec![KeyedView::new(
            "header",
            Grid::new()
                .columns([GridLength::STAR, GridLength::Auto])
                .children((
                    theme::page_header(
                        palette,
                        "Loadouts",
                        &format!(
                            "SKU-specific setups for {}",
                            GAME_FLAVORS[self.selected_flavor].label
                        ),
                    ),
                    theme::accent_button(palette, "New Loadout")
                        .grid_column(1)
                        .vertical_alignment(VerticalAlignment::Center)
                        .on_click(context.callback(|_| Message::NewLoadout)),
                )),
        )];

        if let Some(prompt) = &self.loadout_prompt {
            children.push(KeyedView::new(
                "prompt",
                self.loadout_prompt_view(prompt, palette, context),
            ));
        }

        if let Some(draft) = &self.loadout_draft {
            let addon_toggles = self
                .source_list
                .addons
                .iter()
                .filter(|addon| {
                    self.installed_addon_ids.contains(&addon.id)
                        || draft.addon_ids.contains(&addon.id)
                })
                .map(|addon| {
                    KeyedView::new(
                        addon.id.clone(),
                        Grid::new()
                            .columns([GridLength::STAR, GridLength::Auto])
                            .children((
                                StackPanel::new().spacing(2.0).children((
                                    TextBlock::new()
                                        .text(addon.name.clone())
                                        .foreground(palette.text_primary),
                                    TextBlock::new()
                                        .text(addon.category.clone())
                                        .font_size(11.0)
                                        .foreground(palette.text_muted),
                                )),
                                ToggleSwitch::new()
                                    .grid_column(1)
                                    .is_on(draft.addon_ids.contains(&addon.id))
                                    .on_toggled({
                                        let id = addon.id.clone();
                                        context.callback(move |enabled| {
                                            Message::ToggleLoadoutAddon(id.clone(), enabled)
                                        })
                                    }),
                            )),
                    )
                })
                .collect::<Vec<_>>();
            children.push(KeyedView::new(
                "editor",
                Border::new()
                    .background(palette.card_bg)
                    .border_brush(palette.accent)
                    .border_thickness(1.0)
                    .corner_radius(theme::CARD_RADIUS)
                    .padding(Thickness::uniform(18.0))
                    .content(
                        StackPanel::new().spacing(14.0).children((
                            TextBlock::new()
                                .text(if draft.editing_index.is_some() {
                                    "Edit loadout"
                                } else {
                                    "Create a loadout"
                                })
                                .font_size(20.0)
                                .font_weight(FontWeight::SEMI_BOLD)
                                .foreground(palette.text_primary),
                            TextBox::new()
                                .text(draft.name.clone())
                                .placeholder_text("Loadout name, e.g. Raid Night")
                                .on_text_changed(context.callback(Message::LoadoutNameChanged)),
                            TextBlock::new()
                                .text(format!("{} addons selected", draft.addon_ids.len()))
                                .foreground(palette.text_muted),
                            StackPanel::new()
                                .spacing(10.0)
                                .keyed_children(addon_toggles),
                            StackPanel::new()
                                .orientation(Orientation::Horizontal)
                                .spacing(8.0)
                                .children((
                                    theme::accent_button(palette, "Save Loadout")
                                        .on_click(context.callback(|_| Message::SaveLoadout)),
                                    theme::outline_button(palette, "Cancel")
                                        .on_click(context.callback(|_| Message::CancelLoadout)),
                                )),
                        )),
                    ),
            ));
        }

        if visible.is_empty() && self.loadout_draft.is_none() && self.loadout_prompt.is_none() {
            children.push(KeyedView::new(
                "empty",
                theme::empty_state(
                    palette,
                    "No loadouts for this SKU",
                    "Create a loadout from your installed addons for raids, PvP, or leveling.",
                ),
            ));
        } else {
            children.extend(visible.into_iter().map(|index| {
                let loadout = &self.loadouts[index];
                KeyedView::new(
                    format!("loadout-{index}"),
                    Border::new()
                        .background(palette.card_bg)
                        .border_brush(palette.stroke)
                        .border_thickness(1.0)
                        .corner_radius(theme::CARD_RADIUS)
                        .padding(Thickness::uniform(16.0))
                        .content(
                            Grid::new()
                                .columns([GridLength::STAR, GridLength::Auto])
                                .children((
                                    StackPanel::new().spacing(4.0).children((
                                        TextBlock::new()
                                            .text(loadout.name.clone())
                                            .font_size(18.0)
                                            .font_weight(FontWeight::SEMI_BOLD)
                                            .foreground(palette.text_primary),
                                        TextBlock::new()
                                            .text(format!(
                                                "{} addons · {}",
                                                loadout.addon_ids.len(),
                                                GAME_FLAVORS[self.selected_flavor].label
                                            ))
                                            .foreground(palette.text_muted),
                                    )),
                                    StackPanel::new()
                                        .grid_column(1)
                                        .orientation(Orientation::Horizontal)
                                        .spacing(8.0)
                                        .children((
                                            theme::accent_button(palette, "Apply").on_click(
                                                context.callback(move |_| {
                                                    Message::RequestApplyLoadout(index)
                                                }),
                                            ),
                                            theme::outline_button(palette, "Edit")
                                                .on_click(context.callback(move |_| {
                                                    Message::EditLoadout(index)
                                                })),
                                            theme::outline_button(palette, "Delete").on_click(
                                                context.callback(move |_| {
                                                    Message::RequestDeleteLoadout(index)
                                                }),
                                            ),
                                        )),
                                )),
                        ),
                )
            }));
        }

        ScrollViewer::new()
            .horizontal_scroll_bar_visibility(ScrollBarVisibility::Disabled)
            .content(
                StackPanel::new()
                    .spacing(16.0)
                    .margin(Thickness::uniform(theme::PAGE_MARGIN))
                    .keyed_children(children),
            )
    }

    fn loadout_prompt_view(
        &self,
        prompt: &LoadoutPrompt,
        palette: &theme::Palette,
        context: &ViewContext<Self>,
    ) -> View {
        let (title, message, confirm, cancel) = match prompt {
            LoadoutPrompt::ConfirmApply(plan) => (
                format!("Apply {}?", plan.name),
                format!(
                    "This will make {} change(s) on this SKU: install {} and remove {}.\nInstall: {}\nRemove: {}",
                    plan.change_count(),
                    plan.install.len(),
                    plan.uninstall.len(),
                    self.format_addon_names(&plan.install),
                    self.format_addon_names(&plan.uninstall)
                ),
                Some("Apply loadout"),
                "Cancel",
            ),
            LoadoutPrompt::ConfirmDelete { name, .. } => (
                format!("Delete {name}?"),
                "This loadout will be removed from saved settings. Installed addons are not changed."
                    .to_string(),
                Some("Delete loadout"),
                "Cancel",
            ),
            LoadoutPrompt::Result(result) => {
                let mut message = if result.installed.is_empty() && result.uninstalled.is_empty() {
                    "This loadout already matches the installed addons.".to_string()
                } else {
                    format!(
                        "Installed: {}\nRemoved: {}",
                        self.format_addon_names(&result.installed),
                        self.format_addon_names(&result.uninstalled)
                    )
                };
                if !result.failures.is_empty() {
                    let failures = result
                        .failures
                        .iter()
                        .map(|failure| {
                            format!(
                                "{} {}: {}",
                                failure.action,
                                self.addon_display_name(&failure.id),
                                failure.error
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    message.push_str("\nFailures:\n");
                    message.push_str(&failures);
                }
                (
                    if result.failed() {
                        format!("{} finished with errors", result.name)
                    } else {
                        format!("{} applied", result.name)
                    },
                    message,
                    None,
                    "Close",
                )
            }
        };
        Border::new()
            .background(palette.card_bg)
            .border_brush(palette.accent)
            .border_thickness(1.0)
            .corner_radius(theme::CARD_RADIUS)
            .padding(Thickness::uniform(18.0))
            .content(
                StackPanel::new().spacing(12.0).children((
                    TextBlock::new()
                        .text(title)
                        .font_size(20.0)
                        .font_weight(FontWeight::SEMI_BOLD)
                        .foreground(palette.text_primary),
                    TextBlock::new()
                        .text(message)
                        .text_wrapping(TextWrapping::Wrap)
                        .foreground(palette.text_muted),
                    StackPanel::new()
                        .orientation(Orientation::Horizontal)
                        .spacing(8.0)
                        .keyed_children(match confirm {
                            Some(label) => vec![
                                KeyedView::new(
                                    "confirm",
                                    theme::accent_button(palette, label).on_click(
                                        context.callback(|_| Message::ConfirmLoadoutPrompt),
                                    ),
                                ),
                                KeyedView::new(
                                    "cancel",
                                    theme::outline_button(palette, cancel).on_click(
                                        context.callback(|_| Message::DismissLoadoutPrompt),
                                    ),
                                ),
                            ],
                            None => vec![KeyedView::new(
                                "close",
                                theme::accent_button(palette, cancel)
                                    .on_click(context.callback(|_| Message::DismissLoadoutPrompt)),
                            )],
                        }),
                )),
            )
    }

    fn settings_view(&self, context: &ViewContext<Self>) -> View {
        let palette = theme::palette(self.selected_flavor);
        ScrollViewer::new()
            .horizontal_scroll_bar_visibility(ScrollBarVisibility::Disabled)
            .content(
                StackPanel::new()
                    .spacing(16.0)
                    .margin(Thickness::uniform(theme::PAGE_MARGIN))
                    .children((
                        StackPanel::new().spacing(12.0).children((
                            theme::page_header(palette, "Settings", "Configure application behavior"),
                            theme::outline_button(palette, "Show author-support message").on_click(
                                context.callback(|_| Message::RestoreSupportBanner),
                            ),
                        )),
                        InfoBar::new()
                            .is_open(self.wow_folder_missing)
                            .is_closable(false)
                            .severity(InfoBarSeverity::Warning)
                            .title(format!(
                                "{} folder not found",
                                GAME_FLAVORS[self.selected_flavor].label
                            ))
                            .message("Folder not found for the selected game version, please select it."),
                        TextBlock::new()
                            .text("Game Path")
                            .font_size(12.0)
                            .foreground(palette.text_muted),
                        Grid::new()
                            .columns([GridLength::STAR, GridLength::Auto])
                            .column_spacing(12.0)
                            .children((
                                TextBox::new()
                                    .text(self.wow_folder.clone())
                                    .placeholder_text(
                                        "C:\\Program Files (x86)\\World of Warcraft",
                                    )
                                    .on_text_changed(context.callback(Message::WowFolderChanged)),
                                theme::outline_button(palette, "Browse")
                                    .grid_column(1)
                                    .vertical_alignment(VerticalAlignment::Center)
                                    .on_click(context.callback(|_| Message::BrowseWowFolder)),
                            )),
                        TextBlock::new()
                            .text("Addon directory source")
                            .font_size(12.0)
                            .foreground(palette.text_muted),
                        TextBox::new()
                            .text(self.pending_source_base_url.clone())
                            .on_text_changed(context.callback(Message::SourceUrlChanged)),
                        InfoBar::new()
                            .is_open(true)
                            .is_closable(false)
                            .severity(InfoBarSeverity::Informational)
                            .title("HTTPS directory")
                            .message(
                                "The source must be an HTTPS URL. WinWam loads addons.<flavor>.json from this base, matching addons.schema.json.",
                            ),
                        Grid::new()
                            .columns([GridLength::STAR, GridLength::Auto])
                            .children((
                                TextBlock::new()
                                    .text("Check for Updates")
                                    .foreground(palette.text_primary)
                                    .vertical_alignment(VerticalAlignment::Center),
                                ToggleSwitch::new()
                                    .grid_column(1)
                                    .is_on(self.check_for_updates)
                                    .on_toggled(context.callback(Message::ToggleUpdates)),
                            )),
                        Grid::new()
                            .columns([GridLength::STAR, GridLength::Auto])
                            .children((
                                StackPanel::new().spacing(3.0).children((
                                    TextBlock::new()
                                        .text("Diagnostics")
                                        .font_size(12.0)
                                        .foreground(palette.text_muted),
                                    TextBlock::new()
                                        .text("Inspect the live application event stream.")
                                        .font_size(11.0)
                                        .foreground(palette.text_muted),
                                )),
                                theme::outline_button(
                                    palette,
                                    if self.hood_open { "Close the Hood" } else { "Open the Hood" },
                                )
                                .grid_column(1)
                                .vertical_alignment(VerticalAlignment::Center)
                                .on_click(context.callback(|_| Message::ToggleHood)),
                            )),
                        TextBlock::new()
                            .text("Telemetry detail")
                            .font_size(12.0)
                            .foreground(palette.text_muted),
                        ComboBox::new()
                            .width(220.0)
                            .horizontal_alignment(HorizontalAlignment::Left)
                            .items_source(["Off", "Errors only", "Standard", "Verbose"])
                            .selected_index(self.telemetry_level)
                            .on_selection_changed(context.callback(Message::SelectTelemetryLevel)),
                        TextBlock::new()
                            .text(match self.telemetry_level {
                                0 => "No in-app telemetry is collected.",
                                1 => "Only failures are retained.",
                                2 => "Navigation and operations are retained.",
                                _ => "All available diagnostics are retained.",
                            })
                            .font_size(11.0)
                            .foreground(palette.text_muted),
                        TextBlock::new()
                            .text("Theme")
                            .font_size(12.0)
                            .foreground(palette.text_muted),
                        ComboBox::new()
                            .width(160.0)
                            .horizontal_alignment(HorizontalAlignment::Left)
                            .items_source(["Dark"])
                            .selected_index(0)
                            .is_enabled(false),
                        theme::accent_button(palette, "Save Changes")
                            .on_click(context.callback(|_| Message::ApplySettings)),
                        TextBlock::new()
                            .text(format!(
                                "Directory v{} · {} · Privacy: https://github.com/bitobrian/WinWam/blob/main/PRIVACY.md",
                                self.source_list.schema_version,
                                self.source_list.directory.repository
                            ))
                            .font_size(11.0)
                            .text_wrapping(TextWrapping::Wrap)
                            .foreground(palette.text_muted),
                    )),
            )
    }
}

fn addon_card(
    addon: &Addon,
    palette: &theme::Palette,
    context: &ViewContext<WinWam>,
    installed: bool,
    managed: bool,
    expanded: bool,
    installing: bool,
) -> View {
    let overlay: View = if installing {
        StackPanel::new()
            .orientation(Orientation::Horizontal)
            .spacing(8.0)
            .horizontal_alignment(HorizontalAlignment::Right)
            .vertical_alignment(VerticalAlignment::Bottom)
            .margin(Thickness::uniform(8.0))
            .children((
                ProgressRing::new()
                    .width(18.0)
                    .height(18.0)
                    .is_indeterminate(true)
                    .is_active(true),
                TextBlock::new()
                    .text("Installing…")
                    .font_size(theme::META_SIZE)
                    .foreground(palette.accent),
            ))
    } else if !installed {
        theme::accent_button(palette, "Install")
            .height(32.0)
            .horizontal_alignment(HorizontalAlignment::Right)
            .vertical_alignment(VerticalAlignment::Bottom)
            .on_click({
                let id = addon.id.clone();
                context.callback(move |_| Message::InstallAddon(id.clone()))
            })
            .into()
    } else {
        let remove = theme::outline_button(palette, "Remove")
            .height(32.0)
            .horizontal_alignment(HorizontalAlignment::Right)
            .vertical_alignment(VerticalAlignment::Bottom)
            .enabled(managed)
            .automation_name(if managed {
                format!("Remove {}", addon.name)
            } else {
                "WinWAM did not install this addon, so it will not remove it.".to_string()
            });
        if managed {
            remove
                .on_click({
                    let id = addon.id.clone();
                    context.callback(move |_| Message::UninstallAddon(id.clone()))
                })
                .into()
        } else {
            remove.into()
        }
    };
    let downloads: View = match addon.download_count {
        Some(count) => TextBlock::new()
            .text(directory::compact_count(count))
            .font_size(theme::META_SIZE)
            .foreground(palette.text_muted)
            .into(),
        None => Border::new().height(0.0).into(),
    };
    let source: View = match directory::resolved_source_url(addon) {
        Some(url) => Button::new()
            .style(ButtonStyle::Default)
            .width(24.0)
            .height(24.0)
            .automation_name(format!("Open {} source project", addon.name))
            .on_click(context.callback(move |_| Message::OpenHttpsUrl(url.clone())))
            .content(
                Image::new()
                    .source_data(EncodedImage::from_static(include_bytes!(
                        "../assets/generated/source-mark.png"
                    )))
                    .width(15.0)
                    .height(15.0)
                    .stretch(Stretch::Uniform),
            ),
        None => Border::new().width(0.0).height(0.0).into(),
    };
    Border::new()
        .width(theme::CATALOG_CARD_WIDTH)
        .height(theme::CATALOG_CARD_HEIGHT)
        .background(palette.card_bg)
        .border_brush(palette.stroke)
        .border_thickness(1.0)
        .corner_radius(theme::CARD_RADIUS)
        .content(
            Grid::new()
                .rows([
                    GridLength::Pixel(theme::CATALOG_ART_HEIGHT),
                    GridLength::STAR,
                ])
                .children((
                    Border::new()
                        .background(palette.tile_bg)
                        .content(Grid::new().children((
                            theme::addon_icon_tile_at(palette, &addon.name, 58.0),
                            if installed {
                                theme::installed_badge(palette)
                            } else {
                                Border::new().width(0.0).into()
                            },
                            overlay,
                        ))),
                    StackPanel::new()
                        .grid_row(1)
                        .spacing(4.0)
                        .margin(Thickness::xy(12.0, 10.0))
                        .children((
                            TextBlock::new()
                                .text(addon.name.clone())
                                .font_size(theme::CARD_TITLE_SIZE)
                                .font_weight(FontWeight::SEMI_BOLD)
                                .foreground(palette.text_primary),
                            TextBlock::new()
                                .text(format!("by {}", addon.author))
                                .font_size(theme::META_SIZE)
                                .foreground(palette.text_muted),
                            downloads,
                            StackPanel::new()
                                .orientation(Orientation::Horizontal)
                                .spacing(8.0)
                                .children((
                                    source,
                                    theme::outline_button(
                                        palette,
                                        if expanded {
                                            "Hide details"
                                        } else {
                                            "View details"
                                        },
                                    )
                                    .height(28.0)
                                    .on_click({
                                        let id = addon.id.clone();
                                        context.callback(move |_| {
                                            Message::ToggleAddonDetails(id.clone())
                                        })
                                    }),
                                )),
                        )),
                )),
        )
}

fn message_is_error(message: &Message) -> bool {
    matches!(
        message,
        Message::InstallFinished(_, Err(_)) | Message::DirectoryRefreshFinished(_, Err(_))
    )
}

fn message_telemetry(message: &Message) -> String {
    match message {
        Message::Navigate(Page::Discover) => "Navigation → Discover".to_string(),
        Message::Navigate(Page::Installed) => "Navigation → Installed".to_string(),
        Message::Navigate(Page::Loadouts) => "Navigation → Loadouts".to_string(),
        Message::Navigate(Page::Workshop) => "Navigation → Workshop".to_string(),
        Message::Navigate(Page::Settings) => "Navigation → Settings".to_string(),
        Message::CheckForUpdates => "Check for updates requested".to_string(),
        Message::DirectoryRefreshFinished(_, Ok(_)) => "Directory refresh completed".to_string(),
        Message::DirectoryRefreshFinished(_, Err(error)) => {
            format!("Directory refresh failed · {error}")
        }
        Message::ShowWorkshopScreen(WorkshopScreen::Overview) => "Workshop → Overview".to_string(),
        Message::ShowWorkshopScreen(WorkshopScreen::Anatomy) => "Workshop → Anatomy".to_string(),
        Message::Search(query) => format!("Search changed · {} characters", query.len()),
        Message::SelectFlavor(index) => format!("SKU selection changed · {index:?}"),
        Message::SelectCategory(index) => format!("Category selection changed · {index:?}"),
        Message::OpenHttpsUrl(url) => format!("Open source URL · {url}"),
        Message::SourceUrlChanged(_) => "Directory source edited".to_string(),
        Message::WowFolderChanged(_) => "WoW folder edited".to_string(),
        Message::BrowseWowFolder => "WoW folder picker opened".to_string(),
        Message::ToggleUpdates(value) => format!("Update checks toggled · {value}"),
        Message::ApplySettings => "Settings applied".to_string(),
        Message::InstallAddon(id) => format!("Install started · {id}"),
        Message::InstallFinished(id, Ok(())) => format!("Install completed · {id}"),
        Message::InstallFinished(id, Err(error)) => format!("Install failed · {id} · {error}"),
        Message::UninstallAddon(id) => format!("Uninstall requested · {id}"),
        Message::ToggleAddonDetails(id) => format!("Addon details toggled · {id}"),
        Message::CloseAddonDetails => "Addon details closed".to_string(),
        Message::DismissSupportBanner => "Support banner dismissed".to_string(),
        Message::RestoreSupportBanner => "Support banner restored".to_string(),
        Message::ShowSupportAuthors => "Support authors opened".to_string(),
        Message::CloseSupportAuthors => "Support authors closed".to_string(),
        Message::ToggleHood => "Hood toggled".to_string(),
        Message::SelectTelemetryLevel(level) => format!("Telemetry selection · {level:?}"),
        Message::NewLoadout => "Loadout creation started".to_string(),
        Message::EditLoadout(index) => format!("Loadout edit started · {index}"),
        Message::RequestDeleteLoadout(index) => format!("Loadout delete requested · {index}"),
        Message::RequestApplyLoadout(index) => format!("Loadout apply requested · {index}"),
        Message::ConfirmLoadoutPrompt => "Loadout prompt confirmed".to_string(),
        Message::DismissLoadoutPrompt => "Loadout prompt dismissed".to_string(),
        Message::LoadoutNameChanged(name) => {
            format!("Loadout name edited · {} characters", name.len())
        }
        Message::ToggleLoadoutAddon(id, enabled) => {
            format!("Loadout addon toggled · {id} · {enabled}")
        }
        Message::SaveLoadout => "Loadout save requested".to_string(),
        Message::CancelLoadout => "Loadout edit cancelled".to_string(),
    }
}

fn category_ribbon_icon(category: &str) -> Option<&'static [u8]> {
    match category {
        "Bags" => Some(include_bytes!("../assets/generated/ribbon-bar/bags.png")),
        "Collections" => Some(include_bytes!(
            "../assets/generated/ribbon-bar/collections.png"
        )),
        "Combat" => Some(include_bytes!("../assets/generated/ribbon-bar/combat.png")),
        "Dungeons" => Some(include_bytes!("../assets/generated/ribbon-bar/dungeon.png")),
        "Economy" => Some(include_bytes!("../assets/generated/ribbon-bar/economy.png")),
        "Interface" => Some(include_bytes!(
            "../assets/generated/ribbon-bar/interface.png"
        )),
        "QualityOfLife" => Some(include_bytes!(
            "../assets/generated/ribbon-bar/qualityoflife.png"
        )),
        "Quests" => Some(include_bytes!(
            "../assets/generated/ribbon-bar/questing.png"
        )),
        "Libraries" => Some(include_bytes!(
            "../assets/generated/ribbon-bar/crafting.png"
        )),
        _ => None,
    }
}

fn meta_row(palette: &theme::Palette, label: &str, value: &str) -> View {
    StackPanel::new().spacing(2.0).children((
        TextBlock::new()
            .text(label.to_string())
            .font_size(theme::META_SIZE)
            .foreground(palette.text_muted),
        TextBlock::new()
            .text(value.to_string())
            .foreground(palette.text_primary),
    ))
}

fn civil_unix_days(year: i64, month: u32, day: u32) -> Option<i64> {
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let yoe = year.rem_euclid(400);
    let month = month as i64;
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146097 + doe - 719468)
}

fn parse_published_at(value: &str) -> Option<std::time::SystemTime> {
    let date = value.trim().split(['T', ' ']).next()?;
    let mut parts = date.split('-');
    let year: i64 = parts.next()?.parse().ok()?;
    let month: u32 = parts.next()?.parse().ok()?;
    let day: u32 = parts.next()?.parse().ok()?;
    let days = civil_unix_days(year, month, day)?;
    let seconds = u64::try_from(days.checked_mul(86400)?).ok()?;
    Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(seconds))
}

fn format_age(then: std::time::SystemTime) -> String {
    let Ok(elapsed) = std::time::SystemTime::now().duration_since(then) else {
        return "today".to_string();
    };
    match elapsed.as_secs() / 86400 {
        0 => "today".to_string(),
        1 => "1 day ago".to_string(),
        days => format!("{days} days ago"),
    }
}

fn matches_query(addon: &Addon, query: &str) -> bool {
    query.is_empty()
        || addon.name.to_lowercase().contains(query)
        || addon.author.to_lowercase().contains(query)
        || addon.category.to_lowercase().contains(query)
        || addon
            .tags
            .iter()
            .any(|tag| tag.to_lowercase().contains(query))
}

fn initial_page(missing: bool, last: Option<&str>) -> Page {
    if missing {
        Page::Settings
    } else {
        page_from_slug(last).unwrap_or(Page::Installed)
    }
}

fn page_from_slug(slug: Option<&str>) -> Option<Page> {
    match slug? {
        "discover" => Some(Page::Discover),
        "installed" => Some(Page::Installed),
        "loadouts" => Some(Page::Loadouts),
        "workshop" => Some(Page::Workshop),
        _ => None,
    }
}

fn page_slug(page: Page) -> Option<&'static str> {
    match page {
        Page::Discover => Some("discover"),
        Page::Installed => Some("installed"),
        Page::Loadouts => Some("loadouts"),
        Page::Workshop => Some("workshop"),
        Page::Settings => None,
    }
}

fn active_catalog_surface_mut<'a>(
    page: Page,
    flavor: usize,
    discover: &'a mut [CatalogSurfaceState; 5],
    installed: &'a mut [CatalogSurfaceState; 5],
) -> Option<&'a mut CatalogSurfaceState> {
    match page {
        Page::Discover => discover.get_mut(flavor),
        Page::Installed => installed.get_mut(flavor),
        _ => None,
    }
}

fn apply_directory_result(
    current: AddonSourceList,
    result: Result<AddonSourceList, String>,
) -> (AddonSourceList, Option<String>) {
    match result {
        Ok(source_list) => (source_list, None),
        Err(error) => (current, Some(error)),
    }
}

fn adopt_directory_result(
    result: Result<AddonSourceList, String>,
    cache: Option<AddonSourceList>,
    current: AddonSourceList,
) -> (AddonSourceList, Option<String>, bool) {
    match result {
        Ok(source_list) => (source_list, None, false),
        Err(error) => {
            let (recovered, stale) = directory::keep_last_cache(
                Err(directory::DirectoryError::Network(error.clone())),
                cache,
            );
            if stale && let Some(source_list) = recovered {
                (
                    source_list,
                    Some(directory::STALE_DIRECTORY_MESSAGE.to_string()),
                    true,
                )
            } else {
                let (source_list, directory_error) = apply_directory_result(current, Err(error));
                (
                    source_list,
                    directory_error.map(|error| format!("Could not load directory: {error}")),
                    false,
                )
            }
        }
    }
}

fn adopt_directory_load(
    index: usize,
    source_base_url: &str,
    current: AddonSourceList,
) -> (AddonSourceList, Option<String>, bool) {
    match load_source_list(index, source_base_url) {
        Ok(source_list) => (source_list, None, false),
        Err(error) => {
            logging::error(&format!("Could not load the addon directory: {error}"));
            adopt_directory_result(
                Err(error.to_string()),
                directory::load_cached_directory(GAME_FLAVORS[index].slug),
                current,
            )
        }
    }
}

fn flavor_index_from_slug(slug: &str) -> usize {
    GAME_FLAVORS
        .iter()
        .position(|flavor| flavor.slug.eq_ignore_ascii_case(slug.trim()))
        .unwrap_or(0)
}

fn flavor_install_folders(index: usize) -> &'static [&'static str] {
    match index {
        0 => &["_retail_"],
        1 => &["_classic_"],
        2 => &["_classic_era_"],
        3 => &["_classic_anniversary_"],
        4 => &["_classic_era_"],
        _ => &[],
    }
}

fn is_wow_folder_for_flavor(path: &Path, index: usize) -> bool {
    path.is_dir()
        && flavor_install_folders(index)
            .iter()
            .any(|folder| path.join(folder).is_dir())
}

fn addons_folder(wow_folder: &Path, index: usize) -> Option<PathBuf> {
    flavor_install_folders(index)
        .first()
        .map(|folder| wow_folder.join(folder).join("Interface").join("AddOns"))
}

fn catalog_entries(addons: &[Addon]) -> Vec<scan::CatalogEntry<'_>> {
    addons
        .iter()
        .map(|addon| scan::CatalogEntry {
            id: &addon.id,
            name: &addon.name,
            repo: &addon.repo,
        })
        .collect()
}

fn scan_installed(
    wow_folder: Option<&Path>,
    index: usize,
    addons: &[Addon],
) -> (Vec<scan::InstalledAddon>, BTreeSet<String>) {
    let catalog = catalog_entries(addons);
    let installed = scan::scan_installed_addons(
        wow_folder
            .and_then(|path| addons_folder(path, index))
            .as_deref(),
        &catalog,
    );
    let ids = installed.iter().map(|addon| addon.id.clone()).collect();
    (installed, ids)
}

#[cfg(debug_assertions)]
fn install_test_addon(wow_folder: &Path, index: usize, id: &str) -> std::io::Result<()> {
    let root = addons_folder(wow_folder, index)
        .ok_or_else(|| std::io::Error::other("unknown game flavor"))?
        .join(id);
    std::fs::create_dir_all(&root)?;
    std::fs::write(root.join(".winwam-id"), id)?;
    std::fs::write(
        root.join(format!("{id}.toc")),
        format!("## Title: {id}\n## Version: 1.0.0\n{id}.lua\n"),
    )?;
    std::fs::write(root.join(format!("{id}.lua")), "-- WinWam test addon\n")
}

#[cfg(not(debug_assertions))]
fn install_test_addon(_wow_folder: &Path, _index: usize, _id: &str) -> std::io::Result<()> {
    Err(std::io::Error::other("installation is not implemented yet"))
}

#[cfg(debug_assertions)]
fn uninstall_test_addon(wow_folder: &Path, index: usize, id: &str) -> std::io::Result<()> {
    let addons = addons_folder(wow_folder, index)
        .ok_or_else(|| std::io::Error::other("unknown game flavor"))?;
    let root = scan::find_addon_directory(&addons, id).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "addon folder not found")
    })?;
    let marker = std::fs::read_to_string(root.join(".winwam-id"))?;
    if marker.trim() != id {
        return Err(std::io::Error::other(
            "addon ownership marker does not match",
        ));
    }
    std::fs::remove_dir_all(root)
}

#[cfg(not(debug_assertions))]
fn uninstall_test_addon(_wow_folder: &Path, _index: usize, _id: &str) -> std::io::Result<()> {
    Err(std::io::Error::other(
        "uninstallation is not implemented yet",
    ))
}

fn detect_wow_folder(index: usize) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    #[cfg(debug_assertions)]
    candidates.push(PathBuf::from(LOCAL_TEST_ROOT).join("World of Warcraft"));
    for variable in ["ProgramFiles(x86)", "ProgramFiles"] {
        if let Some(program_files) = std::env::var_os(variable) {
            candidates.push(PathBuf::from(program_files).join("World of Warcraft"));
        }
    }
    candidates.extend([
        PathBuf::from(r"C:\Program Files (x86)\World of Warcraft"),
        PathBuf::from(r"C:\Program Files\World of Warcraft"),
    ]);

    candidates
        .into_iter()
        .find(|path| is_wow_folder_for_flavor(path, index))
}

fn main() {
    logging::initialize();
    logging::debug("Starting WinWam");
    if let Err(error) = App::run_component::<WinWam>(()) {
        logging::error(&format!("WinWam failed to start: {error}"));
        panic!("WinWam failed to start: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_card_width_is_three_column_reference() {
        assert_eq!(theme::catalog_card_width(), 362.0);
        assert_eq!(theme::CATALOG_CARD_WIDTH, 362.0);
        assert!(!directory::CATEGORY_ORDER.contains(&"All"));
        assert_eq!(directory::compact_count(1500), "1.5k");
        assert_eq!(directory::compact_count(12_300), "12.3k");
    }

    #[test]
    fn flavor_index_from_slug_is_case_insensitive() {
        assert_eq!(flavor_index_from_slug("classic"), 2);
        assert_eq!(flavor_index_from_slug(" MoP-Classic "), 1);
        assert_eq!(flavor_index_from_slug("unknown"), 0);
    }

    fn sample_addon(id: &str) -> Addon {
        Addon {
            id: id.into(),
            name: id.into(),
            summary: "Bags".into(),
            author: "Northwind Labs".into(),
            source_kind: "GitHub".into(),
            host: "github.com".into(),
            owner: "winwam-test".into(),
            repo: id.into(),
            category: "Bags".into(),
            tags: vec!["inventory".into()],
            ..Addon::default()
        }
    }

    fn sample_source_list(ids: &[&str]) -> AddonSourceList {
        AddonSourceList {
            schema_version: "1.0".into(),
            directory: directory::DirectoryInfo {
                name: "test".into(),
                repository: "https://github.com/bitobrian/wow-addons-directory".into(),
                description: None,
                generated_at: None,
            },
            flavor: "Retail".into(),
            addons: ids.iter().copied().map(sample_addon).collect(),
        }
    }

    #[test]
    fn initial_page_defaults_to_installed() {
        assert_eq!(initial_page(true, Some("discover")), Page::Settings);
        assert_eq!(initial_page(false, None), Page::Installed);
        assert_eq!(initial_page(false, Some("discover")), Page::Discover);
        assert_eq!(initial_page(false, Some("settings")), Page::Installed);
        assert_eq!(initial_page(false, Some("workshop")), Page::Workshop);
    }

    #[test]
    fn sku_switch_restores_per_slug_page() {
        let mut last_pages = BTreeMap::from([
            ("retail".to_string(), "discover".to_string()),
            ("forever".to_string(), "workshop".to_string()),
        ]);
        assert_eq!(
            initial_page(false, last_pages.get("retail").map(String::as_str)),
            Page::Discover
        );
        last_pages.insert(
            "retail".to_string(),
            page_slug(Page::Loadouts).unwrap().into(),
        );
        assert_eq!(
            initial_page(false, last_pages.get("forever").map(String::as_str)),
            Page::Workshop
        );
        assert_eq!(
            initial_page(true, last_pages.get("forever").map(String::as_str)),
            Page::Settings
        );
        assert_eq!(
            initial_page(false, last_pages.get("retail").map(String::as_str)),
            Page::Loadouts
        );
    }

    #[test]
    fn catalog_surfaces_stay_independent() {
        let mut discover = std::array::from_fn(|_| CatalogSurfaceState::default());
        let mut installed = std::array::from_fn(|_| CatalogSurfaceState::default());
        active_catalog_surface_mut(Page::Discover, 0, &mut discover, &mut installed)
            .unwrap()
            .query = "bags".into();
        active_catalog_surface_mut(Page::Installed, 0, &mut discover, &mut installed)
            .unwrap()
            .query = "raid".into();
        active_catalog_surface_mut(Page::Discover, 4, &mut discover, &mut installed)
            .unwrap()
            .query = "quest".into();
        assert_eq!(discover[0].query, "bags");
        assert_eq!(installed[0].query, "raid");
        assert_eq!(discover[4].query, "quest");
        assert!(installed[4].query.is_empty());
        assert!(
            active_catalog_surface_mut(Page::Workshop, 0, &mut discover, &mut installed).is_none()
        );
    }

    #[test]
    fn apply_directory_result_keeps_nonempty_catalog() {
        let current = sample_source_list(&["arcane-alerts", "bag-commander"]);
        let (kept, error) = apply_directory_result(current, Err("network down".to_string()));
        assert_eq!(kept.addons.len(), 2);
        assert_eq!(kept.addons[0].id, "arcane-alerts");
        assert_eq!(error.as_deref(), Some("network down"));

        let empty = sample_source_list(&[]);
        let (kept_empty, error) = apply_directory_result(empty, Err("network down".to_string()));
        assert!(kept_empty.addons.is_empty());
        assert!(error.is_some());

        let (replaced, error) = apply_directory_result(
            sample_source_list(&["old"]),
            Ok(sample_source_list(&["new"])),
        );
        assert_eq!(replaced.addons[0].id, "new");
        assert!(error.is_none());
    }

    #[test]
    fn matches_query_covers_name_author_and_tags() {
        let addon = Addon {
            id: "test-bag-manager".into(),
            name: "Test Bag Manager".into(),
            summary: "Bags".into(),
            author: "Northwind Labs".into(),
            source_kind: "GitHub".into(),
            host: "github.com".into(),
            owner: "winwam-test".into(),
            repo: "test-bag-manager".into(),
            category: "Bags".into(),
            tags: vec!["inventory".into()],
            ..Addon::default()
        };
        assert!(matches_query(&addon, "bag"));
        assert!(matches_query(&addon, "northwind"));
        assert!(matches_query(&addon, "inventory"));
        assert!(matches_query(&addon, "bags"));
        assert!(!matches_query(&addon, "raid"));
    }

    #[test]
    fn format_age_reports_whole_days() {
        let now = std::time::SystemTime::now();
        assert_eq!(format_age(now), "today");
        assert_eq!(
            format_age(now - std::time::Duration::from_secs(3 * 86400 + 120)),
            "3 days ago"
        );
        assert!(parse_published_at("2026-09-01T12:00:00Z").is_some());
    }
}
