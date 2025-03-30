use opentelemetry_semantic_conventions::{
    attribute::{HTTP_REQUEST_METHOD, HTTP_ROUTE},
    trace::HTTP_RESPONSE_STATUS_CODE,
};
use std::{
    future::{ready, Ready},
    time::SystemTime,
};

use actix_web::{
    body::{BodySize, MessageBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::CONTENT_LENGTH,
    Error,
};
use futures_util::future::LocalBoxFuture;
use opentelemetry::{
    metrics::{Counter, Histogram},
    KeyValue,
};

#[derive(Clone)]
pub struct RequestMetrics {
    pub request_count: Counter<u64>,
    pub response_count: Counter<u64>,
    pub duration_seconds: Histogram<f64>,
    pub request_size: Histogram<u64>,
    pub response_size: Histogram<u64>,
}

pub struct MeterFactory {
    pub metrics: RequestMetrics,
}

impl<S, B> Transform<S, ServiceRequest> for MeterFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MeterMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MeterMiddleware {
            service,
            metrics: self.metrics.clone(),
        }))
    }
}

pub struct MeterMiddleware<S> {
    service: S,
    metrics: RequestMetrics,
}

impl<S, B> Service<ServiceRequest> for MeterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let timer = SystemTime::now();
        let method = req.method().to_string();
        let route = req.match_pattern().unwrap_or(req.path().to_string());
        let name = req.match_name().unwrap_or("").to_string();

        let content_length = req
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|len| len.to_str().ok().and_then(|s| s.parse().ok()))
            .unwrap_or(0);

        let mut attributes: Vec<KeyValue> = Vec::with_capacity(4);
        attributes.push(KeyValue::new(HTTP_REQUEST_METHOD, method.clone()));
        attributes.push(KeyValue::new(HTTP_ROUTE, route.clone()));
        attributes.push(KeyValue::new("http.name", name.clone()));

        self.metrics.request_count.add(1, &attributes);
        self.metrics
            .request_size
            .record(content_length, &attributes);

        let metrics = self.metrics.clone();

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            let response_code = res.response().status().as_u16() as i64;
            attributes.push(KeyValue::new(HTTP_RESPONSE_STATUS_CODE, response_code));

            let response_size = match res.response().body().size() {
                BodySize::Sized(size) => size,
                _ => 0,
            };

            metrics.duration_seconds.record(
                timer.elapsed().map(|t| t.as_secs_f64()).unwrap_or_default(),
                &attributes,
            );

            metrics.response_size.record(response_size, &attributes);

            metrics.response_count.add(1, &attributes);

            Ok(res)
        })
    }
}
