use std::fmt::Debug;

use crate::{
    error::ApplicationError,
    shared::event::{EVENT_BUS, Event},
};

pub struct CommandResult<T, E> {
    pub output: T,
    pub events: Vec<E>,
}

impl<T, E> CommandResult<T, E> {
    pub fn without_events(result: T) -> Self {
        Self {
            output: result,
            events: vec![],
        }
    }

    pub fn with_event(result: T, event: E) -> Self {
        Self {
            output: result,
            events: vec![event],
        }
    }

    pub fn with_events(result: T, events: Vec<E>) -> Self {
        Self {
            output: result,
            events,
        }
    }
}

pub trait CommandHandler: Debug {
    type Command;
    type Output;
    type Event: Into<Event>;
    fn execute(
        &self,
        cmd: Self::Command,
    ) -> impl Future<Output = Result<CommandResult<Self::Output, Self::Event>, ApplicationError>>;

    fn handle(
        &self,
        cmd: Self::Command,
    ) -> impl Future<Output = Result<Self::Output, ApplicationError>> {
        async {
            let CommandResult { output, events } = self.execute(cmd).await?;
            for event in events {
                EVENT_BUS.publish(event.into());
            }
            Ok(output)
        }
    }
}
