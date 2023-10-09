//! Types for creating and viewing jobs.

use std::borrow::Cow;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{task, Status};

/// Create a job (which consists of a set of named tasks)
///
/// See [cloudconvert.com/api/v2/jobs#jobs-create](https://cloudconvert.com/api/v2/jobs#jobs-create)
///
/// See the implementation of [`crate::ImportConvertExport::create_job`] for an example of creating
/// a job.
pub struct Create<'a> {
    /// The tasks within this job. A map of names to tasks.
    pub tasks: HashMap<String, task::Task<'a>>,
    pub tag: Option<Cow<'a, str>>,
    pub webhook_url: Option<Cow<'a, str>>,
}

/// The actual request body that is sent to the API server for a [`Create`] call.
#[doc(hidden)]
#[derive(Serialize)]
pub struct CreateJobRequest<'a> {
    tasks: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    webhook_url: Option<Cow<'a, str>>,
}

impl<'a> From<Create<'a>> for CreateJobRequest<'a> {
    fn from(call: Create<'a>) -> CreateJobRequest<'a> {
        CreateJobRequest {
            tasks: HashMap::from_iter(
                call.tasks
                    .into_iter()
                    .map(|(key, value)| (key, value.to_job_task().unwrap())),
            ),
            tag: call.tag,
            webhook_url: call.webhook_url,
        }
    }
}

#[doc(hidden)]
#[derive(Deserialize)]
pub struct JobsOutput {
    pub data: Job,
}

impl From<JobsOutput> for Job {
    fn from(output: JobsOutput) -> Job {
        let mut job = output.data;
        // Ensure the task `job_id` field is filled out.
        for task in job.tasks.iter_mut() {
            if let Some(task_job_id) = task.job_id.as_deref() {
                assert_eq!(task_job_id, job.id);
            } else {
                task.job_id = Some(job.id.to_string());
            }
        }
        job
    }
}

/// The status or results of a job.
///
/// Docs: [cloudconvert.com/api/v2/jobs](https://cloudconvert.com/api/v2/jobs#jobs-show)
#[derive(Debug, Deserialize)]
pub struct Job {
    pub id: String,

    #[serde(default)]
    pub tag: Option<String>,

    #[serde(default)]
    pub status: Option<Status>,

    // TODO: created_at, started_at, ended_at
    /// The tasks that are part of this job.
    #[serde(default)]
    pub tasks: Vec<task::Status>,

    #[serde(default)]
    pub links: Option<HashMap<String, String>>,
}

impl Job {
    /// Return the task from the task name, if it exists within the job.
    ///
    /// If multiple tasks exist with the same name, this could return any of them.
    pub fn get_task_by_name<'a>(&'a self, name: &str) -> Option<&'a task::Status> {
        self.tasks
            .iter()
            .find(|task| matches!(&task.name, Some(task_name) if task_name == name))
    }
}
