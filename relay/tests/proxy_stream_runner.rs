#[allow(dead_code)]
mod utils;

use ethereum_relay::downloader::{
    bonsai::proxy_callback_proof_request_stream::{
        ProxyCallbackProofRequestStream, ProxyRunnerError,
    },
    common::event_processor::EventProcessorError,
};
use ethers::types::{Bytes, H160, H256};

#[ignore]
#[tokio::test]
async fn integration_test_proxy_stream_runner() {
    let anvil = utils::get_anvil();
    let client = utils::get_ethers_client(
        utils::get_ws_provider(anvil.as_ref()).await,
        utils::get_wallet(anvil.as_ref()),
    )
    .await;

    let proxy = utils::deploy_proxy_contract(client.clone()).await;

    // create an event
    let fake_image_id: [u8; 32] =
        hex::decode("de11d9df3349a60bec5cd7271dc81d4f8c7089921b0b645f21d333e818b14d32")
            .expect("should be valid hex")
            .try_into()
            .expect("should be 32 bytes");

    let fake_address: [u8; 20] = hex::decode("9ebda139eba69a5d232828bfd551ab80d0cebf05")
        .expect("should be valid hex")
        .try_into()
        .expect("should be 20 bytes");

    let processor = utils::test_callback_proof_request_processor::TestAddressTopicCallbackProofRequestProcessor {
        expected_account: client.address(),
        expected_image_id: H256::from(fake_image_id),
        expected_input: Bytes::from("hello world".to_string().as_bytes().to_vec()),
        expected_callback_contract: H160::from(fake_address),
    };

    let runner = ProxyCallbackProofRequestStream::new(client.clone(), proxy.address(), processor);
    let runner_handle = tokio::spawn(runner.run());

    let function_selector: [u8; 4] = [0xab, 0xcd, 0xef, 0xab];
    let gas_limit: u64 = 3000000;
    proxy
        .method::<_, ()>(
            "request_callback",
            (
                H256::from(fake_image_id),
                Bytes::from("hello world".to_string().as_bytes().to_vec()),
                H160::from(fake_address),
                function_selector,
                gas_limit,
            ),
        )
        .expect("request_callback should be a valid function")
        .send()
        .await
        .unwrap()
        .await
        .unwrap();

    assert!(match runner_handle
        .await
        .expect("tokio task should have succeeded")
    {
        Err(ProxyRunnerError::LoopTerminatedError {
            source: EventProcessorError::ProcessEventError { source },
        }) => {
            *(source
                .downcast::<utils::test_callback_proof_request_processor::Error>()
                .expect("should be a valid downcast"))
                == utils::test_callback_proof_request_processor::Error::TerminateSuccess
        }
        _ => false,
    });
}
