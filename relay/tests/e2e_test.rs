// Copyright 2023 Risc0, Inc.
//
// All rights reserved.

use std::time::SystemTime;

use bonsai_elfs::{TRUE_ELF, TRUE_ID};
use bonsai_proxy_contract::ProxyContract;
use bonsai_sdk::types::{ImageID, H256};
use ethereum_relay::{run_with_ethers_client, Config};
use ethers::types::{Bytes, H160, H256 as ethers_H256, U256};
use risc0_zkvm::{prove::get_prover, serde::to_vec, Executor, ExecutorEnv};
use tokio::time::{sleep, Duration};

#[allow(dead_code)]
mod utils;

#[tokio::test]
#[ignore]
async fn e2e_test_counter_true_predicate() {
    // deploy the contracts
    let anvil = utils::get_anvil();
    let ethers_client = utils::get_ethers_client(
        utils::get_ws_provider(anvil.as_ref()).await,
        utils::get_wallet(anvil.as_ref()),
    )
    .await;
    let proxy = ProxyContract::deploy(ethers_client.clone(), H160::default())
        .expect("should be able to deploy the proxy contract")
        .send()
        .await
        .expect("deployment should succeed");
    let counter = utils::deploy_counter_contract(ethers_client.clone()).await;
    assert_eq!(
        counter
            .method::<_, U256>("value", ())
            .expect("value should be a function")
            .call()
            .await
            .expect("a call to value should succeed"),
        U256::from(0),
    );

    // run the bonsai relayer
    let config = Config {
        bonsai_url: utils::get_bonsai_url(),
        bonsai_api_key: utils::get_api_key(),
        proxy_address: proxy.address(),
        log_status_interval: 1,
    };

    dbg!("starting bonsai relayer");
    tokio::spawn(run_with_ethers_client(config, ethers_client.clone()));

    // register elf
    dbg!(H256::from_full_digest_words(TRUE_ID));
    let bonsai_client = utils::get_bonsai_client(utils::get_api_key());
    let image_id = bonsai_client
        .put_image_from_elf(TRUE_ELF)
        .await
        .expect("elf was not registered");
    assert_eq!(
        image_id.image_id.as_ref(),
        H256::from_full_digest_words(TRUE_ID).as_ref()
    );

    // Since we are using the True Elf, the first 4 bytes need to be the length
    // of the slice (in little endian)
    let mut input = vec![0; 36];
    input[0] = 32;
    input[35] = 100;

    // Invoke the Counter contract which should request a callback through bonsai
    let gas_limit: u64 = 3000000;
    let image_id_bytes: [u8; 32] = image_id.image_id.into();
    counter
        .method::<_, ()>(
            "request_callback",
            (
                ethers_H256::from(image_id_bytes),
                Bytes::from(input),
                gas_limit,
                proxy.address(),
            ),
        )
        .expect("request_callback should be a function")
        .send()
        .await
        .expect("request_callback should succeed");

    let now = SystemTime::now();
    let max_seconds_to_wait = 120;
    let expected_value = U256::from(100);
    while now.elapsed().expect("error occured getting time").as_secs() < max_seconds_to_wait {
        let value = counter
            .method::<_, U256>("value", ())
            .expect("value should be a function")
            .call()
            .await
            .expect("a call to value should succeed");

        if value == expected_value {
            // noticed in dev e2e tests, this condition returns true but the
            // assertion at the end of the test fails. I believe this is because
            // Infura does not ask the same node that returned the value for this
            // call. Adding a sleep of 5 seconds to allow for nodes to sync and
            // catch up.
            dbg!("Success! Waiting 5 seconds for nodes to catch up...");
            sleep(Duration::new(5, 0)).await;
            break;
        }

        dbg!(
            format!("waiting {max_seconds_to_wait} seconds for bridge to finish"),
            now.elapsed().expect("valid time").as_secs(),
        );
        sleep(Duration::new(1, 0)).await
    }
}

#[tokio::test]
#[ignore]
async fn e2e_test_counter_true_predicate_private() {
    // deploy the contracts
    let anvil = utils::get_anvil();
    let ethers_client = utils::get_ethers_client(
        utils::get_ws_provider(anvil.as_ref()).await,
        utils::get_wallet(anvil.as_ref()),
    )
    .await;
    let proxy = ProxyContract::deploy(ethers_client.clone(), H160::default())
        .expect("should be able to deploy the proxy contract")
        .send()
        .await
        .expect("deployment should succeed");
    let counter = utils::deploy_counter_contract(ethers_client.clone()).await;
    assert_eq!(
        counter
            .method::<_, U256>("value", ())
            .expect("value should be a function")
            .call()
            .await
            .expect("a call to value should succeed"),
        U256::from(0),
    );

    // run the bonsai relayer
    let config = Config {
        bonsai_url: utils::get_bonsai_url(),
        bonsai_api_key: utils::get_api_key(),
        proxy_address: proxy.address(),
        log_status_interval: 1,
    };

    dbg!("starting bonsai relayer");
    tokio::spawn(run_with_ethers_client(config, ethers_client.clone()));

    // compute proof
    // Since we are using the True Elf, the first 4 bytes need to be the length
    // of the slice (in little endian)
    let mut input = vec![0; 36];
    input[0] = 32;
    input[35] = 100;

    let env = ExecutorEnv::builder()
        .add_input(&to_vec(&input).unwrap())
        .build();
    let mut exec = Executor::from_elf(env, TRUE_ELF).unwrap();
    let session = exec.run().unwrap();
    let prover = get_prover("$poseidon");
    let session_receipt = prover.prove_session(&session).unwrap();

    // submit proof
    dbg!(H256::from_full_digest_words(TRUE_ID));
    let bonsai_client = utils::get_bonsai_client(utils::get_api_key());
    let proof_id = bonsai_client
        .post_private_proof(ImageID::from_full_digest_words(TRUE_ID), session_receipt)
        .await
        .expect("Failed to submit private proof");
    dbg!(&proof_id);

    // Invoke the Counter contract which should request a callback through bonsai
    let gas_limit: u64 = 3000000;
    counter
        .method::<_, ()>(
            "request_callback",
            (
                ethers_H256::from([0u8; 32]),
                Bytes::from(proof_id.as_bytes()),
                gas_limit,
                proxy.address(),
            ),
        )
        .expect("request_callback should be a function")
        .send()
        .await
        .expect("request_callback should succeed");

    let now = SystemTime::now();
    let max_seconds_to_wait = 120;
    let expected_value = U256::from(100);
    while now
        .elapsed()
        .expect("error occurred getting time")
        .as_secs()
        < max_seconds_to_wait
    {
        let value = counter
            .method::<_, U256>("value", ())
            .expect("value should be a function")
            .call()
            .await
            .expect("a call to value should succeed");

        if value == expected_value {
            // noticed in dev e2e tests, this condition returns true but the
            // assertion at the end of the test fails. I believe this is because
            // Infura does not ask the same node that returned the value for this
            // call. Adding a sleep of 5 seconds to allow for nodes to sync and
            // catch up.
            dbg!("Success! Waiting 5 seconds for nodes to catch up...");
            sleep(Duration::new(5, 0)).await;
            break;
        }

        dbg!(
            format!("waiting {max_seconds_to_wait} seconds for bridge to finish"),
            now.elapsed().expect("valid time").as_secs(),
        );
        sleep(Duration::new(1, 0)).await
    }
}
