use crate::routes::AssetIface;

use super::*;

const TEST_DIR_BASE: &str = "tmp/send_receive/";
const NODE1_PEER_PORT: u16 = 9811;
const NODE2_PEER_PORT: u16 = 9812;

#[serial_test::serial]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[traced_test]
async fn send_receive() {
    initialize();

    let test_dir_node1 = format!("{TEST_DIR_BASE}node1");
    let test_dir_node2 = format!("{TEST_DIR_BASE}node2");
    let (node1_addr, _) = start_node(test_dir_node1, NODE1_PEER_PORT, false).await;
    let (node2_addr, _) = start_node(test_dir_node2, NODE2_PEER_PORT, false).await;

    fund_and_create_utxos(node1_addr).await;
    fund_and_create_utxos(node2_addr).await;

    let asset_id = issue_asset(node1_addr).await;

    let recipient_id = rgb_invoice(node2_addr, None).await.recipient_id;
    send_asset(node1_addr, &asset_id, 400, recipient_id).await;
    mine(false);
    refresh_transfers(node2_addr).await;
    refresh_transfers(node2_addr).await;
    refresh_transfers(node1_addr).await;
    assert_eq!(asset_balance(node1_addr, &asset_id).await, 600);
    assert_eq!(asset_balance(node2_addr, &asset_id).await, 400);

    let RgbInvoiceResponse {
        recipient_id,
        invoice,
        expiration_timestamp: _,
    } = rgb_invoice(node1_addr, Some(asset_id.clone())).await;
    send_asset(node2_addr, &asset_id, 300, recipient_id.clone()).await;
    mine(false);
    refresh_transfers(node1_addr).await;
    refresh_transfers(node1_addr).await;
    refresh_transfers(node2_addr).await;
    assert_eq!(asset_balance(node1_addr, &asset_id).await, 900);
    assert_eq!(asset_balance(node2_addr, &asset_id).await, 100);

    // check decoded RGB invoice (with asset ID)
    let decoded = decode_rgb_invoice(node1_addr, &invoice).await;
    assert_eq!(decoded.recipient_id, recipient_id);
    assert!(matches!(decoded.asset_iface, Some(AssetIface::RGB20)));
    assert_eq!(decoded.asset_id, Some(asset_id.clone()));
    assert_eq!(decoded.amount, None);
    assert!(decoded.network.is_none());
    assert!(decoded.expiration_timestamp.is_some());
    assert_eq!(decoded.transport_endpoints, vec![PROXY_ENDPOINT_REGTEST]);

    let recipient_id = rgb_invoice(node2_addr, None).await.recipient_id;
    send_asset(node1_addr, &asset_id, 200, recipient_id).await;
    mine(false);
    refresh_transfers(node2_addr).await;
    refresh_transfers(node2_addr).await;
    refresh_transfers(node1_addr).await;
    assert_eq!(asset_balance(node1_addr, &asset_id).await, 700);
    assert_eq!(asset_balance(node2_addr, &asset_id).await, 300);
}
