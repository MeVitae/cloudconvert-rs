//! ## Creating a client
//!
//! To create a [`Client`] using a bearer token, then create a job:
//!
//! ```
//! # async fn example() {
//! use cloudconvert::{Client, job};
//! let client = Client::default_client("your_bearer_token");
//! client.call(job::Create{
//!     tasks: todo!(),
//!     tag: Some("your_tag".into()),
//!     webhook_url: None,
//! }).await;
//! # }
//! ```
//!
//! ## Jobs
//!
//! Jobs can be crated using the [`job::Create`] API call. Jobs consist of a number of named
//! [`task`]s.
//!
//! ## Tasks
//!
//! A [`task::Task`] enum represents one of the following tasks:
//!
//! ### Import tasks
//!
//! - [`task::ImportUrl`]: Import a document from a URL.
//! - [`task::ImportS3`]: Import a document from an S3 compatible bucket.
//! - [`task::ImportAzureBlob`]: Import a document from Azure Blob Storage.
//! - [`task::ImportGoogleCloud`]: Import a document from Google Cloud Storage.
//! - [`task::ImportOpenStack`]: Import a document from OpenStack Object Storage (Swift).
//! - [`task::ImportSFTP`]: Import a document from an SFTP server.
//!
//! ### Processing tasks
//!
//! - [`task::Convert`]: Convert a document!
//! - [`task::Optimize`]: Optimize a document.
//! - [`task::Watermark`]: Watermark a document.
//! - [`task::Thumbnail`]: Add a thumbnail to a document.
//! - [`task::Merge`]: Merge multiple documents.
//! - [`task::Archive`]: Create zip, rar, 7z or tar archives.
//! - [`task::Capture`]: Capture a website.
//!
//! ### Export tasks
//!
//! - [`task::ExportUrl`]: Export a document to a URL.
//! - [`task::ExportS3`]: Export documents to an S3 compatible bucket.
//! - [`task::ExportAzureBlob`]: Export documents to Azure Blob Storage.
//! - [`task::ExportGoogleCloud`]: Export documents to Google Cloud Storage.
//! - [`task::ExportOpenStack`]: Export documents to OpenStack Object Storage (Swift).
//! - [`task::ExportSFTP`]: Export documents to an SFTP server.
//!
//! ## Pre-made jobs
//!
//! - [`ImportConvertExport`]: An API call (underneath, `job::Create`) which creates a job
//!   consisting of an import task, convert task, then export task.
//!
//! ## Webhooks
//!
//! Tools for verifying and parsing webhooks can be found within the [`webhook`] module.

use std::borrow::Cow;
use std::collections::HashMap;

use serde::Deserialize;

#[cfg(test)]
mod tests;

mod format;
pub mod job;
pub mod task;
pub mod webhook;

pub use format::Format;
use task::Task;

hapic::json_api!(
    /// The API client. This is used to call items implementing [`ApiCall`]. Usually, this is
    /// [`job::Create`], but it can be any [`task::Task`].
    ///
    /// You should construct a client using [`Client::default_client`], for example:
    ///
    /// ```
    /// # async fn example() {
    /// use cloudconvert::{Client, job};
    /// let client = Client::default_client("your_bearer_token");
    /// client.call(job::Create{
    ///     tasks: todo!(),
    ///     tag: Some("your_tag".into()),
    ///     webhook_url: None,
    /// }).await;
    /// # }
    /// ```
    pub struct Client<B, T: Transport<B>>;

    /// The trait implemented by all the API calls. Execute these using a [`Client`].
    pub trait ApiCall;

    json {
        <'a> "/jobs": job::Create<'a> as job::CreateJobRequest<'a> => job::JobsOutput as job::Job;
        <'a> "/jobs": ImportConvertExport<'a> as job::CreateJobRequest<'a> => job::JobsOutput as job::Job;
    }
);

/// A [`Client`] using [`hyper`] for transport.
pub type HyperClient = Client<hyper::Body, hapic::transport::HttpsTransport>;

impl HyperClient {
    pub fn default_client(bearer_token: &str) -> HyperClient {
        let mut client = Client::new(Cow::Borrowed("https://api.cloudconvert.com/v2"));
        client.client.authorization =
            Some(("Bearer ".to_string() + bearer_token).try_into().unwrap());
        client
    }
}

/// Status of a [`job::Job`] or [`task::Status`].
#[derive(Debug, Deserialize)]
pub enum Status {
    #[serde(rename = "waiting")]
    Waiting,
    #[serde(rename = "processing")]
    Processing,
    #[serde(rename = "finished")]
    Finished,
    #[serde(rename = "error")]
    Error,
}

/// An API call, which underneath converts to a [`job::Create`] call, to import from a URL,
/// convert, and then export to a URL.
#[derive(Debug)]
pub struct ImportConvertExport<'a> {
    /// The tag to apply to the created job.
    pub tag: Option<Cow<'a, str>>,

    /// The import task to use.
    pub import: task::ImportUrl<'a>,

    /// The input format to convert from.
    pub input_format: Format,

    /// The format to convert to.
    pub output_format: Format,

    /// The `export_inline` option for [`task::ExportUrl`].
    pub export_inline: bool,

    pub timeout: Option<u32>,
    pub webhook_url: Option<Cow<'a, str>>,
}

impl<'a> From<ImportConvertExport<'a>> for job::Create<'a> {
    fn from(options: ImportConvertExport<'a>) -> job::Create<'a> {
        options.create_job()
    }
}

impl<'a> From<ImportConvertExport<'a>> for job::CreateJobRequest<'a> {
    fn from(options: ImportConvertExport<'a>) -> job::CreateJobRequest<'a> {
        options.create_job().into()
    }
}

impl<'a> ImportConvertExport<'a> {
    /// Convert `self` to a [`job::Create`] call.
    pub fn create_job(self) -> job::Create<'a> {
        let import_id = "import";
        let convert_id = "convert";
        let export_id = "export";
        job::Create {
            tasks: HashMap::from([
                (import_id.to_string(), self.import.into()),
                (
                    convert_id.to_string(),
                    task::Convert {
                        input: import_id.into(),
                        input_format: Some(self.input_format),
                        output_format: self.output_format,
                        filename: None,
                        engine: None,
                        engine_version: None,
                        timeout: self.timeout,
                    }
                    .into(),
                ),
                (
                    export_id.to_string(),
                    task::ExportUrl {
                        input: convert_id.into(),
                        inline: self.export_inline,
                        archive_multiple_files: false,
                    }
                    .into(),
                ),
            ] as [(String, Task); 3]),
            tag: self.tag,
            webhook_url: self.webhook_url,
        }
    }
}
