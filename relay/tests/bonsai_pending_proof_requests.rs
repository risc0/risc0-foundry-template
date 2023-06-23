use bonsai_sdk::client::Client;
use ethereum_relay::uploader::bonsai::pending_proofs::pending_proof_request_future::PendingProofRequest;

#[allow(dead_code)]
mod utils;

#[ignore]
#[tokio::test]
async fn integration_test_bonsai_pending_proof_requests_work() {
    // Mock API server
    let (proof_id, server) = utils::get_test_bonsai_server().await;

    let client = Client::new(server.uri(), "").unwrap();

    let pending_proof_request = PendingProofRequest::new(client, proof_id);
    let completed_proof_response = pending_proof_request.await;
    assert!(completed_proof_response.is_ok());

    let completed_proof_id = completed_proof_response.unwrap();
    assert_eq!(completed_proof_id, proof_id);
}
