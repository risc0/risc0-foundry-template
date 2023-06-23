use snafu::prelude::*;

#[derive(Debug, Snafu)]
pub enum EventProcessorError {
    #[snafu(display("Error processing event: source: {}", source))]
    ProcessEventError {
        source: Box<dyn snafu::Error + Sync + Send>,
    },
}

#[async_trait::async_trait]
pub trait EventProcessor {
    type Event;

    async fn process_event(&self, event: Self::Event) -> Result<(), EventProcessorError>;
}
