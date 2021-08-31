#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate rmp_serde;
extern crate byteorder;
extern crate indyrs as indy;
extern crate futures;

#[macro_use]
mod utils;

#[allow(unused_imports)]
use futures::Future;

use utils::wallet::Wallet;
use utils::constants::{DID_TRUSTEE, VERKEY_TRUSTEE, METADATA, DID};

extern crate failure;

use indy::ErrorCode;
use indy::INVALID_WALLET_HANDLE;

mod create_pairwise {
    use super::*;

    #[test]
    pub fn create_pairwise_works() {
        let wallet = Wallet::new();
        let their_identity_json = json!({"did": DID_TRUSTEE, "verkey": VERKEY_TRUSTEE}).to_string();
        indy::did::store_their_did(wallet.handle, &their_identity_json).wait().unwrap();

        let (did, _) = indy::did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();
        indy::pairwise::create_pairwise(wallet.handle, DID_TRUSTEE, &did, Some(METADATA)).wait().unwrap();
    }

    #[test]
    pub fn create_pairwise_works_for_empty_metadata() {
        let wallet = Wallet::new();
        let their_identity_json = json!({"did": DID_TRUSTEE, "verkey": VERKEY_TRUSTEE}).to_string();
        indy::did::store_their_did(wallet.handle, &their_identity_json).wait().unwrap();

        let (did, _) = indy::did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();
        indy::pairwise::create_pairwise(wallet.handle, DID_TRUSTEE, &did, None).wait().unwrap();
    }

}

mod pairwise_exists {
    use super::*;

    #[test]
    pub fn pairwise_exists_works() {
        let wallet = Wallet::new();
        let their_identity_json = json!({"did": DID_TRUSTEE, "verkey": VERKEY_TRUSTEE}).to_string();
        indy::did::store_their_did(wallet.handle, &their_identity_json).wait().unwrap();

        let (did, _) = indy::did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();
        indy::pairwise::create_pairwise(wallet.handle, DID_TRUSTEE, &did, Some(METADATA)).wait().unwrap();

        assert!(indy::pairwise::is_pairwise_exists(wallet.handle, DID_TRUSTEE).wait().unwrap());
    }
}

mod get_pairwise {
    use super::*;

    #[test]
    pub fn get_pairwise_works() {
        let wallet = Wallet::new();
        let their_identity_json = json!({"did": DID_TRUSTEE, "verkey": VERKEY_TRUSTEE}).to_string();
        indy::did::store_their_did(wallet.handle, &their_identity_json).wait().unwrap();

        let (did, _) = indy::did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();
        indy::pairwise::create_pairwise(wallet.handle, DID_TRUSTEE, &did, Some(METADATA)).wait().unwrap();

        let pairwise_info_json = indy::pairwise::get_pairwise(wallet.handle, DID_TRUSTEE).wait().unwrap();

        assert_eq!(format!(r#"{{"my_did":"{}","metadata":"{}"}}"#, did, METADATA), pairwise_info_json);
    }
}

mod set_pairwise_metadata {
    use super::*;

    #[test]
    pub fn set_pairwise_metadata_works() {
        let wallet = Wallet::new();
        let their_identity_json = json!({"did": DID_TRUSTEE, "verkey": VERKEY_TRUSTEE}).to_string();
        indy::did::store_their_did(wallet.handle, &their_identity_json).wait().unwrap();

        let (did, _) = indy::did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();
        indy::pairwise::create_pairwise(wallet.handle, DID_TRUSTEE, &did, None).wait().unwrap();

        let pairwise_info_without_metadata = indy::pairwise::get_pairwise(wallet.handle, DID_TRUSTEE).wait().unwrap();

        assert_eq!(format!(r#"{{"my_did":"{}"}}"#, did), pairwise_info_without_metadata);

        indy::pairwise::set_pairwise_metadata(wallet.handle, DID_TRUSTEE, Some(METADATA)).wait().unwrap();

        let pairwise_info_with_metadata = indy::pairwise::get_pairwise(wallet.handle, DID_TRUSTEE).wait().unwrap();

        assert_ne!(pairwise_info_with_metadata, pairwise_info_without_metadata);
        assert_eq!(format!(r#"{{"my_did":"{}","metadata":"{}"}}"#, did, METADATA), pairwise_info_with_metadata);
    }
}