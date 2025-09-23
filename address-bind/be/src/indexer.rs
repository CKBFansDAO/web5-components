use crate::verify::verify_tx;
use ckb_jsonrpc_types::BlockNumber;
use ckb_sdk::{NetworkType, rpc::CkbRpcClient};
use rusqlite::{Connection, Result};
use std::time::Duration;
use tokio::time::sleep;

// 定义查询处理函数
async fn query_by_from(
    axum::extract::Path(from): axum::extract::Path<String>,
) -> impl axum::response::IntoResponse {
    let conn = Connection::open("bind_info.db").unwrap();
    let mut stmt = conn
        .prepare("SELECT to_addr, timestamp FROM bind_info WHERE from_addr = ?1")
        .unwrap();
    let rows = stmt
        .query_map([from], |row| {
            Ok(serde_json::json!({
                "to": row.get::<_, String>(0)?,
                "timestamp": row.get::<_, i64>(1)?
            }))
        })
        .unwrap();

    let result: Vec<_> = rows.filter_map(|r| r.ok()).collect();
    axum::Json(result)
}

async fn query_by_to(
    axum::extract::Path(to): axum::extract::Path<String>,
) -> impl axum::response::IntoResponse {
    let conn = Connection::open("bind_info.db").unwrap();
    let mut stmt = conn
        .prepare(
            "
            WITH latest_binds AS (
                SELECT from_addr, MAX(timestamp) as max_timestamp
                FROM bind_info
                GROUP BY from_addr
            )
            SELECT b.from_addr, b.timestamp
            FROM bind_info b
            INNER JOIN latest_binds l
                ON b.from_addr = l.from_addr
                AND b.timestamp = l.max_timestamp
            WHERE b.to_addr = ?1
        ",
        )
        .unwrap();
    let rows = stmt
        .query_map([to], |row| {
            Ok(serde_json::json!({
                "from": row.get::<_, String>(0)?,
                "timestamp": row.get::<_, i64>(1)?
            }))
        })
        .unwrap();

    let result: Vec<_> = rows.filter_map(|r| r.ok()).collect();
    axum::Json(result)
}

pub async fn server(
    ckb_client: &CkbRpcClient,
    network_type: NetworkType,
    start_height: u64,
    listen_port: u64,
) -> Result<(), String> {
    // 打开 SQLite 数据库
    let conn =
        Connection::open("bind_info.db").map_err(|_| "Failed to open database".to_string())?;

    // 创建表（如果不存在）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sync_status (height INTEGER PRIMARY KEY)",
        [],
    )
    .map_err(|_| "Failed to create table sync_status".to_string())?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bind_info (from_addr TEXT, to_addr TEXT, timestamp INTEGER, UNIQUE(from_addr, to_addr, timestamp))",
        [],
    )
    .map_err(|_| "Failed to create table bind_info".to_string())?;

    // 获取上次同步的高度
    let mut current_height = conn
        .query_row(
            "SELECT height FROM sync_status ORDER BY height DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(start_height);

    // start http server on listen_port 
    let app = axum::Router::new()
        .route("/by_from/:from", axum::routing::get(query_by_from))
        .route("/by_to/:to", axum::routing::get(query_by_to));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", listen_port)).await.unwrap();
    axum::serve(listener, app).await
        .map_err(|e| format!("Server error: {}", e))?;

    loop {
        // 获取最新区块高度
        let tip_block = ckb_client
            .get_tip_block_number()
            .map_err(|e| "Failed to get tip block number".to_string())?;

        // 如果已经同步到最新高度，等待新区块
        if current_height >= tip_block.into() {
            sleep(Duration::from_secs(10)).await;
            continue;
        }

        // 获取当前高度的区块
        let ret = ckb_client.get_block_by_number(BlockNumber::from(current_height));

        println!("current_height: {}", current_height);

        if let Ok(Some(block)) = ret {
            // 遍历区块中的所有交易
            for tx in block.transactions.into_iter() {
                // 验证交易
                if let Ok((from, to, timestamp)) =
                    verify_tx(ckb_client, network_type, &tx.inner).await
                {
                    println!("from: {}, to: {}, timestamp: {}", from, to, timestamp);
                    // 记录绑定信息到数据库
                    conn.execute(
                        "INSERT OR REPLACE INTO bind_info (from_addr, to_addr, timestamp) VALUES (?1, ?2, ?3)",
                        [&from, &to, &timestamp.to_string()],
                    )
                    .map_err(|_| "Failed to insert bind info".to_string())?;
                }
            }

            // 更新同步高度
            if current_height % 100 == 0 {
                conn.execute(
                    "INSERT OR REPLACE INTO sync_status (height) VALUES (?1)",
                    [current_height],
                )
                .map_err(|_| "Failed to update sync status".to_string())?;
            }

            current_height += 1;
        }
    }
}
