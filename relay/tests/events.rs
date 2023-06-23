#![feature(closure_track_caller)]
#[allow(dead_code)]
mod utils;

use ethereum_relay::downloader::common::event_stream::EventStream;
use ethers::types::{Log, U256};
use futures::StreamExt as _;

#[ignore]
#[tokio::test]
async fn integration_test_http_events() {
    let anvil = utils::get_anvil();
    let provider = utils::get_http_provider(anvil.as_ref());
    let client =
        utils::get_ethers_client(provider.clone(), utils::get_wallet(anvil.as_ref())).await;

    let data: Vec<U256> = Vec::from([
        123.into(),
        643253.into(),
        564.into(),
        2324.into(),
        4356.into(),
    ]);
    let logger = utils::deploy_logger_contract(client.clone()).await;
    let stream = EventStream::new(provider, logger.address()).from_block(0);
    for number in data.clone() {
        logger
            .method::<_, ()>("log", number)
            .expect("log should be a valid function")
            .send()
            .await
            .unwrap()
            .await
            .unwrap();
    }
    let logs: Vec<Log> = stream.poll().await.take(data.len()).collect().await;
    let results: Vec<U256> = logs
        .into_iter()
        .map(|log| U256::from_big_endian(&log.data))
        .collect();
    assert_eq!(results, data);
}

#[ignore]
#[tokio::test]
async fn integration_test_ws_events() {
    let anvil = utils::get_anvil();
    let provider = utils::get_ws_provider(anvil.as_ref()).await;
    let client =
        utils::get_ethers_client(provider.clone(), utils::get_wallet(anvil.as_ref())).await;
    let data: Vec<U256> = Vec::from([
        123.into(),
        643253.into(),
        564.into(),
        2324.into(),
        4356.into(),
    ]);

    let logger = utils::deploy_logger_contract(client.clone()).await;
    for number in data.clone() {
        logger
            .method::<_, ()>("log", number)
            .expect("log should be a valid function")
            .send()
            .await
            .unwrap()
            .await
            .unwrap();
    }
    let stream = EventStream::new(provider, logger.address()).from_block(0);
    let logs: Vec<Log> = stream.watch().await.take(data.len()).collect().await;
    let results: Vec<U256> = logs
        .into_iter()
        .map(|log| U256::from_big_endian(&log.data))
        .collect();
    assert_eq!(results, data);
}
