use actix_tracing_util::RequestMetrics;
use opentelemetry::global;
use opentelemetry_semantic_conventions::metric::HTTP_SERVER_REQUEST_DURATION;
use source_control_event_store_interface::subscribers::organization_subscriber::SubscriberMetrics;

pub fn request_metrics() -> RequestMetrics {
    let meter = global::meter("com.rafaeltab.actix");

    RequestMetrics {
        response_size: meter
            .u64_histogram("http.server.response.size")
            .with_description("Size of the http response")
            .with_unit("Byte")
            .with_boundaries(
                [
                    0.0, 1024.0, 4096.0, 16384.0, 65536.0, 262144.0, 1048576.0, 4194304.0,
                    16777216.0, 67108864.0,
                ]
                .to_vec(),
            )
            .build(),
        request_size: meter
            .u64_histogram("http.server.request.size")
            .with_description("Size of the http request")
            .with_unit("Byte")
            .with_boundaries(
                [
                    0.0, 1024.0, 4096.0, 16384.0, 65536.0, 262144.0, 1048576.0, 4194304.0,
                    16777216.0, 67108864.0,
                ]
                .to_vec(),
            )
            .build(),
        duration_seconds: meter
            .f64_histogram(HTTP_SERVER_REQUEST_DURATION)
            .with_description("Duration fo requests")
            .with_unit("second")
            .with_boundaries(
                [
                    0.001, 0.004, 0.016, 0.064, 0.256, 1.024, 4.096, 16.384, 65.536,
                ]
                .to_vec(),
            )
            .build(),
        response_count: meter
            .u64_counter("http.server.response.total")
            .with_description("Amount of responses sent")
            .with_unit("response")
            .build(),
        request_count: meter
            .u64_counter("http.server.request.total")
            .with_description("Amount of requests received")
            .with_unit("response")
            .build(),
    }
}

pub fn subscriber_metrics() -> SubscriberMetrics {
    let meter = global::meter("com.rafaeltab.eventstore");

    SubscriberMetrics {
        event_projection_duration_seconds: meter
            .f64_histogram("eventstore.projection.duration")
            .with_description("Event projection")
            .with_unit("second")
            .with_boundaries(
                [
                    0.001, 0.004, 0.016, 0.064, 0.256, 1.024, 4.096, 16.384, 65.536,
                ]
                .to_vec(),
            )
            .build(),
        event_projection_completed: meter
            .u64_counter("eventstore.projection.completed.total")
            .with_description("Amount of events completed")
            .with_unit("projection")
            .build(),
        event_projection_started: meter
            .u64_counter("eventstore.projection.started.total")
            .with_description("Amount of event projections started")
            .with_unit("projection")
            .build(),
    }
}
