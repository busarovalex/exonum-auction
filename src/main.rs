extern crate chrono;
#[macro_use]
extern crate exonum;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate bincode;

use exonum::helpers::fabric::NodeBuilder;

mod service;
mod schema;
mod transactions;
mod api;

fn main() {
    exonum::helpers::init_logger().unwrap();
    NodeBuilder::new()
        .with_service(Box::new(exonum_configuration::ServiceFactory))
        .with_service(Box::new(service::ServiceFactory))
        .run();
}
