#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate rmp_serde;
extern crate byteorder;
extern crate indyrs as indy;
extern crate futures;

#[allow(unused_variables)]
#[allow(unused_macros)]
#[allow(dead_code)]
#[macro_use]
pub mod utils;

use indy::did;
use indy::ledger;
use indy::pool;
use utils::constants::PROTOCOL_VERSION;
use utils::setup::{Setup, SetupConfig};
use utils::wallet::Wallet;
#[allow(unused_imports)]
use futures::Future;

const REQUEST_JSON: &str = r#"{
                              "reqId":1496822211362017764,
                              "identifier":"GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL",
                              "operation":{
                                   "type":"1",
                                   "dest":"VsKV7grR1BUE29mG2Fm2kX",
                                   "verkey":"GjZWsBLgZCR18aL468JAT7w9CZRiBnpxUPPgyQxh4voa"
                                   },
                              "protocolVersion":2
                          }"#;
#[cfg(test)]
mod test_sign_and_submit_request {

    use super::*;

    #[test]
    pub fn sign_and_submit_request_success() {
        pool::set_protocol_version(PROTOCOL_VERSION as usize).wait().unwrap();

        let wallet = Wallet::new();
        let setup = Setup::new(&wallet, SetupConfig {
            connect_to_pool: false,
            num_trustees: 0,
            num_nodes: 4,
            num_users: 0,
        });

        let pool_handle = pool::open_pool_ledger(&setup.pool_name, None).wait().unwrap();
        let (did, _) = did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();

        let result = ledger::sign_and_submit_request(pool_handle, wallet.handle, &did, REQUEST_JSON).wait();

        pool::close_pool_ledger(pool_handle).wait().unwrap();

        match result {
            Ok(_) => { },
            Err(ec) => { assert!(false, "sign_and_submit_request_success got error code {:?}", ec); },
        }


        /*
         * The format of SignAndSubmitRequestAsync response is like this.
         *
            {"result":{
                "reqSignature":{
                    "type":"ED25519",
                    "values":[{"value":"7kDrVBrmrKAvSs1QoQWYq6F774ZN3bRXx5e3aaUFiNvmh4F1yNqQw1951Az35nfrnGjZ99vtCmSFXZ5GqS1zLiG","from":"V4SGRU86Z58d6TV7PBUe6f"}]
                },
                "txnMetadata":{
                    "txnTime":1536876204,
                    "seqNo":36,
                    "txnId":"5d38ac6a242239c97ee28884c2b5cadec62248b2256bce51afd814c7847a853e"
                },
                "ver":"1",
                "auditPath":["DATtzSu9AMrArv8C2oribQh4wJ6TaD2K9o76t7EL2N7G","AbGuM7s9MudnT8M2eZe1yaG2EGUGxggMXSSbXCm4DFDx","3fjMoUdsbNrRfG5ZneHaQuX994oA4Z2pYPZtRRPmkngw"],
                "rootHash":"A9LirjLuoBT59JJTJYvUgfQyEJA32Wb7njrbD9XqT2wc",
                "txn":{
                    "data":{
                        "dest":"KQRpY4EmSG4MwH7md8gMoN","verkey":"B2nW4JfqZ2omHksoCmwD8zXXmtBsvbQk6WVSboazd8QB"
                    },
                    "protocolVersion":2,
                    "type":"1",
                    "metadata":{
                        "digest":"14594e0b31f751faf72d4bf4abdc6f54af34dab855fe1a0c67fe651b47bb93b5","reqId":1536876205519496000,"from":"V4SGRU86Z58d6TV7PBUe6f"
                    }
                }
            },
            "op":"REPLY"}
        */
    }
}

#[cfg(test)]
mod test_submit_request {
    use super::*;

    #[test]
    pub fn submit_request_success() {
        pool::set_protocol_version(PROTOCOL_VERSION as usize).wait().unwrap();

        let wallet = Wallet::new();
        let setup = Setup::new(&wallet, SetupConfig {
            connect_to_pool: false,
            num_trustees: 0,
            num_nodes: 4,
            num_users: 0,
        });

        let pool_handle = pool::open_pool_ledger(&setup.pool_name, None).wait().unwrap();
        let (_, _) = did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();

        let submit_request_result = ledger::submit_request(pool_handle, REQUEST_JSON).wait();

        pool::close_pool_ledger(pool_handle).wait().unwrap();

        match submit_request_result {
            Ok(submit_request_response) => {
                // return is REQNACK client request invalid: MissingSignature()....this is ok.  we wanted to make sure the function works
                // and getting that response back indicates success
                assert!(submit_request_response.contains("REQNACK"), "submit_request did not return REQNACK => {:?}", submit_request_response);
                assert!(submit_request_response.contains("MissingSignature"), "submit_request did not return MissingSignature => {:?}", submit_request_response);
            },
            Err(ec) => {
                assert!(false, "submit_request failed with {:?}", ec);
            }
        }

    }
}

