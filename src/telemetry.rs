use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt};

use opentelemetry_sdk::Resource;
use tracing::Subscriber;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::{MakeWriter};
use opentelemetry_sdk::trace::{RandomIdGenerator, Sampler, SdkTracerProvider};
use opentelemetry::{
    global,
    trace::{TracerProvider},
    KeyValue,
};
use opentelemetry_sdk::trace::Tracer as SdkTracer;
use opentelemetry_otlp::{WithExportConfig};
use crate::configuration::JaegerSettings;

pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
    settings: JaegerSettings,
) -> impl Subscriber + Send + Sync
    where
        Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatter_layer = BunyanFormattingLayer::new(name, sink);

    let tracer = construct_open_telemetry_tracer(&settings.address, settings.port);
    // let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatter_layer)
        // .with(telemetry_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync + 'static) {
    LogTracer::init().expect("Failed to initialize logger");
    set_global_default(subscriber).expect("Failed to set global default subscriber");
}

pub fn construct_open_telemetry_tracer(address: &String, port: u16) -> SdkTracer {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(format!("http://{}:{}", address, port))
        .with_timeout(Duration::from_secs(3))
        .build().unwrap();

    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_sampler(Sampler::AlwaysOn)
        .with_id_generator(RandomIdGenerator::default())
        .with_max_events_per_span(64)
        .with_max_attributes_per_span(16)
        .with_resource(Resource::builder_empty().with_attributes([KeyValue::new("service.name", "zero2prod")]).build())
        .build();
        
    global::set_tracer_provider(tracer_provider.clone());
    tracer_provider.tracer("tracer-name")
}