use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use ttrpc::TtrpcContext;
use ttrpc::context::{Context};
use ttrpc::{Client, Server};

// Instead of using include!, we'll generate the files to src/ directory
// and import them as normal modules
mod echo;
mod echo_ttrpc;

use echo::{EchoRequest, EchoResponse};
use echo_ttrpc::{Echo, EchoClient, create_echo};

#[derive(Clone)]
struct EchoService;

impl Echo for EchoService {
    fn echo(&self, _ctx: &TtrpcContext, req: EchoRequest) -> ttrpc::Result<EchoResponse> {
        Ok(EchoResponse {
            message: req.message.clone(),
            ..Default::default()
        })
    }
}

async fn run_unix_socket_benchmark(iterations: usize) -> Result<Vec<Duration>, Box<dyn std::error::Error>> {
    let socket_path = "unix:///tmp/ttrpc_bench.sock";
    
    // Clean up any existing socket
    let _ = std::fs::remove_file(socket_path);
    
    // Start server
    let service: Arc<dyn Echo + Send + Sync> = Arc::new(EchoService);
    //let service = Arc::new(Box::new(EchoService) as Box<dyn Echo + Send + Sync>);
    let mut server = Server::new()
        .bind(socket_path)?
        .register_service(create_echo(service));
    
    let server_handle = tokio::spawn(async move {
        let _ = server.start();
    });
    
    // Give server time to start
    sleep(Duration::from_millis(100)).await;
    
    // Create client
    let client = Client::connect(socket_path)?;
    let echo_client = EchoClient::new(client);
    
    let mut latencies = Vec::with_capacity(iterations);
    
    // Warmup
    for _ in 0..10 {
        let req = EchoRequest {
            message: "warmup".to_string(),
            ..Default::default()
        };
        let _ = echo_client.echo(Context::default(), &req);
    }
    
    // Benchmark
    for i in 0..iterations {
        let req = EchoRequest {
            message: format!("benchmark message {}", i),
            ..Default::default()
        };
        
        let start = Instant::now();
        let _ = echo_client.echo(Context::default(), &req)?;
        let duration = start.elapsed();
        
        latencies.push(duration);
    }
    
    // Cleanup
    server_handle.abort();
    let _ = std::fs::remove_file(socket_path);
    
    Ok(latencies)
}

async fn run_tcp_socket_benchmark(iterations: usize) -> Result<Vec<Duration>, Box<dyn std::error::Error>> {
    let addr = "tcp://127.0.0.1:8087";
    
    // Start server
    //let service = Arc::new(Box::new(EchoService) as Box<dyn Echo + Send + Sync>);
    let service: Arc<dyn Echo + Send + Sync> = Arc::new(EchoService);
    let mut server = Server::new()
        .bind(addr)?
        .register_service(create_echo(service));
    
    let server_handle = tokio::spawn(async move {
        let _ = server.start();
    });
    
    // Give server time to start
    sleep(Duration::from_millis(100)).await;
    
    // Create client
    let client = Client::connect(addr)?;
    let echo_client = EchoClient::new(client);
    
    let mut latencies = Vec::with_capacity(iterations);
    
    // Warmup
    for _ in 0..10 {
        let req = EchoRequest {
            message: "warmup".to_string(),
            ..Default::default()
        };
        let _ = echo_client.echo(Context::default(), &req);
    }
    
    // Benchmark
    for i in 0..iterations {
        let req = EchoRequest {
            message: format!("benchmark message {}", i),
            ..Default::default()
        };
        
        let start = Instant::now();
        let _ = echo_client.echo(Context::default(), &req)?;
        let duration = start.elapsed();
        
        latencies.push(duration);
    }
    
    // Cleanup
    server_handle.abort();
    
    Ok(latencies)
}

fn calculate_stats(latencies: &[Duration]) -> (Duration, Duration, Duration, Duration) {
    let mut sorted = latencies.to_vec();
    sorted.sort();
    
    let sum: Duration = sorted.iter().sum();
    let avg = sum / sorted.len() as u32;
    
    let min = sorted[0];
    let max = sorted[sorted.len() - 1];
    let p99 = sorted[(sorted.len() as f64 * 0.99) as usize];
    
    (min, avg, max, p99)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let iterations = 1000;
    
    println!("Running ttrpc-rust latency benchmark with {} iterations...\n", iterations);
    
    // Unix socket benchmark
    println!("Testing Unix sockets...");
    let unix_latencies = run_unix_socket_benchmark(iterations).await?;
    let (unix_min, unix_avg, unix_max, unix_p99) = calculate_stats(&unix_latencies);
    
    println!("Unix Socket Results:");
    println!("  Min:     {:?}", unix_min);
    println!("  Average: {:?}", unix_avg);
    println!("  Max:     {:?}", unix_max);
    println!("  P99:     {:?}", unix_p99);
    println!();
    
    // TCP socket benchmark
    println!("Testing TCP sockets...");
    let tcp_latencies = run_tcp_socket_benchmark(iterations).await?;
    let (tcp_min, tcp_avg, tcp_max, tcp_p99) = calculate_stats(&tcp_latencies);
    
    println!("TCP Socket Results:");
    println!("  Min:     {:?}", tcp_min);
    println!("  Average: {:?}", tcp_avg);
    println!("  Max:     {:?}", tcp_max);
    println!("  P99:     {:?}", tcp_p99);
    println!();
    
    // Comparison
    let speedup = tcp_avg.as_nanos() as f64 / unix_avg.as_nanos() as f64;
    println!("Comparison:");
    if speedup > 1.0 {
        println!("  Unix sockets are {:.2}x faster than TCP", speedup);
    } else {
        println!("  TCP sockets are {:.2}x faster than Unix", 1.0 / speedup);
    }
    
    Ok(())
}

// Updated build.rs content:
/*
use std::env;
use std::path::PathBuf;

fn main() {
    let src_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("src");
    let proto_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    
    ttrpc_codegen::Codegen::new()
        .out_dir(&src_dir)  // Generate directly to src/ directory
        .inputs(&[proto_path.join("echo.proto")])
        .include(&proto_path)
        .rust_protobuf()
        .run()
        .expect("Generate code failed.");
}
*/

// Cargo.toml dependencies needed:
/*
[package]
name = "ttrpc-benchmark"
version = "0.1.0"
edition = "2021"

[dependencies]
ttrpc = "0.8"
tokio = { version = "1.0", features = ["full"] }
protobuf = "3.0"

[build-dependencies]
ttrpc-codegen = "0.4"
*/