#[cfg(test)]
mod test_submit_action {
    use super::*;

    const NODES: &str = "[\"Node1\", \"Node2\"]";

    #[test]
    pub fn submit_action_success() {

        pool::set_protocol_version(PROTOCOL_VERSION as usize).wait().unwrap();

        let wallet = Wallet::new();
        let setup = Setup::new(&wallet, SetupConfig {
            connect_to_pool: false,
            num_trustees: 0,
            num_nodes: 4,
            num_users: 0,
        });

        let pool_handle = pool::open_pool_ledger(&setup.pool_name, None).wait().unwrap();

        let (did, _) = did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();
        let validator_request = ledger::build_get_validator_info_request(&did).wait().unwrap();
        let signed_request = ledger::sign_request(wallet.handle, &did, &validator_request).wait().unwrap();

        let result = ledger::submit_action(pool_handle, &signed_request, Some(NODES), Some(5)).wait();

        pool::close_pool_ledger(pool_handle).wait().unwrap();

        match result {
            Ok(_) => {},
            Err(ec) => {
                assert!(false, "submit_action_success failed with {:?} extra {:?}", ec, signed_request);
            }
        }
    }
}

#[cfg(test)]
mod test_sign_request {
    use super::*;

    #[test]
    pub fn sign_request_success() {
        pool::set_protocol_version(PROTOCOL_VERSION as usize).wait().unwrap();

        let wallet = Wallet::new();
        let setup = Setup::new(&wallet, SetupConfig {
            connect_to_pool: false,
            num_trustees: 0,
            num_nodes: 4,
            num_users: 0,
        });

        let pool_handle = pool::open_pool_ledger(&setup.pool_name, None).wait().unwrap();

        let (did, _) = did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();
        let validator_request = ledger::build_get_validator_info_request(&did).wait().unwrap();
        let signed_request_result = ledger::sign_request(wallet.handle, &did, &validator_request).wait();

        pool::close_pool_ledger(pool_handle).wait().unwrap();

        match signed_request_result {
            Ok(_) => {},
            Err(ec) => {
                assert!(false, "sign_request returned error {:?}", ec);
            }
        }
    }
}

#[cfg(test)]
mod test_build_nym_request {
    use super::*;

    use utils::did::NymRole;

    #[test]
    pub fn build_nym_request_success() {
        pool::set_protocol_version(PROTOCOL_VERSION as usize).wait().unwrap();

        let wallet = Wallet::new();
        let (did, verkey) = did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();
        let (trustee_did, _) = did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();

        let nym_result = ledger::build_nym_request(&trustee_did, &did, Some(&verkey), None, NymRole::Trustee.prepare()).wait();

        match nym_result {
            Ok(_) => {},
            Err(ec) => {
                assert!(false, "build_nym_request returned error_code {:?}", ec);
            }
        }

    }

    #[test]
    pub fn build_nym_request_with_no_verkey_success() {
        pool::set_protocol_version(PROTOCOL_VERSION as usize).wait().unwrap();

        let wallet = Wallet::new();
        let (did, _) = did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();
        let (trustee_did, _) = did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();

        let nym_result = ledger::build_nym_request(&trustee_did, &did, None, None, NymRole::Trustee.prepare()).wait();

        match nym_result {
            Ok(_) => {},
            Err(ec) => {
                assert!(false, "build_nym_request returned error_code {:?}", ec);
            }
        }

    }

    #[test]
    pub fn build_nym_request_with_data_success() {
        pool::set_protocol_version(PROTOCOL_VERSION as usize).wait().unwrap();

        let wallet = Wallet::new();
        let (did, _) = did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();
        let (trustee_did, _) = did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();

        let nym_result = ledger::build_nym_request(&trustee_did, &did, None, Some("some_data"), NymRole::Trustee.prepare()).wait();

        match nym_result {
            Ok(_) => {},
            Err(ec) => {
                assert!(false, "build_nym_request returned error_code {:?}", ec);
            }
        }
    }
}

