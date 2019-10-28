use node_runtime::{AccountId, BridgeCall, Call, UncheckedExtrinsic};
use parity_codec::{Compact, Encode};
use primitives::{H160, H256};
use rustc_hex::ToHex;
use substrate_api_client::{hexstr_to_u256, Api};

use node_runtime;
use primitives::{blake2_256, crypto::Pair, hexdisplay::HexDisplay, sr25519};
use runtime_primitives::generic::Era;

pub fn mint(
    sub_api: &Api,
    signer_mnemonic_phrase: String,
    message_id: primitives::H256,
    from: primitives::H160,
    to: AccountId,
    amount: u128,
) {
    let xthex = build_mint(
        &sub_api,
        get_sr25519_pair(&signer_mnemonic_phrase),
        message_id,
        from,
        to,
        amount,
    );
    //send and watch extrinsic until finalized
    let _tx_hash = sub_api.send_extrinsic(xthex);
}

pub fn approve_transfer(
    sub_api: &Api,
    signer_mnemonic_phrase: String,
    message_id: primitives::H256,
) {
    let xthex = build_approve_transfer(
        &sub_api,
        get_sr25519_pair(&signer_mnemonic_phrase),
        message_id,
    );
    //send and watch extrinsic until finalized
    let _tx_hash = sub_api.send_extrinsic(xthex);
}

pub fn cancel_transfer(
    sub_api: &Api,
    signer_mnemonic_phrase: String,
    message_id: primitives::H256,
) {
    let xthex = build_cancel_transfer(
        &sub_api,
        get_sr25519_pair(&signer_mnemonic_phrase),
        message_id,
    );
    //send and watch extrinsic until finalized
    let _tx_hash = sub_api.send_extrinsic(xthex);
}

pub fn confirm_transfer(
    sub_api: &Api,
    signer_mnemonic_phrase: String,
    message_id: primitives::H256,
) {
    let xthex = build_confirm_transfer(
        &sub_api,
        get_sr25519_pair(&signer_mnemonic_phrase),
        message_id,
    );
    //send and watch extrinsic until finalized
    let _tx_hash = sub_api.send_extrinsic(xthex);
}

fn get_sr25519_pair(signer_mnemonic_phrase: &str) -> sr25519::Pair {
    sr25519::Pair::from_phrase(signer_mnemonic_phrase, None).expect("invalid menemonic phrase")
}

pub fn build_mint(
    sub_api: &Api,
    signer: sr25519::Pair,
    message_id: H256,
    from: H160,
    to: AccountId,
    amount: u128,
) -> String {
    let signer_index = signer_index(sub_api, &signer);
    let genesis_hash = sub_api.genesis_hash.expect("can not get genesiss hash");
    let function = Call::Bridge(BridgeCall::multi_signed_mint(message_id, from, to, amount));
    let era = Era::immortal();

    log::debug!("using genesis hash: {:?}", genesis_hash);
    let raw_payload = (Compact(signer_index), function, era, genesis_hash);
    let signature = sign_raw_payload(&raw_payload, &signer);
    let ext = UncheckedExtrinsic::new_signed(
        signer_index,
        raw_payload.1,
        signer.public().into(),
        signature,
        era,
    );

    log::debug!("extrinsic: {:?}", ext);

    let mut xthex: String = ext.encode().to_hex();
    xthex.insert_str(0, "0x");
    xthex
}

pub fn build_approve_transfer(sub_api: &Api, signer: sr25519::Pair, message_id: H256) -> String {
    let signer_index = signer_index(sub_api, &signer);
    let genesis_hash = sub_api.genesis_hash.expect("can not get genesiss hash");
    let function = Call::Bridge(BridgeCall::approve_transfer(message_id));
    let era = Era::immortal();

    log::debug!("using genesis hash: {:?}", genesis_hash);
    let raw_payload = (Compact(signer_index), function, era, genesis_hash);
    let signature = sign_raw_payload(&raw_payload, &signer);
    let ext = UncheckedExtrinsic::new_signed(
        signer_index,
        raw_payload.1,
        signer.public().into(),
        signature,
        era,
    );

    log::debug!("extrinsic: {:?}", ext);

    let mut xthex: String = ext.encode().to_hex();
    xthex.insert_str(0, "0x");
    xthex
}

pub fn build_cancel_transfer(sub_api: &Api, signer: sr25519::Pair, message_id: H256) -> String {
    let signer_index = signer_index(sub_api, &signer);
    let genesis_hash = sub_api.genesis_hash.expect("can not get genesiss hash");
    let function = Call::Bridge(BridgeCall::cancel_transfer(message_id));
    let era = Era::immortal();

    log::debug!("using genesis hash: {:?}", genesis_hash);
    let raw_payload = (Compact(signer_index), function, era, genesis_hash);
    let signature = sign_raw_payload(&raw_payload, &signer);
    let ext = UncheckedExtrinsic::new_signed(
        signer_index,
        raw_payload.1,
        signer.public().into(),
        signature,
        era,
    );

    log::debug!("extrinsic: {:?}", ext);

    let mut xthex: String = ext.encode().to_hex();
    xthex.insert_str(0, "0x");
    xthex
}

pub fn build_confirm_transfer(sub_api: &Api, signer: sr25519::Pair, message_id: H256) -> String {
    let signer_index = signer_index(sub_api, &signer);
    let genesis_hash = sub_api.genesis_hash.expect("can not get genesiss hash");
    let function = Call::Bridge(BridgeCall::confirm_transfer(message_id));
    let era = Era::immortal();

    log::debug!("using genesis hash: {:?}", genesis_hash);
    let raw_payload = (Compact(signer_index), function, era, genesis_hash);
    let signature = sign_raw_payload(&raw_payload, &signer);
    let ext = UncheckedExtrinsic::new_signed(
        signer_index,
        raw_payload.1,
        signer.public().into(),
        signature,
        era,
    );

    log::debug!("extrinsic: {:?}", ext);

    let mut xthex: String = ext.encode().to_hex();
    xthex.insert_str(0, "0x");
    xthex
}

fn signer_index(sub_api: &Api, signer: &sr25519::Pair) -> u64 {
    let account_id = signer.public();
    let result_str = sub_api
        .get_storage("System", "AccountNonce", Some(account_id.encode()))
        .expect("can not read account nonce");
    let nonce = hexstr_to_u256(result_str);
    nonce.low_u64()
}

type RawPayload = (Compact<u64>, node_runtime::Call, Era, H256);

fn sign_raw_payload(raw_payload: &RawPayload, signer: &sr25519::Pair) -> sr25519::Signature {
    raw_payload.using_encoded(|payload| {
        if payload.len() > 256 {
            signer.sign(&blake2_256(payload)[..])
        } else {
            log::debug!("signing {}", HexDisplay::from(&payload));
            signer.sign(payload)
        }
    })
}
