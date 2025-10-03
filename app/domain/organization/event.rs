use crate::{
    organization::entity::{department::Department, role::Role, user::User},
    shared::event_util::UpdatedEvent,
};
#[derive(Debug, Clone)]
pub enum OrganizationEvent {
    UsersCreated {
        items: Vec<User>,
    },
    UsersUpdated {
        items: Vec<UpdatedEvent<User>>,
    },
    UsersDeleted {
        items: Vec<User>,
    },
    RolesCreated {
        items: Vec<Role>,
    },
    RolesUpdated {
        items: Vec<UpdatedEvent<Role>>,
    },
    RolesDeleted {
        items: Vec<Role>,
    },
    DepartmentsCreated {
        items: Vec<Department>,
    },
    DepartmentsUpdated {
        items: Vec<UpdatedEvent<Department>>,
    },
    DepartmentsDeleted {
        items: Vec<Department>,
    },
}
