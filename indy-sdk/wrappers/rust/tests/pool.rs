#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate rmp_serde;
extern crate byteorder;
extern crate futures;
extern crate indyrs as indy;
#[macro_use]
pub mod utils;

use utils::constants::{DID_1};
use utils::setup::{Setup, SetupConfig};
use indy::ErrorCode;
use indy::pool;
#[allow(unused_imports)]
use futures::Future;

#[cfg(test)]
mod open_pool {
    use super::*;
    use futures::future::Future;

    #[test]
    pub fn open_pool_works() {
        let wallet = utils::wallet::Wallet::new();
        let setup = Setup::new(&wallet, SetupConfig {
            connect_to_pool: false,
            num_trustees: 0,
            num_nodes: 4,
            num_users: 0,
        });

        let pool_handle = indy::pool::open_pool_ledger(&setup.pool_name, None).wait().unwrap();

        indy::pool::close_pool_ledger(pool_handle).wait().unwrap();
    }
}

#[cfg(test)]
mod close_pool {
    use super::*;
    use futures::future::Future;

    #[test]
    pub fn close_pool_works() {
        let wallet = utils::wallet::Wallet::new();
        let setup = Setup::new(&wallet, SetupConfig {
            connect_to_pool: false,
            num_trustees: 0,
            num_nodes: 4,
            num_users: 0,
        });

        let pool_handle = indy::pool::open_pool_ledger(&setup.pool_name, None).wait().unwrap();

        indy::pool::close_pool_ledger(pool_handle).wait().unwrap();
    }
}

#[cfg(test)]
mod test_pool_create_config {
    use super::*;

    use std::fs;
    use utils::file::TempFile;
    use utils::pool::{PoolList, test_pool_name, test_genesis_config};

    #[inline]
    pub fn assert_pool_exists(name: &str) {
        assert!(PoolList::new().pool_exists(name));
    }

    #[inline]
    pub fn assert_pool_not_exists(name: &str) {
        assert!(! PoolList::new().pool_exists(name));
    }

    /*
    Returns the file, otherwise the file would be deleted
    when it goes out of scope.rustc_lsan
    */
    fn invalid_temporary_genesis_config() -> (String, TempFile) {
        let file = TempFile::new(None).unwrap();
        fs::write(&file, b"Some nonsensical data").unwrap();
        let config = json!({"genesis_txn": file.as_ref()}).to_string();

        (config, file)
    }

    #[test]
    /* Create a valid config with custom genesis txn. */
    fn config_with_genesis_txn() {
        let name = test_pool_name();
        let (config, _file) = test_genesis_config();
        let result = pool::create_pool_ledger_config(&name, Some(&config)).wait();

        assert_eq!((), result.unwrap());
        assert_pool_exists(&name);
        pool::delete_pool_ledger(&name).wait().unwrap();
    }
}

#[cfg(test)]
mod test_delete_config {
    use super::*;

    use futures::future::Future;

    use utils::pool::{PoolList, create_default_pool};

    const NON_EXISTENT_NAME: &str = "a_pool_name_which_does_not_exist";

    #[inline]
    pub fn assert_pool_exists(name: &str) {
        assert!(PoolList::new().pool_exists(name));
    }

    #[inline]
    pub fn assert_pool_not_exists(name: &str) {
        assert!(! PoolList::new().pool_exists(name));
    }

    #[test]
    /* Delete a pool_config. */
    fn delete_pool() {
        let pool_name = create_default_pool();
        assert_pool_exists(&pool_name);

        let result = pool::delete_pool_ledger(&pool_name).wait();
        assert_eq!((), result.unwrap());

        assert_pool_not_exists(&pool_name);
    }
}