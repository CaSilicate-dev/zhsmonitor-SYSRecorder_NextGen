mod structs;
mod utils;

use sqlx::mysql::MySqlPoolOptions;
use sqlx::query;
use std::fs;
use std::time::Duration;
use structs::*;
use sysinfo::{Networks, System};
use utils::*;

#[tokio::main]
async fn main() {
    let content = fs::read_to_string("config.json").expect("Failed to read config");
    let config: AppConfig = serde_json::from_str(content.as_str()).expect("Failed to parse config");

    let interval = config.interval;
    let db_url = config.db_url;
    let min_connections = config.min_connections;
    let max_connections = config.max_connections;

    let pool = MySqlPoolOptions::new()
        .min_connections(min_connections)
        .max_connections(max_connections)
        .connect(db_url.as_str())
        .await
        .expect("failed to connect to database");

    let mut ticker = tokio::time::interval(Duration::from_millis(interval as u64));

    let mut sys = System::new_all();
    let mut networks = Networks::new();

    let mut running_counter: i64 = 0;

    loop {
        ticker.tick().await;
        sys.refresh_all();
        networks.refresh(true);

        let network_data = networks
            .get(config.network_card_name.as_str())
            .expect("Network interface not found");

        let total_memory = sys.total_memory() as f64;
        let used_memory = sys.used_memory() as f64;

        let cpu_usage = advanced_round(sys.global_cpu_usage() as f64 , 2);
        let memory_usage = advanced_round((used_memory / total_memory) * 100.0 , 2);
        let net_send = network_data.transmitted();
        let net_recv = network_data.received();

        if running_counter >= 5 {
            let s = query(
                format!(
                    r#"
                    INSERT INTO {table} (cpu_usage, memory_usage, net_send_rate, net_recv_rate) VALUES (?, ?, ?, ?)
                    "#,
                    table = config.db_table
                )
                .as_str(),
            )
            .bind(cpu_usage)
            .bind(memory_usage)
            .bind(net_send)
            .bind(net_recv)
            .execute(&pool)
            .await;
            match s {
                Ok(a) => {
                    if a.rows_affected() != 1 {
                        println!(
                            "something wrong when inserting data, {} lines affected",
                            a.rows_affected()
                        );
                    }
                }
                Err(e) => {
                    println!("Failed to insert data: {}", e);
                }
            }
        }

        running_counter += 1;
    }
}
