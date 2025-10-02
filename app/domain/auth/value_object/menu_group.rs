use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::auth::value_object::menu::Menu;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuGroup(HashSet<Menu>);

impl Default for MenuGroup {
    fn default() -> Self {
        Self(HashSet::with_capacity(0))
    }
}
impl MenuGroup {
    pub fn new(set: HashSet<Menu>) -> Self {
        Self(set)
    }

    pub fn permit(&self, required: &Menu) -> bool {
        self.0.contains(required)
    }

    pub fn permits(&self, checker: MenuChecker) -> bool {
        match checker {
            MenuChecker::All(group) => group.0.iter().all(|p| self.0.contains(p)),
            MenuChecker::Any(group) => group.0.iter().any(|p| self.0.contains(p)),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug, Clone)]
pub enum MenuChecker {
    All(MenuGroup),
    Any(MenuGroup),
}

impl MenuChecker {
    pub fn all(group: MenuGroup) -> Self {
        Self::All(group)
    }

    pub fn any(group: MenuGroup) -> Self {
        Self::Any(group)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let mut set = HashSet::new();
        set.insert(Menu::new(1));
        let group = MenuGroup::new(set);
        assert_eq!(group.0.len(), 1);
    }

    #[test]
    fn test_empty() {
        let group = MenuGroup::default();
        assert!(group.0.is_empty());
    }

    #[test]
    fn test_permit() {
        let mut set = HashSet::new();
        set.insert(Menu::new(1));
        let group = MenuGroup::new(set);
        assert!(group.permit(&Menu::new(1)));
    }

    #[test]
    fn test_permits_all() {
        let mut set1 = HashSet::new();
        set1.insert(Menu::new(1));
        set1.insert(Menu::new(2));
        let group1 = MenuGroup::new(set1);

        let mut set2 = HashSet::new();
        set2.insert(Menu::new(1));
        let group2 = MenuGroup::new(set2);

        assert!(group1.permits(MenuChecker::all(group2)));
    }

    #[test]
    fn test_permits_any() {
        let mut set1 = HashSet::new();
        set1.insert(Menu::new(1));
        let group1 = MenuGroup::new(set1);

        let mut set2 = HashSet::new();
        set2.insert(Menu::new(1));
        set2.insert(Menu::new(3));
        let group2 = MenuGroup::new(set2);

        assert!(group1.permits(MenuChecker::any(group2)));
    }
}
