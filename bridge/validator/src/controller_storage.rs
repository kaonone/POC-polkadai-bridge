use web3::types::H256;

use std::collections::HashMap;
use std::iter::Iterator;

use crate::controller::Event;

#[derive(Debug)]
pub struct ControllerStorage {
    events: HashMap<H256, Event>,
    events_queue: Vec<Event>,
}

#[derive(Debug)]
pub enum Error {
    Duplicate,
}

impl ControllerStorage {
    pub fn new() -> Self {
        ControllerStorage {
            events: HashMap::new(),
            events_queue: Vec::new(),
        }
    }

    pub fn put_event(&mut self, event: &Event) -> Result<(), Error> {
        let message_id = event.message_id();
        match self.events.get(message_id) {
            Some(e) if e == event => Err(Error::Duplicate),
            _ => {
                self.events.insert(*message_id, event.clone());
                Ok(())
            }
        }
    }

    pub fn put_event_to_queue(&mut self, event: Event) {
        self.events_queue.push(event)
    }

    pub fn iter_events_queue(&self) -> impl Iterator<Item = &Event> {
        self.events_queue.iter()
    }

    pub fn clear_events_queue(&mut self) {
        self.events_queue.clear();
    }
}
