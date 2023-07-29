use client::Channel;
use gpui::{executor::Deterministic, TestAppContext};
use std::sync::Arc;

use super::TestServer;

#[gpui::test]
async fn test_basic_channels(
    deterministic: Arc<Deterministic>,
    cx_a: &mut TestAppContext,
    cx_b: &mut TestAppContext,
) {
    deterministic.forbid_parking();
    let mut server = TestServer::start(&deterministic).await;
    let client_a = server.create_client(cx_a, "user_a").await;
    let client_b = server.create_client(cx_b, "user_b").await;

    let channel_a_id = client_a
        .channel_store
        .update(cx_a, |channel_store, _| {
            channel_store.create_channel("channel-a", None)
        })
        .await
        .unwrap();

    client_a.channel_store.read_with(cx_a, |channels, _| {
        assert_eq!(
            channels.channels(),
            &[Channel {
                id: channel_a_id,
                name: "channel-a".to_string(),
                parent_id: None,
            }]
        )
    });

    client_b
        .channel_store
        .read_with(cx_b, |channels, _| assert_eq!(channels.channels(), &[]));

    // Invite client B to channel A as client A.
    client_a
        .channel_store
        .update(cx_a, |channel_store, _| {
            channel_store.invite_member(channel_a_id, client_b.user_id().unwrap(), false)
        })
        .await
        .unwrap();

    // Wait for client b to see the invitation
    deterministic.run_until_parked();

    client_b.channel_store.read_with(cx_b, |channels, _| {
        assert_eq!(
            channels.channel_invitations(),
            &[Channel {
                id: channel_a_id,
                name: "channel-a".to_string(),
                parent_id: None,
            }]
        )
    });

    // Client B now sees that they are in channel A.
    client_b
        .channel_store
        .update(cx_b, |channels, _| {
            channels.respond_to_channel_invite(channel_a_id, true)
        })
        .await
        .unwrap();
    client_b.channel_store.read_with(cx_b, |channels, _| {
        assert_eq!(channels.channel_invitations(), &[]);
        assert_eq!(
            channels.channels(),
            &[Channel {
                id: channel_a_id,
                name: "channel-a".to_string(),
                parent_id: None,
            }]
        )
    });
}

// TODO:
// Invariants to test:
// 1. Dag structure is maintained for all operations (can't make a cycle)
// 2. Can't be a member of a super channel, and accept a membership of a sub channel (by definition, a noop)

// #[gpui::test]
// async fn test_block_cycle_creation(deterministic: Arc<Deterministic>, cx: &mut TestAppContext) {
//     // deterministic.forbid_parking();
//     // let mut server = TestServer::start(&deterministic).await;
//     // let client_a = server.create_client(cx, "user_a").await;
//     // let a_id = crate::db::UserId(client_a.user_id().unwrap() as i32);
//     // let db = server._test_db.db();

//     // let zed_id = db.create_root_channel("zed", a_id).await.unwrap();
//     // let first_id = db.create_channel("first", Some(zed_id)).await.unwrap();
//     // let second_id = db
//     //     .create_channel("second_id", Some(first_id))
//     //     .await
//     //     .unwrap();
// }

/*
Linear things:
- A way of expressing progress to the team
- A way for us to agree on a scope
- A way to figure out what we're supposed to be doing

*/
