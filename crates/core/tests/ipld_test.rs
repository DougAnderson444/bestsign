#[path = "./fixtures.rs"]
mod fixtures;
use fixtures::{RawBlock, TestKeyManager};

use bestsign_core::ops::config::{LockScript, UnlockScript};
use bestsign_core::ops::create;
use bestsign_core::ops::open::config::NewLogBuilder;
use provenance_log::{Key, Script};

use serde::{Deserialize, Serialize};
use serde_ipld_dagcbor::to_vec;

use blockstore::block::Block;
use blockstore::{Blockstore, InMemoryBlockstore};
use cid::Cid;

// create a plog, and test the IPLD functionality of the fields that are Cid
#[tokio::test]
async fn test_ipld() {
    let lock_str = r#"
            check_signature("/recoverykey", "/entry/") ||
            check_signature("/pubkey", "/entry/") ||
            check_preimage("/hash")
        "#;

    let lock_script = Script::Code(Key::default(), lock_str.to_string());

    let unlock_str = r#"
                push("/entry/");
                push("/entry/proof");
            "#;

    let unlock_script = Script::Code(Key::default(), unlock_str.to_string());

    let config = NewLogBuilder::new(LockScript(lock_script), UnlockScript(unlock_script))
        .try_build()
        .unwrap();

    let mut key_manager = TestKeyManager::new();

    let plog = create(&config, &mut key_manager).unwrap();

    let head_cid: Vec<u8> = plog.head.clone().into();

    // convert multicid into Cid
    let head_cid_converted = Cid::try_from(head_cid.clone()).unwrap();

    // bytes should match
    assert_eq!(head_cid_converted.to_bytes(), head_cid.clone());

    let plog_bytes: Vec<u8> = plog.into();

    let plog_block = RawBlock(plog_bytes);
    let root_cid = plog_block.cid().unwrap();

    let blockstore = InMemoryBlockstore::<64>::new();
    blockstore.put(plog_block.clone()).await.unwrap();

    let all_blocks = blockstore.get(&root_cid).await.unwrap().unwrap();

    assert_eq!(plog_block.0, all_blocks);

    // check to see if the head cid is able to be retrieved from the blockstore
    // This will only work if you add the Head to the blockstore separately
    // You'd have to traverse the struct
    //let head_block = blockstore.get(&head_cid_converted).await.unwrap().unwrap();

    //assert_eq!(head_block, head_cid);
}

// try to serialize a struct with Cid links that is NOT a Plog, first
#[tokio::test]
async fn test_basic_ipld() {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct MyIpld {
        link_1: Cid,
        link_2: Cid,
    }

    let blockstore = InMemoryBlockstore::<64>::new();

    let link_1_data = b"link_1".to_vec();
    let link_2_data = b"link_2".to_vec();

    blockstore.put(RawBlock(link_1_data.clone())).await.unwrap();
    blockstore.put(RawBlock(link_2_data.clone())).await.unwrap();

    let link_1 = RawBlock(link_1_data.clone()).cid().unwrap();
    let link_2 = RawBlock(link_2_data.clone()).cid().unwrap();

    let my_ipld = MyIpld { link_1, link_2 };

    let my_ipld_bytes = to_vec(&my_ipld).unwrap();

    let my_ipld_block = RawBlock(my_ipld_bytes.clone());

    let root_cid = my_ipld_block.cid().unwrap();

    blockstore.put(my_ipld_block.clone()).await.unwrap();

    let all_blocks = blockstore.get(&root_cid).await.unwrap().unwrap();

    assert_eq!(my_ipld_bytes, all_blocks);

    let head_block = blockstore.get(&link_1).await.unwrap().unwrap();

    assert_eq!(link_1_data, head_block);
}
