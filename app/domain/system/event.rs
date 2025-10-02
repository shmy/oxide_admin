use crate::shared::event_util::UpdatedEvent;
use crate::system::entity::file::File;
use crate::system::entity::sched::Sched;
#[derive(Debug, Clone)]
pub enum SystemEvent {
    FilesCreated { items: Vec<File> },
    FilesUpdated { items: Vec<UpdatedEvent<File>> },
    FilesDeleted { items: Vec<File> },
    SchedsCreated { items: Vec<Sched> },
    SchedsUpdated { items: Vec<UpdatedEvent<Sched>> },
    SchedsDeleted { items: Vec<Sched> },
}
