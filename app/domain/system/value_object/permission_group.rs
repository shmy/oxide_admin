use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::system::value_object::permission::Permission;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGroup(HashSet<Permission>);

impl Default for PermissionGroup {
    fn default() -> Self {
        Self(HashSet::with_capacity(0))
    }
}
impl PermissionGroup {
    pub fn new(set: HashSet<Permission>) -> Self {
        Self(set)
    }

    pub fn permit(&self, required: &Permission) -> bool {
        self.0.contains(required)
    }

    pub fn permits(&self, checker: PermissionChecker) -> bool {
        match checker {
            PermissionChecker::All(group) => group.0.iter().all(|p| self.0.contains(p)),
            PermissionChecker::Any(group) => group.0.iter().any(|p| self.0.contains(p)),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug, Clone)]
pub enum PermissionChecker {
    All(PermissionGroup),
    Any(PermissionGroup),
}

impl PermissionChecker {
    pub fn all(group: PermissionGroup) -> Self {
        Self::All(group)
    }

    pub fn any(group: PermissionGroup) -> Self {
        Self::Any(group)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let mut set = HashSet::new();
        set.insert(Permission::new(1));
        let group = PermissionGroup::new(set);
        assert_eq!(group.0.len(), 1);
    }

    #[test]
    fn test_empty() {
        let group = PermissionGroup::default();
        assert!(group.0.is_empty());
    }

    #[test]
    fn test_permit() {
        let mut set = HashSet::new();
        set.insert(Permission::new(1));
        let group = PermissionGroup::new(set);
        assert!(group.permit(&Permission::new(1)));
    }

    #[test]
    fn test_permits_all() {
        let mut set1 = HashSet::new();
        set1.insert(Permission::new(1));
        set1.insert(Permission::new(2));
        let group1 = PermissionGroup::new(set1);

        let mut set2 = HashSet::new();
        set2.insert(Permission::new(1));
        let group2 = PermissionGroup::new(set2);

        assert!(group1.permits(PermissionChecker::all(group2)));
    }

    #[test]
    fn test_permits_any() {
        let mut set1 = HashSet::new();
        set1.insert(Permission::new(1));
        let group1 = PermissionGroup::new(set1);

        let mut set2 = HashSet::new();
        set2.insert(Permission::new(1));
        set2.insert(Permission::new(3));
        let group2 = PermissionGroup::new(set2);

        assert!(group1.permits(PermissionChecker::any(group2)));
    }
}
