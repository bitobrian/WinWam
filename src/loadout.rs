use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Loadout {
    pub name: String,
    #[serde(default)]
    pub flavor: String,
    pub addon_ids: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadoutDraft {
    pub editing_index: Option<usize>,
    pub name: String,
    pub addon_ids: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadoutPlan {
    pub name: String,
    pub flavor: String,
    pub install: Vec<String>,
    pub uninstall: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadoutFailure {
    pub id: String,
    pub action: &'static str,
    pub error: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadoutApplyResult {
    pub name: String,
    pub installed: Vec<String>,
    pub uninstalled: Vec<String>,
    pub failures: Vec<LoadoutFailure>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadoutPrompt {
    ConfirmApply(LoadoutPlan),
    ConfirmDelete { index: usize, name: String },
    Result(LoadoutApplyResult),
}

impl LoadoutPlan {
    pub fn is_noop(&self) -> bool {
        self.install.is_empty() && self.uninstall.is_empty()
    }

    pub fn change_count(&self) -> usize {
        self.install.len() + self.uninstall.len()
    }
}

impl LoadoutApplyResult {
    pub fn failed(&self) -> bool {
        !self.failures.is_empty()
    }
}

pub fn plan(
    loadout: &Loadout,
    installed: &BTreeSet<String>,
    removable: &BTreeSet<String>,
) -> LoadoutPlan {
    LoadoutPlan {
        name: loadout.name.clone(),
        flavor: loadout.flavor.clone(),
        install: loadout
            .addon_ids
            .difference(installed)
            .cloned()
            .collect(),
        uninstall: removable.difference(&loadout.addon_ids).cloned().collect(),
    }
}

pub fn flavor_indices(loadouts: &[Loadout], flavor: &str) -> Vec<usize> {
    loadouts
        .iter()
        .enumerate()
        .filter(|(_, loadout)| loadout.flavor.is_empty() || loadout.flavor == flavor)
        .map(|(index, _)| index)
        .collect()
}

pub fn assign_missing_flavors(loadouts: &mut [Loadout], flavor: &str) {
    for loadout in loadouts {
        if loadout.flavor.is_empty() {
            loadout.flavor = flavor.to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loadout(name: &str, flavor: &str, ids: &[&str]) -> Loadout {
        Loadout {
            name: name.to_string(),
            flavor: flavor.to_string(),
            addon_ids: ids.iter().map(|id| (*id).to_string()).collect(),
        }
    }

    #[test]
    fn plan_splits_installs_and_removals() {
        let loadout = loadout("Raid Night", "retail", &["alpha", "bravo"]);
        let installed: BTreeSet<String> = ["bravo", "charlie"]
            .into_iter()
            .map(str::to_string)
            .collect();
        let plan = plan(&loadout, &installed, &installed);
        assert_eq!(plan.install, vec!["alpha".to_string()]);
        assert_eq!(plan.uninstall, vec!["charlie".to_string()]);
        assert_eq!(plan.change_count(), 2);
        assert!(!plan.is_noop());
    }

    #[test]
    fn plan_is_noop_when_sets_match() {
        let loadout = loadout("Solo", "classic", &["alpha"]);
        let installed: BTreeSet<String> = ["alpha"].into_iter().map(str::to_string).collect();
        assert!(plan(&loadout, &installed, &installed).is_noop());
    }

    #[test]
    fn flavor_indices_keep_matching_and_unscoped_loadouts() {
        let loadouts = vec![
            loadout("Retail Raid", "retail", &["a"]),
            loadout("Legacy", "", &["b"]),
            loadout("Classic Only", "classic", &["c"]),
        ];
        assert_eq!(flavor_indices(&loadouts, "retail"), vec![0, 1]);
        assert_eq!(flavor_indices(&loadouts, "classic"), vec![1, 2]);
    }

    #[test]
    fn plan_does_not_uninstall_unmanaged_addons() {
        let loadout = loadout("Solo", "retail", &["alpha"]);
        let installed: BTreeSet<String> = ["alpha", "local-ui"]
            .into_iter()
            .map(str::to_string)
            .collect();
        let removable: BTreeSet<String> = ["alpha"].into_iter().map(str::to_string).collect();
        assert!(plan(&loadout, &installed, &removable).is_noop());
    }

    #[test]
    fn assign_missing_flavors_stamps_current_sku() {
        let mut loadouts = vec![loadout("Legacy", "", &["a"])];
        assign_missing_flavors(&mut loadouts, "retail");
        assert_eq!(loadouts[0].flavor, "retail");
    }
}
