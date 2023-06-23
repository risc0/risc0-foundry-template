pub mod downloader;
pub mod storage;
pub mod uploader;

use std::sync::Arc;

use bonsai_common_log::setup_heartbeat_log;
use downloader::bonsai::{
    proxy_callback_proof_processor::ProxyCallbackProofRequestProcessor,
    proxy_callback_proof_request_stream::ProxyCallbackProofRequestStream,
};
use ethers::{core::types::Address, prelude::*};
use storage::in_memory::InMemoryStorage;
use tokio::sync::Notify;
use uploader::bonsai::{
    completed_proofs::manager::BonsaiCompleteProofManager,
    pending_proofs::manager::BonsaiPendingProofManager,
};

pub struct Config {
    pub bonsai_url: String,
    pub bonsai_api_key: String,
    pub proxy_address: Address,
    pub log_status_interval: u64,
}

pub async fn run_with_ethers_client<M: Middleware + 'static>(config: Config, ethers_client: Arc<M>)
where
    <M as ethers::providers::Middleware>::Provider: PubsubClient,
    <<M as ethers::providers::Middleware>::Provider as ethers::providers::PubsubClient>::NotificationStream: Sync,
{
    let bonsai_client = Client::new(config.bonsai_url, config.bonsai_api_key)
        .expect("Failed to create Bonsai client.")
        .clone();
    let storage = InMemoryStorage::new();

    // Setup Downloader
    let new_pending_proof_request_notifier = Arc::new(Notify::new());
    let proxy_callback_proof_request_processor = ProxyCallbackProofRequestProcessor::new(
        bonsai_client.clone(),
        storage.clone(),
        Some(new_pending_proof_request_notifier.clone()),
    );

    let downloader = ProxyCallbackProofRequestStream::new(
        ethers_client.clone(),
        config.proxy_address.clone(),
        proxy_callback_proof_request_processor,
    );

    // Setup Uploader
    let new_complete_proof_notifier = Arc::new(Notify::new());

    let uploader_pending_proof_manager = BonsaiPendingProofManager::new(
        bonsai_client.clone(),
        storage.clone(),
        new_pending_proof_request_notifier.clone(),
        new_complete_proof_notifier.clone(),
    );

    let send_batch_notifier = Arc::new(Notify::new());
    let max_batch_size: usize = 3;
    let send_batch_interval = tokio::time::interval(tokio::time::Duration::from_millis(1000));

    let uploader_complete_proof_manager = BonsaiCompleteProofManager::new(
        bonsai_client.clone(),
        storage.clone(),
        new_complete_proof_notifier.clone(),
        send_batch_notifier.clone(),
        max_batch_size,
        config.proxy_address.clone(),
        ethers_client.clone(),
        send_batch_interval,
    );

    // Start everything
    let downloader_handle = tokio::spawn(downloader.run());
    let uploader_pending_proof_manager_handle = tokio::spawn(uploader_pending_proof_manager.run());
    let uploader_complete_proof_manager_handle =
        tokio::spawn(uploader_complete_proof_manager.run());

    setup_heartbeat_log(config.log_status_interval);

    tokio::select! {
        err = downloader_handle => {
            panic!("{}", format!("downloader exited: {:?}", err))
        }
        err = uploader_pending_proof_manager_handle => {
            panic!("{}", format!("pending proof manager exited: {:?}", err))
        }
        err = uploader_complete_proof_manager_handle => {
            panic!("{}", format!("complete proof manager exited: {:?}", err))
        }
    }
}