#[cfg(test)]
mod test_build_get_nym_request {
    use super::*;

    #[test]
    pub fn build_get_nym_request_success() {
        pool::set_protocol_version(PROTOCOL_VERSION as usize).wait().unwrap();

        let submitter_wallet = Wallet::new();
        let wallet = Wallet::new();
        let (submitter_did, _) = did::create_and_store_my_did(submitter_wallet.handle, "{}").wait().unwrap();
        let (did, _) = did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();

        let get_result = ledger::build_get_nym_request(Some(&submitter_did), &did).wait();

        match get_result {
            Ok(_) => {},
            Err(ec) => {
                assert!(false, "build_get_nym_request returned error_code {:?}", ec);
            }
        }
    }
}

#[cfg(test)]
mod test_build_get_attrib_request {

    use super::*;

    #[test]
    pub fn build_get_attrib_request_success() {

        let submitter_wallet = Wallet::new();
        let wallet = Wallet::new();
        let f1 = did::create_and_store_my_did(submitter_wallet.handle, "{}");
        let f2 = did::create_and_store_my_did(wallet.handle, "{}");
        f1.join(f2).map(|((submitter_did, _), (did, _))| {
            match ledger::build_get_attrib_request(Some(&submitter_did), &did, Some("{}"), None, None).wait() {
                Ok(_) => {},
                Err(ec) => {
                    assert!(false, "build_attrib_request failed with error {:?}", ec);
                }
            }
        }).wait().unwrap();
    }
}

#[cfg(test)]
mod test_build_schema_request {
    use super::*;

    const SCHEMA_DATA: &str = r#"{"id":"NcYxiDXkpYi6ov5FcYDi1e:2:gvt2:3.1","attrNames": ["name", "male"],"name":"gvt2","version":"3.1","ver":"1.0"}"#;

    #[test]
    pub fn build_schema_request_success() {
        let wallet = Wallet::new();
        let (did, _) = did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();

        match ledger::build_schema_request(&did, SCHEMA_DATA).wait() {
            Ok(_) => {},
            Err(ec) => {
                assert!(false, "build_schema_request failed with error {:?}", ec);
            }
        }
    }
}

#[cfg(test)]
mod test_build_get_schema_request {
    use super::*;

    const SCHEMA_REQUEST: &str = "5LEV4bTAXntXqmtLFm7yCS:2:bob:1.0";

    #[test]
    pub fn build_get_schema_request_success() {
        let wallet = Wallet::new();
        let (did, _) = did::create_and_store_my_did(wallet.handle, "{}").wait().unwrap();


        match ledger::build_get_schema_request(Some(&did), SCHEMA_REQUEST).wait() {
            Ok(_) => {},
            Err(ec) => {
                assert!(false, "build_get_schema_request failed with error {:?}", ec);
            }
        }
    }
}

#[cfg(test)]
mod test_build_auth_rule_request {
    use super::*;

    const DID: &str = "VsKV7grR1BUE29mG2Fm2kX";
    const NYM_AUTH_TYPE: &str = "1";
    const ADD_AUTH_ACTION: &str = "ADD";
    const FIELD: &str = "role";
    const OLD_VALUE: &str = "0";
    const NEW_VALUE: &str = "101";
    const ROLE_CONSTRAINT: &str = r#"{
        "sig_count": 1,
        "metadata": {},
        "role": "0",
        "constraint_id": "ROLE",
        "need_to_be_owner": false
    }"#;

    #[test]
    pub fn build_auth_rule_request_success() {
        let _auth_rule_request = ledger::build_auth_rule_request(DID,
                                                                 NYM_AUTH_TYPE,
                                                                 &ADD_AUTH_ACTION,
                                                                 FIELD,
                                                                 None,
                                                                 Some(NEW_VALUE),
                                                                 ROLE_CONSTRAINT).wait().unwrap();
    }

    #[test]
    pub fn build_get_auth_rule_request_success() {
        let _get_auth_rule_request = ledger::build_get_auth_rule_request(Some(DID),
                                                                         Some(NYM_AUTH_TYPE),
                                                                         Some(ADD_AUTH_ACTION),
                                                                         Some(FIELD),
                                                                         Some(OLD_VALUE),
                                                                         Some(NEW_VALUE)).wait().unwrap();
    }
}