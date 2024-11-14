use core::time::Duration;

use error_stack::Report;
use opentelemetry::{
    KeyValue, global,
    trace::{TraceError, TracerProvider as _},
};
use opentelemetry_otlp::{SpanExporter, WithExportConfig as _};
use opentelemetry_sdk::{
    Resource,
    propagation::TraceContextPropagator,
    runtime,
    trace::{self, RandomIdGenerator, Sampler},
};
use tokio::runtime::Handle;
use tracing::Subscriber;
use tracing_subscriber::{Layer, registry::LookupSpan};

/// Arguments for configuring the logging setup
#[derive(Debug, Clone)]
#[cfg_attr(feature = "clap", derive(clap::Args), clap(next_help_heading = Some("Open Telemetry")))]
pub struct OpenTelemetryConfig {
    /// The OpenTelemetry protocol endpoint for sending traces.
    #[cfg_attr(
        feature = "clap",
        clap(long = "otlp-endpoint", default_value = None, env = "HASH_GRAPH_OTLP_ENDPOINT", global = true)
    )]
    pub endpoint: Option<String>,
}

const OPENTELEMETRY_TIMEOUT_DURATION: Duration = Duration::from_secs(5);

pub type OtlpLayer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
= impl Layer<S>;

/// Creates a layer which connects to the `OpenTelemetry` collector.
///
/// # Errors
///
/// Errors if the `OpenTelemetry` configuration is invalid.
pub fn layer<S>(
    config: &OpenTelemetryConfig,
    handle: &Handle,
) -> Result<OtlpLayer<S>, Report<TraceError>>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    // pipeline spawns a background task to export telemetry data.
    // The handle is used so that we can spawn the task on the correct runtime.
    let _guard = handle.enter();

    let Some(endpoint) = config.endpoint.as_deref() else {
        return Ok(None);
    };

    // Allow correlating trace IDs
    global::set_text_map_propagator(TraceContextPropagator::new());

    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .with_timeout(OPENTELEMETRY_TIMEOUT_DURATION)
        .build()?;

    // Configure sampler args with the following environment variables:
    //   - OTEL_TRACES_SAMPLER_ARG
    //   - OTEL_TRACES_SAMPLER
    //
    // Configure span options with the following environment variables:
    //   - OTEL_SPAN_ATTRIBUTE_COUNT_LIMIT
    //   - OTEL_SPAN_EVENT_COUNT_LIMIT
    //   - OTEL_SPAN_LINK_COUNT_LIMIT
    let trace_config = trace::Config::default()
        .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
            0.1,
        ))))
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(Resource::new(vec![KeyValue::new("service.name", "graph")]));

    let tracer = trace::TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_config(trace_config)
        .build()
        .tracer("graph");

    Ok(Some(tracing_opentelemetry::layer().with_tracer(tracer)))
}
