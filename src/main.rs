pub mod combined_book;
pub mod config;
pub mod exchange;
pub mod grpc;
pub mod orderbook_processor;
pub mod orderbook {
    tonic::include_proto!("orderbook");
}
use config::load_config;
use grpc::orderbook_service::OrderbookService;
use orderbook::orderbook_aggregator_server::OrderbookAggregatorServer;
use orderbook_processor::OrderbookProcessor;
use tonic::transport::Server;
use tracing::{debug, error, info};
use tracing_subscriber;

const GRPC_SERVER_ADDR: &str = "127.0.0.1:50051";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Starting the application, loading configuration file");
    let config = match load_config("config/config.json5") {
        Ok(config) => {
            debug!("Configuration loaded: {:?}", config);
            config
        }
        Err(err) => {
            error!("Failed to load config file: {:?}", err);
            return;
        }
    };

    info!("Creating orderbook processor");
    let mut orderbook_processor = OrderbookProcessor::new(config);

    info!("Creating orderbook service");
    let orderbook_service = OrderbookService::new(orderbook_processor.subscribe());

    info!("Spawning orderbook processor drive loop..");
    tokio::spawn(async move {
        if let Err(e) = orderbook_processor.initialise_exchanges().await {
            error!("Error initializing exchanges: {:?}", e);
        } else {
            orderbook_processor.drive_and_broadcast().await;
        }
    });

    info!("Setting up gRPC service listening on {}", GRPC_SERVER_ADDR);
    if let Err(err) = Server::builder()
        .add_service(OrderbookAggregatorServer::new(orderbook_service))
        .serve(GRPC_SERVER_ADDR.parse().unwrap())
        .await
    {
        error!("Error running gRPC server: {:?}", err);
    }

    info!("Shutting down");
}
