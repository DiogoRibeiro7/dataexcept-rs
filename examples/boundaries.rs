//! Builds framework-neutral HTTP and broker boundary contexts.

use std::collections::BTreeMap;

use dataexcept::{
    BrokerContextOptions, BrokerOperation, broker_context_from_message, http_context_from_request,
};

const TRACEPARENT: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";

fn main() {
    let headers = BTreeMap::from([
        ("x-request-id".to_owned(), "req-42".to_owned()),
        ("x-correlation-id".to_owned(), "corr-9".to_owned()),
        ("traceparent".to_owned(), TRACEPARENT.to_owned()),
    ]);

    let http = http_context_from_request("POST", Some("/users/{id}"), &headers, Some("accounts"))
        .expect("valid HTTP context");

    let broker = broker_context_from_message(
        BrokerOperation::Consume,
        "orders",
        &headers,
        BrokerContextOptions::default()
            .component("billing")
            .correlation_id("corr-9")
            .partition(3)
            .offset(1042)
            .consumer_group("billing")
            .message_id("msg-7"),
    )
    .expect("valid broker context");

    println!(
        "http operation: {}",
        http.operation_context()
            .operation()
            .expect("HTTP operation should be present")
    );
    println!(
        "broker operation: {}",
        broker
            .operation_context()
            .operation()
            .expect("broker operation should be present")
    );
}
