#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate rmp_serde;
extern crate byteorder;
extern crate indyrs as indy;
extern crate futures;
extern crate indy_sys;

#[macro_use]
mod utils;

use indy::did;
use indy::ErrorCode;
use utils::b58::{FromBase58};
use utils::constants::{
    DID_1,
    SEED_1,
    VERKEY_1,
    METADATA,
    VERKEY_ABV_1
};
use utils::setup::{Setup, SetupConfig};
use utils::wallet::Wallet;

use indy::{INVALID_WALLET_HANDLE, INVALID_POOL_HANDLE};

#[allow(unused_imports)]
use futures::Future;

#[inline]
fn assert_verkey_len(verkey: &str) {
    assert_eq!(32, verkey.from_base58().unwrap().len());
}

#[cfg(test)]
mod create_new_did {
    use super::*;

    #[inline]
    fn assert_did_length(did: &str) {
        assert_eq!(16, did.from_base58().unwrap().len());
    }

    #[test]
    fn create_did_with_empty_json() {
        let wallet = Wallet::new();

        let (did, verkey) = did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();

        assert_did_length(&did);
        assert_verkey_len(&verkey);
    }

    #[test]
    fn create_did_with_seed() {
        let wallet = Wallet::new();

        let config = json!({
            "seed": SEED_1
        }).to_string();

        let (did, verkey) = did::create_and_store_my_did(wallet.handle, &config).wait().unwrap();

        assert_eq!(DID_1, did);
        assert_eq!(VERKEY_1, verkey);
    }

    #[test]
    fn create_did_with_did() {
        let wallet = Wallet::new();

        let config = json!({
            "did": DID_1
        }).to_string();

        let (did, verkey) = did::create_and_store_my_did(wallet.handle, &config).wait().unwrap();

        assert_eq!(DID_1, did);
        assert_ne!(VERKEY_1, verkey);
    }
}

#[cfg(test)]
mod replace_keys_start {
    use super::*;

    #[test]
    fn replace_keys_start() {
        let wallet = Wallet::new();
        let (did, verkey) = did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();

        let new_verkey = did::replace_keys_start(wallet.handle, &did, "{}").wait().unwrap();

        assert_verkey_len(&new_verkey);
        assert_ne!(verkey, new_verkey);
    }

    #[test]
    fn replace_keys_start_with_seed() {
        let wallet = Wallet::new();
        let (did, verkey) = did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();
        let config = json!({"seed": SEED_1}).to_string();

        let new_verkey = did::replace_keys_start(wallet.handle, &did, &config).wait().unwrap();

        assert_eq!(VERKEY_1, new_verkey);
        assert_ne!(verkey, new_verkey);
    }
}

#[cfg(test)]
mod replace_keys_apply {
    use super::*;

    fn setup() -> (Wallet, String, String) {
        let wallet = Wallet::new();
        let (did, verkey) = did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();

        (wallet, did, verkey)
    }

    #[inline]
    fn start_key_replacement(wallet: &Wallet, did: &str) {
        let config = json!({"seed": SEED_1}).to_string();
        did::replace_keys_start(wallet.handle, did, &config).wait().unwrap();
    }

    #[test]
    fn replace_keys_apply() {
        let (wallet, did, verkey) = setup();
        start_key_replacement(&wallet, &did);

        let result = did::replace_keys_apply(wallet.handle, &did).wait();

        assert_eq!((), result.unwrap());

        let new_verkey = did::key_for_local_did(wallet.handle, &did).wait().unwrap();

        assert_eq!(VERKEY_1, new_verkey);
        assert_ne!(verkey, new_verkey);
    }
}

#[cfg(test)]
mod test_get_verkey_local {
    use super::*;

    #[test]
    fn get_verkey_local_my_did() {
        let wallet = Wallet::new();
        let config = json!({"seed": SEED_1}).to_string();
        let (did, verkey) = did::create_and_store_my_did(wallet.handle, &config).wait().unwrap();

        let stored_verkey = did::key_for_local_did(wallet.handle, &did).wait().unwrap();

        assert_eq!(verkey, stored_verkey);
    }
}

#[cfg(test)]
mod test_get_verkey_ledger {
    use super::*;

    #[test]
    fn get_verkey_my_did() {
        let wallet = Wallet::new();
        let (did, verkey) = did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();

        let stored_verkey = did::key_for_did(
            -1,
            wallet.handle,
            &did
        ).wait().unwrap();

        assert_eq!(verkey, stored_verkey);
    }

    #[test]
    fn get_verkey_their_did() {
        let wallet = Wallet::new();
        let config = json!({"did": DID_1, "verkey": VERKEY_1}).to_string();
        did::store_their_did(wallet.handle, &config).wait().unwrap();

        let stored_verkey = did::key_for_did(
            -1,
            wallet.handle,
            DID_1,
        ).wait().unwrap();

        assert_eq!(VERKEY_1, stored_verkey);
    }

