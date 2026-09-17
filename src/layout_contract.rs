#[cfg(test)]
const VISUAL_CHROME: &[&str] = &[
    "DISCOVER",
    "MY ADDONS",
    "LOADOUTS",
    "ADDON WORKSHOP",
    "GAME VERSION",
    "Check for Updates",
    "Settings",
    "Last checked: just now",
    "addons available.",
    "ADDON TYPES",
    "DID YOU KNOW?",
    "Enjoy an addon? Consider supporting its author.",
    "Support authors",
    "COMMUNITY PICKS",
    "Featured Addons",
    "YOUR COLLECTION",
    "My Addons",
    "Search addons",
    "START BUILDING",
    "Create your first addon.",
    "Start learning",
    "Using AI assistance",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::directory::AddonSourceList;
    use crate::theme;

    #[test]
    fn reference_window_matches_vnext_plan() {
        assert_eq!(theme::REFERENCE_CLIENT_WIDTH, 1440.0);
        assert_eq!(theme::REFERENCE_CLIENT_HEIGHT, 856.0);
        assert_eq!(theme::MIN_CLIENT_WIDTH, 1400.0);
        assert_eq!(theme::MIN_CLIENT_HEIGHT, 832.0);
        assert_eq!(theme::GAME_PANEL_WIDTH, 280.0);
        assert_eq!(theme::HEADER_HEIGHT, 70.0);
        assert_eq!(theme::RIBBON_HEIGHT, 82.0);
        assert_eq!(theme::PAGE_MARGIN, 24.0);
        assert_eq!(theme::CARD_GAP, 12.0);
        assert_eq!(theme::catalog_card_width(), 362.0);
        assert_eq!(theme::CATALOG_CARD_WIDTH, 362.0);
        assert_eq!(theme::MODAL_WIDTH, 760.0);
        assert_eq!(theme::MODAL_SIDEBAR_WIDTH, 250.0);
        assert_eq!(theme::FOCUS_RING, 2.0);
        assert_eq!(theme::CONTROL_HEIGHT, 42.0);
        assert_eq!(theme::SETTINGS_BUTTON_SIZE, 44.0);
    }

    #[test]
    fn visual_catalog_fixture_parses() {
        let json = include_str!("../data/fixtures/visual-catalog.json");
        let list = AddonSourceList::from_json(json).expect("visual catalog");
        assert_eq!(list.schema_version, "1.1");
        assert!(list.addons.len() >= 2);
        assert!(
            list.addons
                .iter()
                .any(|addon| addon.id == "arcane-alerts" && addon.download_count.is_some())
        );
        for phrase in VISUAL_CHROME {
            assert!(
                json.contains(phrase),
                "visual catalog missing chrome string {phrase}"
            );
        }
    }
}
