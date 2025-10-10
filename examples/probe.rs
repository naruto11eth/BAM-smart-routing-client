/// quick example for probing
/// run with --> cargo run --example probe
use bam_smart_routing_client::{BamSmartRouter, ProbeMethod, RouterConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let mut config = RouterConfig::default();
    config.probe_method = ProbeMethod::TcpConnect;
    config.probe_timeout = std::time::Duration::from_millis(1_500);
    config.rpc_timeout = std::time::Duration::from_secs(5);
    config.retries = 1;

    let router = BamSmartRouter::new(config)?;
    let probes = router.probe_regions().await;

    let mut rows = Vec::new();
    let mut code_width = "Region".len();
    let mut endpoint_width = "Endpoint".len();

    for record in &probes {
        let code = record.region.code;
        let endpoint = record.region.endpoint;
        let status = format!("{}", record.status);
        code_width = code_width.max(code.len());
        endpoint_width = endpoint_width.max(endpoint.len());
        rows.push((code, endpoint, status));
    }

    println!(
        "\n{:<code_w$} | {:<endpoint_w$} | Status",
        "Region",
        "Endpoint",
        code_w = code_width,
        endpoint_w = endpoint_width
    );
    println!(
        "{:-<code_w$}-+-{:-<endpoint_w$}-+-{:-<6}",
        "",
        "",
        "",
        code_w = code_width,
        endpoint_w = endpoint_width
    );

    for (code, endpoint, status) in &rows {
        println!(
            "{code:<code_w$} | {endpoint:<endpoint_w$} | {status}",
            code = code,
            endpoint = endpoint,
            status = status,
            code_w = code_width,
            endpoint_w = endpoint_width
        );
    }

    let tx_b64 = parse_tx_flag();

    if let Ok(selection) = router.select_fastest().await {
        println!();
        println!(
            "Fastest region: {} ({} ms)",
            selection.region.endpoint,
            selection.latency.as_millis()
        );

        if let Some(tx_b64) = tx_b64 {
            match router.submit_transaction_base64(&tx_b64).await {
                Ok(outcome) => {
                    println!(
                        "Submitted tx {} via {} in {} ms (attempts: {})",
                        outcome.signature,
                        outcome.region.endpoint,
                        outcome.latency.as_millis(),
                        outcome.attempts
                    );
                }
                Err(err) => {
                    eprintln!(
                        "Failed to submit transaction (strategy: {:?}): {}",
                        router.config().submission_endpoint,
                        err
                    );
                }
            }
        }
    }

    Ok(())
}

fn parse_tx_flag() -> Option<String> {
    let mut args = std::env::args().skip(1); // skip binary name
    let mut tx_b64: Option<String> = None;

    while let Some(arg) = args.next() {
        if let Some(value) = arg.strip_prefix("--tx-base64=") {
            tx_b64 = Some(value.to_string());
        } else if arg == "--tx-base64" || arg == "-t" {
            if let Some(value) = args.next() {
                tx_b64 = Some(value);
            }
        }
    }

    tx_b64
}
