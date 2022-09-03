use arrayref::array_ref;
use opentelemetry::{
    sdk::trace::Tracer,
    trace::{
        SpanBuilder, SpanContext, SpanId, TraceContextExt, TraceFlags, TraceId, TraceState,
        Tracer as TracerTrait,
    },
    Context,
};
use tracing::info;
use uuid::Uuid;

use crate::structs::WebhookPayload;

pub fn span_id_from_job_id(job_id: &Uuid) -> SpanId {
    SpanId::from_bytes(*array_ref!(job_id.as_bytes(), 0, 8))
}

pub fn trace_id_from_pipeline_id(pipeline_id: &Uuid) -> TraceId {
    TraceId::from_bytes(*pipeline_id.as_bytes())
}

pub fn workflow_span_id_from_pipeline_id(pipeline_id: &Uuid) -> SpanId {
    SpanId::from_bytes(*array_ref!(pipeline_id.as_bytes(), 0, 8))
}

pub fn create_workflow_context(pipeline_id: Uuid) -> Context {
    let cx = Context::current();
    return cx.with_remote_span_context(SpanContext::new(
        trace_id_from_pipeline_id(&pipeline_id),
        workflow_span_id_from_pipeline_id(&pipeline_id),
        TraceFlags::SAMPLED,
        false,
        TraceState::default(),
    ));
}

#[cfg(test)]
mod cx_tests {
    use opentelemetry::trace::TraceContextExt;
    use uuid::Uuid;

    use super::create_workflow_context;

    #[test]
    fn test_has_active_span() {
        assert!(create_workflow_context(Uuid::new_v4()).has_active_span());
    }
}

pub fn build_hook_result_span(payload: &WebhookPayload, tracer: &Tracer) {
    match payload {
        WebhookPayload::PingEvent {
            happened_at,
            id,
            webhook,
        } => {
            info!("Processing PingEvent");
            tracer.build(
                SpanBuilder::from_name("ping")
                    .with_trace_id(TraceId::from_bytes(*id.as_bytes()))
                    .with_start_time(*happened_at)
                    .with_end_time(*happened_at),
            );
        }

        WebhookPayload::JobCompleted {
            happened_at,
            pipeline,
            webhook,
            organization,
            workflow,
            project,
            id,
            job,
        } => {
            if let Some(stopped_at) = job.stopped_at {
                info!("Processing JobCompleted");
                let cx = create_workflow_context(pipeline.id);
                tracer.build_with_context(
                    SpanBuilder::from_name("job")
                        .with_span_id(span_id_from_job_id(&job.id))
                        .with_start_time(job.started_at)
                        .with_end_time(stopped_at),
                    &cx,
                );
            }
        }

        WebhookPayload::WorkflowCompleted {
            id,
            happened_at,
            webhook,
            workflow,
            pipeline,
            project,
            organization,
        } => {
            if let Some(stopped_at) = workflow.stopped_at {
                info!("Processing WorkflowCompleted");
                tracer.build(
                    SpanBuilder::from_name("workflow")
                        .with_trace_id(trace_id_from_pipeline_id(&pipeline.id))
                        .with_span_id(workflow_span_id_from_pipeline_id(&pipeline.id))
                        .with_start_time(workflow.created_at)
                        .with_end_time(stopped_at),
                );
            }
        }
    }
}
