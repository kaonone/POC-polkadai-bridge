use web3::types::H256;

use std::collections::HashMap;

use crate::controller::Events;

#[derive(Debug)]
pub struct ControllerStorage {
    events: HashMap<H256, Events>,
}

#[derive(Debug)]
pub enum Error {
    Duplicate,
}

impl ControllerStorage {
    pub fn new() -> Self {
        ControllerStorage {
            events: HashMap::new(),
        }
    }

    pub fn put_event(&mut self, event: &Events) -> Result<(), Error> {
        let message_id = event.message_id();
        match self.events.get(message_id) {
            Some(e) if e == event => Err(Error::Duplicate),
            _ => {
                self.events.insert(*message_id, event.clone());
                Ok(())
            }
        }
    }
}
