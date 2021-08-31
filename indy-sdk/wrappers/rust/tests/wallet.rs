extern crate indyrs as indy;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate rmp_serde;
extern crate byteorder;
extern crate futures;
extern crate indy_sys;

use indy::did;
use indy::wallet;

use indy::ErrorCode;

use std::path::{Path, PathBuf};

mod utils;

use utils::constants::{DEFAULT_CREDENTIALS, METADATA};
use utils::file::{TempDir, TempFile};
use utils::rand;
#[allow(unused_imports)]
use futures::Future;

const EXPORT_KEY: &str = "TheScythesHangInTheAppleTrees";


mod wallet_config {
    use super::*;
    
    #[inline]
    pub fn new() -> String {
        json!({
            "id": rand::random_string(20)
        }).to_string()
    }

    #[inline]
    pub fn with_storage(storage: &str) -> String {
        json!({
            "id": rand::random_string(20),
            "storage_type": storage,
        }).to_string()
    }

    #[inline]
    pub fn with_custom_path<P: AsRef<Path>>(path: P) -> String {
        json!({
            "id": rand::random_string(20),
            "storage_type": "default",
            "storage_config": {
                "path": path.as_ref().to_str()
            }
        }).to_string()
    }

    pub mod export {
        use super::*;
        
        #[inline]
        pub fn new<P: AsRef<Path>>(path: P, key: &str) -> String {
            json!({
                "path": path.as_ref(),
                "key": key
            }).to_string()
        }

        pub fn with_defaults() -> (String, PathBuf, TempDir) {
            let dir = TempDir::new(None).unwrap();
            let path = dir.as_ref().join("wallet_export");
            let config = wallet_config::export::new(&path, EXPORT_KEY);

            (config, path, dir)
        }
    }
}



#[cfg(test)]
mod test_wallet_register {
    // Future work
}

#[cfg(test)]
mod test_wallet_create {
    use super::*;
    const CREDENTIALS: &str = r#"{"key":"9DXvkIMD7iSgD&RT$XYjHo0t"}"#;
    use futures::Future;

    #[test]
    fn create_default_wallet() {
        let config = wallet_config::with_storage("default");
        
        let result = wallet::create_wallet(&config, CREDENTIALS).wait();

        assert_eq!((), result.unwrap());

        wallet::delete_wallet(&config, CREDENTIALS).wait().unwrap();
    }
}


#[cfg(test)]
mod test_wallet_delete {
    use super::*;
    use futures::Future;

    #[inline]
    fn assert_wallet_deleted(config: &str, credentials: &str) {
        let result = wallet::open_wallet(config, credentials).wait();
        assert_eq!(ErrorCode::WalletNotFoundError, result.unwrap_err().error_code);
    }

    #[test]
    fn delete_wallet_opened() {
        let config = wallet_config::new();

        wallet::create_wallet(&config, DEFAULT_CREDENTIALS).wait().unwrap();
        let handle = wallet::open_wallet(&config, DEFAULT_CREDENTIALS).wait().unwrap();

        let result = wallet::delete_wallet(&config, DEFAULT_CREDENTIALS).wait();
        
        assert_eq!(ErrorCode::CommonInvalidState, result.unwrap_err().error_code);

        wallet::close_wallet(handle).wait().unwrap();
        wallet::delete_wallet(&config, DEFAULT_CREDENTIALS).wait().unwrap();
    }
}

#[cfg(test)]
mod test_wallet_open {
    use super::*;
    
    #[test]
    fn open_wallet() {
        let config = wallet_config::new();
        wallet::create_wallet(&config, DEFAULT_CREDENTIALS).wait().unwrap();

        let handle = wallet::open_wallet(&config, DEFAULT_CREDENTIALS).wait().unwrap();

        wallet::close_wallet(handle).wait().unwrap();
        wallet::delete_wallet(&config, DEFAULT_CREDENTIALS).wait().unwrap();
    }

}

#[cfg(test)]
mod test_wallet_close {
    use super::*;
    use indy::INVALID_WALLET_HANDLE;

    #[test]
    fn close_wallet() {
        let config = wallet_config::new();
        wallet::create_wallet(&config, DEFAULT_CREDENTIALS).wait().unwrap();
        let handle = wallet::open_wallet(&config, DEFAULT_CREDENTIALS).wait().unwrap();

        let result = wallet::close_wallet(handle).wait();

        assert_eq!((), result.unwrap());
    }
}

#[cfg(test)]
mod test_wallet_export {
    use super::*;
    use indy::INVALID_WALLET_HANDLE;

    #[test]
    fn export_wallet_path_already_exists() {
        let config_wallet = wallet_config::new();
        let file = TempFile::new(None).unwrap();
        let config_export = wallet_config::export::new(&file, EXPORT_KEY);

        wallet::create_wallet(&config_wallet, DEFAULT_CREDENTIALS).wait().unwrap();
        let handle = wallet::open_wallet(&config_wallet, DEFAULT_CREDENTIALS).wait().unwrap();

        let result = wallet::export_wallet(handle, &config_export).wait();

        assert_eq!(ErrorCode::CommonIOError, result.unwrap_err().error_code);
        
        wallet::close_wallet(handle).wait().unwrap();
        wallet::delete_wallet(&config_wallet, DEFAULT_CREDENTIALS).wait().unwrap();
    }
}

#[cfg(test)]
mod test_wallet_import {
    use super::*;

    fn setup_exported_wallet(
        config_wallet: &str,
        credentials: &str,
        config_export: &str
    ) -> (String, String) {
        wallet::create_wallet(&config_wallet, credentials).wait().unwrap();
        let handle = wallet::open_wallet(&config_wallet, credentials).wait().unwrap();

        let (did, _) = did::create_and_store_my_did(handle, "{}").wait().unwrap();
        did::set_did_metadata(handle, &did, METADATA).wait().unwrap();
        let did_with_metadata = did::get_did_metadata(handle, &did).wait().unwrap();

        wallet::export_wallet(handle, &config_export).wait().unwrap();

        wallet::close_wallet(handle).wait().unwrap();
        wallet::delete_wallet(&config_wallet, DEFAULT_CREDENTIALS).wait().unwrap();

        (did, did_with_metadata)
    }

    #[test]
    fn import_wallet() {
        let config_wallet = wallet_config::new();
        let (config_export, _path, _dir) = wallet_config::export::with_defaults();
        let (did, did_with_metadata) = setup_exported_wallet(
            &config_wallet,
            DEFAULT_CREDENTIALS,
            &config_export
        );

        let result = wallet::import_wallet(
            &config_wallet,
            DEFAULT_CREDENTIALS,
            &config_export
        ).wait();

        assert_eq!((), result.unwrap());

        let handle = wallet::open_wallet(&config_wallet, DEFAULT_CREDENTIALS).wait().unwrap();

        let imported_did_with_metadata = did::get_did_metadata(handle, &did).wait().unwrap();

        assert_eq!(did_with_metadata, imported_did_with_metadata);

        wallet::close_wallet(handle).wait().unwrap();
        wallet::delete_wallet(&config_wallet, DEFAULT_CREDENTIALS).wait().unwrap();
    }
}