    #[test]
    fn get_verkey_not_on_ledger() {
        let wallet = Wallet::new();
        let wallet2 = Wallet::new();
        let setup = Setup::new(&wallet, SetupConfig {
            connect_to_pool: true,
            num_trustees: 0,
            num_users: 0,
            num_nodes: 4
        });
        let pool_handle = setup.pool_handle.unwrap();

        let (did, _verkey) = did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();

        let result = did::key_for_did(
            pool_handle,
            wallet2.handle,
            &did
        ).wait();

        assert_eq!(ErrorCode::WalletItemNotFound, result.unwrap_err().error_code);
    }

    #[test]
    fn get_verkey_on_ledger() {
        let wallet = Wallet::new();
        let wallet2 = Wallet::new();
        let setup = Setup::new(&wallet, SetupConfig {
            connect_to_pool: true,
            num_trustees: 1,
            num_users: 1,
            num_nodes: 4
        });
        let pool_handle = setup.pool_handle.unwrap();
        let user = &setup.users.as_ref().unwrap()[0];

        let ledger_verkey = did::key_for_did(
            pool_handle,
            wallet2.handle,
            &user.did
        ).wait().unwrap();

        assert_eq!(ledger_verkey, user.verkey);
    }
}

#[cfg(test)]
mod test_set_metadata {
    use super::*;
    use indy::INVALID_WALLET_HANDLE;

    #[inline]
    fn setup() -> (Wallet, String) {
        let wallet = Wallet::new();
        let (did, _) = did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();

        (wallet, did)
    }

    #[test]
    fn set_metadata_my_did() {
        let (wallet, did) = setup();

        let result = did::set_did_metadata(wallet.handle, &did, METADATA).wait();
        let metadata = did::get_did_metadata(wallet.handle, &did).wait().unwrap();

        assert_eq!((), result.unwrap());
        assert_eq!(METADATA, metadata);
    }

    #[test]
    fn set_metadata_empty_string() {
        let (wallet, did) = setup();

        let result = did::set_did_metadata(wallet.handle, &did, "").wait();
        let metadata = did::get_did_metadata(wallet.handle, &did).wait().unwrap();

        assert_eq!((), result.unwrap());
        assert_eq!("", metadata);
    }
}

#[cfg(test)]
mod test_get_metadata {
    use super::*;

    #[inline]
    fn setup() -> (Wallet, String) {
        let wallet = Wallet::new();
        let (did, _) = did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();

        (wallet, did)
    }

    #[test]
    fn get_metadata_my_did() {
        let (wallet, did) = setup();
        did::set_did_metadata(wallet.handle, &did, METADATA).wait().unwrap();

        let result = did::get_did_metadata(wallet.handle, &did).wait();

        assert_eq!(METADATA, result.unwrap());
    }
}

#[cfg(test)]
mod test_set_endpoint {
    use super::*;

    #[test]
    pub fn set_endpoint_succeeds() {
        let wallet = Wallet::new();

        let config = json!({
            "seed": SEED_1
        }).to_string();

        let (did, verkey) = did::create_and_store_my_did(wallet.handle, &config).wait().unwrap();

        indy::did::set_endpoint_for_did(wallet.handle, &did, "192.168.1.10", &verkey).wait().unwrap();

    }
}

#[cfg(test)]
mod test_get_endpoint {
    use super::*;

    #[test]
    pub fn get_endpoint_succeeds() {
        let end_point_address = "192.168.1.10";
        let wallet = Wallet::new();

        let config = json!({
            "seed": SEED_1
        }).to_string();

        let (did, verkey) = did::create_and_store_my_did(wallet.handle, &config).wait().unwrap();

        let pool_setup = Setup::new(&wallet, SetupConfig {
            connect_to_pool: false,
            num_trustees: 0,
            num_nodes: 4,
            num_users: 0,
        });

        indy::did::set_endpoint_for_did(wallet.handle, &did, end_point_address, &verkey).wait().unwrap();

        let pool_handle = indy::pool::open_pool_ledger(&pool_setup.pool_name, None).wait().unwrap();
        let mut test_succeeded : bool = false;
        let mut error_code: indy::ErrorCode = indy::ErrorCode::Success;

        match indy::did::get_endpoint_for_did(wallet.handle, pool_handle, &did).wait() {
            Ok(ret_address) => {

                let (address, _) = Some(ret_address).unwrap();

                if end_point_address.to_string() == address {
                    test_succeeded = true;
                }
            },
            Err(ec) => {
                error_code = ec.error_code;
            }
        }

        indy::pool::close_pool_ledger(pool_handle).wait().unwrap();

        if indy::ErrorCode::Success != error_code {
            assert!(false, "get_endpoint_works failed error code {:?}", error_code);
        }

        if false == test_succeeded {
            assert!(false, "get_endpoint_works failed to successfully compare end_point address");
        }
    }
}

#[cfg(test)]
mod test_get_my_metadata {
    use super::*;

    #[test]
    pub fn get_my_metadata_success() {
        let wallet = Wallet::new();

        let (did, _verkey) = did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();

        match did::get_my_did_with_metadata(wallet.handle, &did).wait() {
            Ok(_) => {},
            Err(ec) => {
                assert!(false, "get_my_metadata_success failed with error code {:?}", ec);
            }
        }
    }
}
