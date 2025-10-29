use crate::{Indexer, error::AppError, verify::verify_tx};
use ckb_jsonrpc_types::BlockNumber;
use ckb_sdk::{NetworkType, rpc::CkbRpcClient};
use color_eyre::{Result, eyre::eyre};
use common_x::restful::{
    axum::{
        Router,
        extract::{Path, State},
        response::IntoResponse,
        routing::get,
    },
    ok,
};
use sqlx::{Executor, postgres::PgPoolOptions, query, query_as};
use std::time::Duration;
use tokio::time::sleep;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer};

// define query handler
async fn query_by_from(
    State(state): State<Indexer>,
    Path(from): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let rows: Vec<(String, i64)> = query_as(&format!(
        "SELECT to_addr, timestamp FROM bind_info WHERE from_addr = '{from}'"
    ))
    .fetch_all(&state.db)
    .await
    .map_err(|e| eyre!("exec sql failed: {e}"))?;
    let result: Vec<_> = rows
        .iter()
        .map(|row| {
            serde_json::json!({
                "to": row.0,
                "timestamp": row.1
            })
        })
        .collect();

    Ok(ok(result))
}

async fn query_by_to(
    State(state): State<Indexer>,
    Path(to): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let rows: Vec<(String, i64)> = query_as(&format!(
        "WITH latest_binds AS (
                SELECT from_addr, MAX(timestamp) as max_timestamp
                FROM bind_info
                GROUP BY from_addr
            )
            SELECT b.from_addr, b.timestamp
            FROM bind_info b
            INNER JOIN latest_binds l
                ON b.from_addr = l.from_addr
                AND b.timestamp = l.max_timestamp
            WHERE b.to_addr = '{to}'"
    ))
    .fetch_all(&state.db)
    .await
    .map_err(|e| eyre!("exec sql failed: {e}"))?;
    let result: Vec<_> = rows
        .iter()
        .map(|row| {
            serde_json::json!({
                "from": row.0,
                "timestamp": row.1
            })
        })
        .collect();

    Ok(ok(result))
}

pub async fn server(
    ckb_client: &CkbRpcClient,
    network_type: NetworkType,
    db_url: &str,
    start_height: u64,
    listen_port: u16,
) -> Result<()> {
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;

    // create table
    db.execute("CREATE TABLE IF NOT EXISTS sync_status (height BIGINT PRIMARY KEY)")
        .await?;
    db.execute(
        "CREATE TABLE IF NOT EXISTS bind_info (from_addr TEXT, to_addr TEXT, timestamp BIGINT, UNIQUE(from_addr, to_addr, timestamp))",
    ).await?;

    // get last sync height
    let mut current_height: u64 =
        query_as("SELECT height FROM sync_status ORDER BY height DESC LIMIT 1")
            .fetch_one(&db)
            .await
            .map(|r: (i32,)| r.0 as u64)
            .unwrap_or(start_height);

    let indexer = Indexer { db: db.clone() };

    tokio::spawn(async move {
        // start http server on listen_port
        let app = Router::new()
            .route("/by_from/{from}", get(query_by_from))
            .route("/by_to/{to}", get(query_by_to))
            .layer((TimeoutLayer::new(Duration::from_secs(10)),))
            .layer(CorsLayer::permissive())
            .with_state(indexer);

        common_x::restful::http_serve(listen_port, app)
            .await
            .map_err(|e| eyre!("{e}"))
    });

    loop {
        // get latest block height
        if let Ok(tip_block) = ckb_client.get_tip_block_number() {
            // if already synced to latest height, wait for new block
            let tip_block = tip_block.value();
            if current_height >= tip_block {
                sleep(Duration::from_secs(1)).await;
                continue;
            } else if current_height.is_multiple_of(10) {
                info!(
                    "tip_block: {tip_block}, current_height: {current_height}, waiting block: {}",
                    tip_block - current_height
                );
            }
        } else {
            sleep(Duration::from_secs(1)).await;
            continue;
        }

        // get block by number
        let ret = ckb_client.get_block_by_number(BlockNumber::from(current_height));

        if let Ok(Some(block)) = ret {
            // proc transactions in block
            for (index, tx) in block.transactions.into_iter().enumerate() {
                // ignore cellbase transaction
                if index == 0 {
                    continue;
                }

                // verify transaction
                match verify_tx(ckb_client, network_type, &tx.inner).await {
                    Ok((from, to, timestamp)) => {
                        info!("from: {from}, to: {to}, timestamp: {timestamp}");
                        // insert bind info to db
                        if let Err(e) = db.execute(query(
                            &format!("INSERT INTO bind_info (from_addr, to_addr, timestamp) VALUES ('{from}', '{to}', '{timestamp}')")
                        ))
                        .await {
                            error!("Failed to insert bind info: {e}");
                        }
                    }
                    Err(e) => {
                        if e.contains("get_tx failed") || e.contains("sig_bytes") {
                            error!("verify_tx {} is failed, err: {e}", tx.hash);
                        }
                    }
                }
            }

            // update sync height
            // not too frequently
            if current_height.is_multiple_of(100)
                && let Err(e) = db
                    .execute(query(&format!(
                        "INSERT INTO sync_status (height) VALUES ('{current_height}') ON CONFLICT (height) DO NOTHING;"
                    )))
                    .await
            {
                error!("Failed to update sync status: {e}");
            }

            current_height += 1;
        }
    }
}

#[tokio::test]
async fn test_one() -> Result<()> {
    common_x::log::init_log_filter("info");
    let ckb_client = CkbRpcClient::new("https://testnet.ckb.dev/");
    let ret = ckb_client.get_block_by_number(BlockNumber::from(18977278));

    if let Ok(Some(block)) = ret {
        // proc transactions in block
        for (index, tx) in block.transactions.into_iter().enumerate() {
            // ignore cellbase transaction
            if index == 0 {
                continue;
            }

            // verify transaction
            match verify_tx(&ckb_client, NetworkType::Testnet, &tx.inner).await {
                Ok((from, to, timestamp)) => {
                    info!("from: {from}, to: {to}, timestamp: {timestamp}");
                }
                Err(e) => {
                    if e.contains("get_tx failed") {
                        error!("verify_tx {} is failed, err: {e}", tx.hash);
                    }
                }
            }
        }
    }
    Ok(())
}
