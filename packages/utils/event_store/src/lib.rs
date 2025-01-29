pub mod aggregates;

pub trait FromEventStoreEvent: Sized {
    fn from_eventstore_event(event: &eventstore::ResolvedEvent) -> Self;
    fn from_recorded_event(event: &eventstore::RecordedEvent) -> Self;
}

pub trait FromJson {
    fn from_json(value: serde_json::Value, event_type: &str) -> Self;
}

pub trait ToJson<TResult> {
    fn to_json(value: TResult) -> serde_json::Value;
}

pub fn from_recorded_event<T: FromJson>(event: &eventstore::RecordedEvent) -> T {
    let parsed = serde_json::from_slice::<serde_json::Value>(&event.data)
        .expect("Expected to be able to parse as json");
    T::from_json(parsed, &event.event_type.clone())
}

pub fn from_resolved_event<T: FromJson>(event: &eventstore::ResolvedEvent) -> T {
    let original_event = event.get_original_event();
    from_recorded_event::<T>(original_event)
}
