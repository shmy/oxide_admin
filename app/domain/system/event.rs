use crate::system::entity::department::Department;
use crate::system::entity::file::File;
use crate::system::entity::sched::Sched;
use crate::{
    shared::event_util::UpdatedEvent,
    system::{
        entity::{role::Role, user::User},
        value_object::user_id::UserId,
    },
};
#[derive(Debug, Clone)]
pub enum SystemEvent {
    UsersCreated {
        items: Vec<User>,
    },
    UsersUpdated {
        items: Vec<UpdatedEvent<User>>,
    },
    UsersDeleted {
        items: Vec<User>,
    },
    UserLoginSucceeded {
        id: UserId,
    },
    UserLogoutSucceeded {
        id: UserId,
    },
    UserRefreshTokenSucceeded {
        id: UserId,
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
    FilesCreated {
        items: Vec<File>,
    },
    FilesUpdated {
        items: Vec<UpdatedEvent<File>>,
    },
    FilesDeleted {
        items: Vec<File>,
    },
    SchedsCreated {
        items: Vec<Sched>,
    },
    SchedsUpdated {
        items: Vec<UpdatedEvent<Sched>>,
    },
    SchedsDeleted {
        items: Vec<Sched>,
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
