use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct Organization {
    pub id: String,
    pub name: String,
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
#[derive(Deserialize, Debug)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}
#[derive(Deserialize, Debug)]
pub struct Webhook {
    pub id: Uuid,
    pub name: String,
}
#[derive(Deserialize, Debug)]
pub struct Workflow {
    pub created_at: DateTime<FixedOffset>,
    pub id: Uuid,
    pub name: String,
    pub status: Option<String>,
    pub stopped_at: Option<DateTime<FixedOffset>>,
    pub url: String,
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

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum WebhookPayload {
    #[serde(rename = "ping")]
    PingEvent {
        happened_at: DateTime<FixedOffset>,
        id: Uuid,
        webhook: Webhook,
    },
    #[serde(rename = "workflow-completed")]
    WorkflowCompleted {
        id: Uuid,
        happened_at: String,
        webhook: Webhook,
        workflow: Workflow,
        pipeline: Pipeline,
        project: Project,
        organization: Organization,
    },
    #[serde(rename = "job-completed")]
    JobCompleted {
        happened_at: String,
        pipeline: Pipeline,
        webhook: Webhook,
        organization: Organization,
        workflow: Workflow,
        project: Project,
        id: Uuid,
        job: Job,
    },
}

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
