use chrono::NaiveDateTime;
use errors::Result;
use models::types::MessageFlag;
pub use self::pg::PgCollector;

mod pg;

static DEFAULT_BATCH_SIZE: usize = 3000;

pub trait Collector {
    fn add_message(&mut self, raw_message: RawMessage) -> Result<()>;
    fn commit(&mut self) -> Result<()>;
}

#[derive(Default)]
pub struct VecCollector {
    pub messages: Vec<RawMessage>
}

impl VecCollector {
    pub fn new() -> VecCollector {
        VecCollector {
            messages: vec!()
        }
    }
}

impl Collector for VecCollector {
    #[allow(unused_variables)]
    fn add_message(&mut self, raw_message: RawMessage) -> Result<()> {
        self.messages.push(raw_message);
        Ok(())
    }

    #[allow(unused_variables)]
    fn commit(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug,Clone)]
pub struct RawMessage {
    pub nick: String,
    pub channel: String,
    pub message: String,
    pub sent_at: NaiveDateTime,
    pub flags: Vec<MessageFlag>
}
