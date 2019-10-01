use node_runtime::{AccountId, BridgeCall, Call, UncheckedExtrinsic};
use parity_codec::{Compact, Encode};
use primitives::{H160, H256};
use rustc_hex::ToHex;
use substrate_api_client::{hexstr_to_u256, Api};

use primitives::{blake2_256, crypto::Pair, hexdisplay::HexDisplay, sr25519};
use runtime_primitives::generic::Era;

pub fn build_mint(
    sub_api: &Api,
    signer: sr25519::Pair,
    message_id: H256,
    from: H160,
    to: AccountId,
    amount: u64,
) -> String {
    let signer_index = signer_index(sub_api, &signer);
    let genesis_hash = sub_api.genesis_hash.expect("can not get genesiss hash");
    let function = Call::Bridge(BridgeCall::multi_signed_mint(message_id, from, to, amount));
    let era = Era::immortal();

    log::debug!("using genesis hash: {:?}", genesis_hash);
    let raw_payload = (Compact(signer_index), function, era, genesis_hash);
    let signature = raw_payload.using_encoded(|payload| {
        if payload.len() > 256 {
            signer.sign(&blake2_256(payload)[..])
        } else {
            log::debug!("signing {}", HexDisplay::from(&payload));
            signer.sign(payload)
        }
    });
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
    let signature = raw_payload.using_encoded(|payload| {
        if payload.len() > 256 {
            signer.sign(&blake2_256(payload)[..])
        } else {
            log::debug!("signing {}", HexDisplay::from(&payload));
            signer.sign(payload)
        }
    });
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
    let signature = raw_payload.using_encoded(|payload| {
        if payload.len() > 256 {
            signer.sign(&blake2_256(payload)[..])
        } else {
            log::debug!("signing {}", HexDisplay::from(&payload));
            signer.sign(payload)
        }
    });
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
