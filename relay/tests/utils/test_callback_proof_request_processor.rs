use bonsai_proxy_contract::CallbackRequestFilter;
use ethereum_relay::downloader::common::event_processor::{EventProcessor, EventProcessorError};
use ethers::prelude::*;
use snafu::prelude::*;

#[derive(Debug, Snafu, PartialEq)]
pub enum Error {
    TerminateSuccess,
}

pub struct TestAddressTopicCallbackProofRequestProcessor {
    pub expected_account: Address,
    pub expected_image_id: H256,
    pub expected_input: Bytes,
    pub expected_callback_contract: Address,
}

#[async_trait::async_trait]
impl EventProcessor for TestAddressTopicCallbackProofRequestProcessor {
    type Event = CallbackRequestFilter;

    async fn process_event(&self, event: CallbackRequestFilter) -> Result<(), EventProcessorError> {
        assert_eq!(event.account, self.expected_account);
        assert_eq!(H256::from(event.image_id), self.expected_image_id);
        assert_eq!(event.input, self.expected_input);
        assert_eq!(event.callback_contract, self.expected_callback_contract);

        // Throwing an error signals for the runner loop to stop.
        Err(EventProcessorError::ProcessEventError {
            source: Box::new(Error::TerminateSuccess),
        })
    }
}
