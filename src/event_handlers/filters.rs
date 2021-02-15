use super::*;
use crate::{filters::FeroxFilters, CommandSender, FeroxChannel, Joiner};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{
    mpsc::{self, UnboundedReceiver},
    oneshot,
};

#[derive(Debug)]
/// Container for filters transmitter and FeroxFilters object
pub struct FiltersHandle {
    /// FeroxFilters object used across modules to track active filters
    pub data: Arc<FeroxFilters>,

    /// transmitter used to update `data`
    pub tx: CommandSender,
}

/// implementation of FiltersHandle
impl FiltersHandle {
    /// Given an Arc-wrapped FeroxFilters and CommandSender, create a new FiltersHandle
    pub fn new(data: Arc<FeroxFilters>, tx: CommandSender) -> Self {
        Self { data, tx }
    }

    /// Send the given Command over `tx`
    pub fn send(&self, command: Command) -> Result<()> {
        self.tx.send(command)?;
        Ok(())
    }

    /// Sync the handle with the handler
    pub async fn sync(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel::<bool>();
        self.send(Command::Sync(tx))?;
        rx.await?;
        Ok(())
    }
}

/// event handler for updating a single data structure of all active filters
#[derive(Debug)]
pub struct FiltersHandler {
    /// collection of FeroxFilters
    data: Arc<FeroxFilters>,

    /// Receiver half of mpsc from which `Command`s are processed
    receiver: UnboundedReceiver<Command>,
}

/// implementation of event handler for filters
impl FiltersHandler {
    /// create new event handler
    pub fn new(data: Arc<FeroxFilters>, receiver: UnboundedReceiver<Command>) -> Self {
        Self { data, receiver }
    }

    /// Initialize new `FeroxFilters` and the sc side of an mpsc channel that is responsible for
    /// updates to the aforementioned object.
    pub fn initialize() -> (Joiner, FiltersHandle) {
        log::trace!("enter: initialize");

        let data = Arc::new(FeroxFilters::default());
        let (tx, rx): FeroxChannel<Command> = mpsc::unbounded_channel();

        let mut handler = Self::new(data.clone(), rx);

        let task = tokio::spawn(async move { handler.start().await });

        let event_handle = FiltersHandle::new(data, tx);

        log::trace!("exit: initialize -> ({:?}, {:?})", task, event_handle);

        (task, event_handle)
    }

    /// Start a single consumer task (sc side of mpsc)
    ///
    /// The consumer simply receives `Command` and acts accordingly
    pub async fn start(&mut self) -> Result<()> {
        log::trace!("enter: start({:?})", self);

        while let Some(command) = self.receiver.recv().await {
            match command {
                Command::AddFilter(filter) => {
                    self.data.push(filter)?;
                }
                Command::Sync(sender) => {
                    log::debug!("filters: {:?}", self);
                    sender.send(true).unwrap_or_default();
                }
                Command::Exit => break,
                _ => {} // no other commands needed for FilterHandler
            }
        }

        log::trace!("exit: start");
        Ok(())
    }
}
