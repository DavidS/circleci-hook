use arrayref::array_ref;
use chrono::{DateTime, FixedOffset};
use opentelemetry::{
    sdk::trace::Tracer,
    trace::{
        SpanBuilder, SpanContext, SpanId, TraceContextExt, TraceFlags, TraceId, TraceState,
        Tracer as TracerTrait,
    },
    Context, Key, KeyValue, StringValue, Value,
};
use serde::Deserialize;
use tracing::{debug, info};
use uuid::Uuid;

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum WebhookPayload {
    #[serde(rename = "ping")]
    PingEvent {
        id: Uuid,
        happened_at: DateTime<FixedOffset>,
        webhook: Webhook,
    },
    #[serde(rename = "workflow-completed")]
    WorkflowCompleted {
        id: Uuid,
        happened_at: String,
        organization: Organization,
        project: Project,
        pipeline: Pipeline,
        webhook: Webhook,
        workflow: Workflow,
    },
    #[serde(rename = "job-completed")]
    JobCompleted {
        id: Uuid,
        happened_at: String,
        organization: Organization,
        project: Project,
        pipeline: Pipeline,
        webhook: Webhook,
        workflow: Workflow,
        job: Job,
    },
}

impl WebhookPayload {
    pub fn build_span(&self, tracer: &Tracer) {
        match self {
            WebhookPayload::PingEvent {
                id,
                happened_at,
                webhook,
            } => {
                info!("Processing PingEvent");
                tracer.build(
                    SpanBuilder::from_name("ping")
                        .with_trace_id(TraceId::from_bytes(*id.as_bytes()))
                        .with_start_time(*happened_at)
                        .with_end_time(*happened_at)
                        .with_attributes([webhook.to_kv()].concat()),
                );
            }

            WebhookPayload::JobCompleted {
                id: _,
                happened_at: _,
                organization,
                project,
                pipeline,
                webhook,
                workflow,
                job,
            } => {
                if let Some(stopped_at) = job.stopped_at {
                    debug!("pipeline: {:#?}", pipeline);
                    info!("Processing JobCompleted");
                    tracer.build_with_context(
                        SpanBuilder::from_name(format!("job: {}", job.name))
                            .with_span_id(job.span_id())
                            .with_start_time(job.started_at)
                            .with_end_time(stopped_at)
                            .with_attributes(
                                [
                                    vec![KeyValue {
                                        key: Key::new("circleci.kind"),
                                        value: Value::String(StringValue::from("job")),
                                    }],
                                    organization.to_kv(),
                                    project.to_kv(),
                                    pipeline.to_kv(),
                                    webhook.to_kv(),
                                    workflow.to_kv(),
                                    job.to_kv(),
                                ]
                                .concat(),
                            ),
                        &workflow.context(),
                    );
                }
            }

            WebhookPayload::WorkflowCompleted {
                id: _,
                happened_at: _,
                organization,
                project,
                pipeline,
                webhook,
                workflow,
            } => {
                if let Some(stopped_at) = workflow.stopped_at {
                    info!("Processing WorkflowCompleted");
                    tracer.build(
                        SpanBuilder::from_name(format!("workflow: {}", workflow.name))
                            .with_trace_id(workflow.trace_id())
                            .with_span_id(workflow.span_id())
                            .with_start_time(workflow.created_at)
                            .with_end_time(stopped_at)
                            .with_attributes(
                                [
                                    vec![KeyValue {
                                        key: Key::new("circleci.kind"),
                                        value: Value::String(StringValue::from("workflow")),
                                    }],
                                    organization.to_kv(),
                                    project.to_kv(),
                                    pipeline.to_kv(),
                                    webhook.to_kv(),
                                    workflow.to_kv(),
                                ]
                                .concat(),
                            ),
                    );
                }
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
}

impl Organization {
    fn to_kv(&self) -> Vec<KeyValue> {
        vec![
            KeyValue {
                key: Key::new("circleci.organization.id"),
                value: Value::String(format!("{}", self.id.urn()).into()),
            },
            KeyValue {
                key: Key::new("circleci.organization.name"),
                value: Value::String(StringValue::from(self.name.clone())),
            },
        ]
    }
}

#[derive(Deserialize, Debug)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

impl Project {
    fn to_kv(&self) -> Vec<KeyValue> {
        vec![
            KeyValue {
                key: Key::new("circleci.project.id"),
                value: Value::String(format!("{}", self.id.urn()).into()),
            },
            KeyValue {
                key: Key::new("circleci.project.name"),
                value: Value::String(StringValue::from(self.name.clone())),
            },
            KeyValue {
                key: Key::new("circleci.project.slug"),
                value: Value::String(StringValue::from(self.slug.clone())),
            },
        ]
    }
}

#[derive(Deserialize, Debug)]
pub struct Pipeline {
    pub created_at: DateTime<FixedOffset>,
    pub id: Uuid,
    pub number: i64,
    // TODO: complete full deserialisation here
    pub trigger: Option<serde_json::Value>,
    // TODO: complete full deserialisation here
    pub vcs: Option<serde_json::Value>,
}

impl Pipeline {
    fn to_kv(&self) -> Vec<KeyValue> {
        vec![
            KeyValue {
                key: Key::new("circleci.pipeline.id"),
                value: Value::String(format!("{}", self.id.urn()).into()),
            },
            KeyValue {
                key: Key::new("circleci.pipeline.number"),
                value: Value::I64(self.number),
            },
        ]
    }
}

#[derive(Deserialize, Debug)]
pub struct Webhook {
    pub id: Uuid,
    pub name: String,
}

impl Webhook {
    fn to_kv(&self) -> Vec<KeyValue> {
        vec![KeyValue {
            key: Key::new("circleci.webhook.id"),
            value: Value::String(format!("{}", self.id.urn()).into()),
        }]
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct Workflow {
    pub created_at: DateTime<FixedOffset>,
    pub id: Uuid,
    pub name: String,
    pub status: Option<String>,
    pub stopped_at: Option<DateTime<FixedOffset>>,
    pub url: String,
}

impl Workflow {
    fn trace_id(&self) -> TraceId {
        TraceId::from_bytes(*self.id.as_bytes())
    }

    fn span_id(&self) -> SpanId {
        SpanId::from_bytes(*array_ref!(self.id.as_bytes(), 0, 8))
    }

    fn context(&self) -> Context {
        let cx = Context::current();
        cx.with_remote_span_context(SpanContext::new(
            self.trace_id(),
            self.span_id(),
            TraceFlags::SAMPLED,
            false,
            TraceState::default(),
        ))
    }

    fn to_kv(&self) -> Vec<KeyValue> {
        let mut result = vec![
            KeyValue {
                key: Key::new("circleci.workflow.id"),
                value: Value::String(format!("{}", self.id.urn()).into()),
            },
            KeyValue {
                key: Key::new("circleci.workflow.name"),
                value: Value::String(StringValue::from(self.name.clone())),
            },
            KeyValue {
                key: Key::new("circleci.workflow.url"),
                value: Value::String(StringValue::from(self.url.clone())),
            },
        ];
        if let Some(status) = &self.status {
            result.push(KeyValue {
                key: Key::new("circleci.workflow.status"),
                value: Value::String(StringValue::from(status.clone())),
            });
        }
        result
    }
}

#[cfg(test)]
mod workflow_tests {
    use opentelemetry::trace::TraceContextExt;

    use super::Workflow;

    #[test]
    fn test_has_active_span() {
        let w = Workflow::default();
        assert!(w.context().has_active_span());
    }
}

#[derive(Deserialize, Debug)]
pub struct Job {
    pub id: Uuid,
    pub name: String,
    pub number: i64,
    pub started_at: DateTime<FixedOffset>,
    pub status: String,
    pub stopped_at: Option<DateTime<FixedOffset>>,
}

impl Job {
    fn span_id(&self) -> SpanId {
        SpanId::from_bytes(*array_ref!(self.id.as_bytes(), 0, 8))
    }

    fn to_kv(&self) -> Vec<KeyValue> {
        vec![
            KeyValue {
                key: Key::new("circleci.job.id"),
                value: Value::String(format!("{}", self.id.urn()).into()),
            },
            KeyValue {
                key: Key::new("circleci.job.name"),
                value: Value::String(StringValue::from(self.name.clone())),
            },
            KeyValue {
                key: Key::new("circleci.job.number"),
                value: Value::I64(self.number),
            },
            KeyValue {
                key: Key::new("circleci.job.status"),
                value: Value::String(StringValue::from(self.status.clone())),
            },
        ]
    }
}

// TODO: complete full deserialisation here
// #[derive(Deserialize, Debug)]
// pub struct Vcs {
//     branch: String,
//     // TODO: complete full deserialisation here
//     commit: serde_json::Value,
//     origin_repository_url: String,
//     provider_name: String,
//     revision: String,
//     target_repository_url: String,
// }

// Example webhook payload:
// {
//     "happened_at": "2022-08-27T20:16:36.531665Z",
//     "id": "00f3055f-d25c-4641-bdcd-33e19f3b5d7d",
//     "type": "ping",
//     "webhook": {
//         "id": "d4ab06bc-eb79-463d-8aa4-47d066382d3b",
//         "name": "ngrok test"
//     }
// }

// Example webhook payload: "workflow-completed"
// Object {
//     "happened_at": String("2022-08-27T20:26:31.388615Z"),
//     "id": String("46924cd3-e825-30da-8036-b2f293194bc9"),
//     "organization": Object {
//         "id": String("b689dafb-ccea-4a88-8d20-f380ef2b439c"),
//         "name": String("DavidS"),
//     },
//     "pipeline": Object {
//         "created_at": String("2022-08-27T20:25:40.570Z"),
//         "id": String("2bed20e7-711a-45cf-b7e8-017a0575a26c"),
//         "number": Number(10),
//         "trigger": Object {
//             "type": String("webhook"),
//         },
//         "vcs": Object {
//             "branch": String("main"),
//             "commit": Object {
//                 "author": Object {
//                     "email": String("david@black.co.at"),
//                     "name": String("David Schmitt"),
//                 },
//                 "authored_at": String("2022-08-27T20:25:35Z"),
//                 "body": String(""),
//                 "committed_at": String("2022-08-27T20:25:35Z"),
//                 "committer": Object {
//                     "email": String("david@black.co.at"),
//                     "name": String("David Schmitt"),
//                 },
//                 "subject": String("chore: implement basic event debugging and the PingEvent"),
//             },
//             "origin_repository_url": String("https://github.com/DavidS/circleci-hook"),
//             "provider_name": String("github"),
//             "revision": String("71eb8857ea7e13f36021af32f3b7cc9304b491dd"),
//             "target_repository_url": String("https://github.com/DavidS/circleci-hook"),
//         },
//     },
//     "project": Object {
//         "id": String("1fbc30b3-cdb4-4874-a42e-abb81ffd0364"),
//         "name": String("circleci-hook"),
//         "slug": String("github/DavidS/circleci-hook"),
//     },
//     "type": String("workflow-completed"),
//     "webhook": Object {
//         "id": String("d4ab06bc-eb79-463d-8aa4-47d066382d3b"),
//         "name": String("ngrok test"),
//     },
//     "workflow": Object {
//         "created_at": String("2022-08-27T20:25:40.675Z"),
//         "id": String("410c427b-40a8-4bb4-9d42-5561f5bce5ba"),
//         "name": String("production"),
//         "status": String("success"),
//         "stopped_at": String("2022-08-27T20:26:31.289Z"),
//         "url": String("https://app.circleci.com/pipelines/github/DavidS/circleci-hook/10/workflows/410c427b-40a8-4bb4-9d42-5561f5bce5ba"),
//     },
// }

// Example webhook payload: "job-completed"
// Object {
//     "happened_at": String("2022-08-27T20:26:31.353978Z"),
//     "id": String("ba0c8055-1f10-326e-8cf2-d7a4f5432d23"),
//     "job": Object {
//         "id": String("20e45d7e-e4a7-4aa3-8f92-fd6d9d01da75"),
//         "name": String("rust/lint-test-build"),
//         "number": Number(10),
//         "started_at": String("2022-08-27T20:25:43.007Z"),
//         "status": String("success"),
//         "stopped_at": String("2022-08-27T20:26:31.289Z"),
//     },
//     "organization": Object {
//         "id": String("b689dafb-ccea-4a88-8d20-f380ef2b439c"),
//         "name": String("DavidS"),
//     },
//     "pipeline": Object {
//         "created_at": String("2022-08-27T20:25:40.570Z"),
//         "id": String("2bed20e7-711a-45cf-b7e8-017a0575a26c"),
//         "number": Number(10),
//         "trigger": Object {
//             "type": String("webhook"),
//         },
//         "vcs": Object {
//             "branch": String("main"),
//             "commit": Object {
//                 "author": Object {
//                     "email": String("david@black.co.at"),
//                     "name": String("David Schmitt"),
//                 },
//                 "authored_at": String("2022-08-27T20:25:35Z"),
//                 "body": String(""),
//                 "committed_at": String("2022-08-27T20:25:35Z"),
//                 "committer": Object {
//                     "email": String("david@black.co.at"),
//                     "name": String("David Schmitt"),
//                 },
//                 "subject": String("chore: implement basic event debugging and the PingEvent"),
//             },
//             "origin_repository_url": String("https://github.com/DavidS/circleci-hook"),
//             "provider_name": String("github"),
//             "revision": String("71eb8857ea7e13f36021af32f3b7cc9304b491dd"),
//             "target_repository_url": String("https://github.com/DavidS/circleci-hook"),
//         },
//     },
//     "project": Object {
//         "id": String("1fbc30b3-cdb4-4874-a42e-abb81ffd0364"),
//         "name": String("circleci-hook"),
//         "slug": String("github/DavidS/circleci-hook"),
//     },
//     "type": String("job-completed"),
//     "webhook": Object {
//         "id": String("d4ab06bc-eb79-463d-8aa4-47d066382d3b"),
//         "name": String("ngrok test"),
//     },
//     "workflow": Object {
//         "created_at": String("2022-08-27T20:25:40.675Z"),
//         "id": String("410c427b-40a8-4bb4-9d42-5561f5bce5ba"),
//         "name": String("production"),
//         "stopped_at": String("2022-08-27T20:26:31.289Z"),
//         "url": String("https://app.circleci.com/pipelines/github/DavidS/circleci-hook/10/workflows/410c427b-40a8-4bb4-9d42-5561f5bce5ba"),
//     },
// }
