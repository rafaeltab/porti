use std::error::Error;

pub trait DomainEvent<TRoot> {
    fn get_event_type(&self) -> &'static str;
    fn apply(&self, root: &mut TRoot);
    fn get_aggregate_id(&self) -> &u64;
}

pub trait DomainError: Error {}

pub struct Aggregate<TEvent, TRoot>
where
    TEvent: DomainEvent<TRoot>,
    TRoot: Default,
{
    pub source_events: Vec<TEvent>,
    pub draft_events: Vec<TEvent>,
    pub root: TRoot,
    pub latest_revision: u64,
}

impl<TEvent, TRoot> Aggregate<TEvent, TRoot>
where
    TEvent: DomainEvent<TRoot>,
    TRoot: Default,
{
    pub fn add_event(&mut self, event: TEvent) {
        event.apply(&mut self.root);

        self.draft_events.push(event);
    }

    pub fn from_events(events: Vec<TEvent>, latest_revision: u64) -> Self {
        let mut initial = TRoot::default();

        for event in &events {
            event.apply(&mut initial);
        }

        Self {
            draft_events: vec![],
            source_events: events,
            root: initial,
            latest_revision,
        }
    }
}
