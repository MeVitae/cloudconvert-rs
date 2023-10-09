//! Types related to managing tasks. See the [crate level docs](crate) for a nice list of available
//! tasks.

use std::borrow::Cow;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::Format;

/// The input of a task: either a single task, or a list of multiple tasks.
///
/// This implements `From<T>` for most sensible `T`.
#[derive(Debug)]
pub enum Input<'a, 'b> {
    Single(Cow<'a, str>),
    List(Cow<'b, [Cow<'a, str>]>),
}

impl<'a, 'b> Input<'a, 'b> {
    /// Convert `Input<'a, 'b> into Input<'static, 'static>`.
    pub fn into_static(self) -> Input<'static, 'static> {
        match self {
            Input::Single(Cow::Owned(s)) => Input::Single(Cow::Owned(s)),
            Input::Single(Cow::Borrowed(s)) => Input::Single(Cow::Owned(s.to_string())),
            Input::List(Cow::Owned(items)) => Input::List(Cow::Owned(
                items
                    .into_iter()
                    .map(Cow::into_owned)
                    .map(Cow::Owned)
                    .collect(),
            )),
            Input::List(Cow::Borrowed(items)) => Input::List(Cow::Owned(
                items
                    .iter()
                    .map(std::ops::Deref::deref)
                    .map(str::to_string)
                    .map(Cow::Owned)
                    .collect(),
            )),
        }
    }
}

impl<'a, 'b> Serialize for Input<'a, 'b> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Input::Single(item) => serializer.serialize_str(item),
            Input::List(items) => Serialize::serialize(items, serializer),
        }
    }
}

impl<'a, 'b> From<&'a str> for Input<'a, 'b> {
    fn from(s: &'a str) -> Input<'a, 'b> {
        Input::Single(Cow::Borrowed(s))
    }
}

impl<'a, 'b> From<String> for Input<'a, 'b> {
    fn from(s: String) -> Input<'a, 'b> {
        Input::Single(Cow::Owned(s))
    }
}

impl<'a, 'b, T: AsRef<str>> From<&'a T> for Input<'a, 'b> {
    fn from(s: &'a T) -> Input<'a, 'b> {
        Input::Single(Cow::Borrowed(s.as_ref()))
    }
}

impl<'a, 'b> From<Vec<String>> for Input<'a, 'b> {
    fn from(items: Vec<String>) -> Input<'a, 'b> {
        Input::List(Cow::Owned(items.into_iter().map(Cow::Owned).collect()))
    }
}

impl<'a, 'b> From<&'a [String]> for Input<'a, 'b> {
    fn from(items: &'a [String]) -> Input<'a, 'b> {
        Input::List(Cow::Owned(
            items
                .iter()
                .map(std::ops::Deref::deref)
                .map(Cow::Borrowed)
                .collect(),
        ))
    }
}

impl<'a, 'b, 'c: 'a> From<&'c [&'a str]> for Input<'a, 'b> {
    fn from(items: &'c [&'a str]) -> Input<'a, 'b> {
        Input::List(Cow::Owned(
            items.iter().map(|item| Cow::Borrowed(*item)).collect(),
        ))
    }
}

impl<'a, 'b> From<Vec<Cow<'a, str>>> for Input<'a, 'b> {
    fn from(items: Vec<Cow<'a, str>>) -> Input<'a, 'b> {
        Input::List(Cow::Owned(items))
    }
}

impl<'a, 'b> From<&'b [Cow<'a, str>]> for Input<'a, 'b> {
    fn from(items: &'b [Cow<'a, str>]) -> Input<'a, 'b> {
        Input::List(Cow::Borrowed(items))
    }
}

macro_rules! make_task_types {
    //(__field_type opt: str) => {
    //    make_task_types!(__field_type opt $field_name: Cow<'a str>);
    //};
    (__field_type opt: $field_type:ty) => {
        Option<$field_type>
    };
    //(__field_type req: str) => {
    //    make_task_types!(__field_type req: Cow<'a str>);
    //};
    (__field_type req: $field_type:ty) => {
        $field_type
    };

    (
        $(#[$task_enum_meta:meta])*
        $enum_vis:vis enum $Task:ident<'a>;

        $(
            $(#[$task_meta:meta])*
            $struct_vis:vis struct $TaskName:ident<'a> {
                operation: $operation:expr,
                $(
                    $(#[$req_field_meta:meta])*
                    req $req_field_name:ident: $req_field_type:ty,
                )*
                $(
                    opt $opt_field_name:ident: $opt_field_type:ty,
                )*
            }
            //$(=> struct $TaskJsonName:ident<'a> {
            //    $(
            //        $(#[$json_field_meta:meta])*
            //        $json_field_name:ident: $json_field_type:ty,
            //    )*
            //})?
        )*
    ) => {
        $(#[$task_enum_meta])*
        $enum_vis enum $Task<'a> {
            $(
                $TaskName($TaskName<'a>),
            )*
        }

        impl<'a> $Task<'a> {
            /// Returns the operation name of this task. For example `convert` or `import/url`.
            pub fn operation(&self) -> &'static str {
                match self {
                    $($Task::$TaskName(_) => $operation,)*
                }
            }

            /// Convert a job to an object to be included in a job call.
            ///
            /// This includes the fields of the task, with an additional `"operation"` field,
            /// containing the operation name.
            pub fn to_job_task(&self) -> serde_json::Result<serde_json::Value> {
                match self {
                    $($Task::$TaskName(task) => task.to_job_task(),)*
                }
            }
        }

        $(
            $(#[$task_meta])*
            #[derive(serde::Serialize, Debug)]
            $struct_vis struct $TaskName<'a> {
                $(
                    $(#[$req_field_meta])*
                    $struct_vis $req_field_name: make_task_types!(__field_type req: $req_field_type),
                )*
                $(
                    // $(#[$opt_field_meta])*
                    #[serde(skip_serializing_if = "Option::is_none")]
                    $struct_vis $opt_field_name: make_task_types!(__field_type opt: $opt_field_type),
                )*
            }

            //$(
            //    #[derive(serde::Serialize, Debug)]
            //    struct $TaskJsonName<'a> {
            //        $(#[$json_field_meta])*
            //        $($json_field_name: $json_field_type,)*
            //    }
            //)?

            hapic::json_api_call!(json <'a> ($crate::ApiCall) $operation: $TaskName<'a> as $TaskName<'a> => TasksOutput as Status);

            impl<'a> From<$TaskName<'a>> for $Task<'a> {
                fn from(task: $TaskName<'a>) -> $Task<'a> {
                    $Task::$TaskName(task)
                }
            }

            impl<'a> $TaskName<'a> {
                /// Convert a job to an object to be included in a job call.
                ///
                /// This includes the fields of the task, with an additional `"operation"` field,
                /// containing the operation name.
                pub fn to_job_task(&self) -> serde_json::Result<serde_json::Value> {
                    let task = self;
                    //$(
                    //    let task = $TaskJsonName::from(self);
                    //    let task = &task;
                    //)?
                    let mut value = serde_json::to_value(task)?;
                    match &mut value {
                        serde_json::Value::Object(value) => {
                            value.insert(
                                "operation".to_string(),
                                serde_json::Value::String($operation.to_string()),
                            );
                        },
                        _ => unreachable!(),
                    }
                    Ok(value)
                }
            }
        )*
    }
}

impl<'a> Task<'a> {
    pub fn task_uri(&self, endpoint: &str) -> String {
        format!("{endpoint}/{}", self.operation())
    }
}

make_task_types!(
    /// A task. See the [crate level docs](crate) for a nice list of available tasks.
    pub enum Task<'a>;

    /// Import a file by downloading it from a URL.
    ///
    /// Docs: [api/v2/import#import-url-tasks](https://cloudconvert.com/api/v2/import#import-url-tasks)
    pub struct ImportUrl<'a> {
        operation: "import/url",

        req url: Cow<'a, str>,
        opt filename: Cow<'a, str>,
        opt headers: HashMap<String, String>,
    }

    /// Import a document from an S3 compatible bucket.
    ///
    /// Docs: [api/v2/import#import-s3-tasks](https://cloudconvert.com/api/v2/import#import-s3-tasks)
    pub struct ImportS3<'a> {
        operation: "import/s3",

        req bucket: Cow<'a, str>,
        req region: Cow<'a, str>,
        req access_key_id: Cow<'a, str>,
        req secret_access_key: Cow<'a, str>,
        opt endpoint: Cow<'a, str>,
        opt key: Cow<'a, str>,
        opt key_prefix: Cow<'a, str>,
        opt session_token: Cow<'a, str>,
        opt filename: Cow<'a, str>,
    }

    /// Import a document from Azure Blob Storage.
    ///
    /// Docs: [api/v2/import#import-azure-blob-tasks](https://cloudconvert.com/api/v2/import#import-azure-blob-tasks)
    pub struct ImportAzureBlob<'a> {
        operation: "import/azure/blob",

        req storage_account: Cow<'a, str>,
        req container: Cow<'a, str>,
        opt storage_access_key: Cow<'a, str>,
        opt sas_token: Cow<'a, str>,
        opt blob: Cow<'a, str>,
        opt blob_prefix: Cow<'a, str>,
        opt filename: Cow<'a, str>,
    }

    /// Import a document from Google Cloud Storage.
    ///
    /// Docs: [api/v2/import#import-google-cloud-storage-tasks](https://cloudconvert.com/api/v2/import#import-google-cloud-storage-tasks)
    pub struct ImportGoogleCloud<'a> {
        operation: "import/google-cloud-storage",

        req project_id: Cow<'a, str>,
        req bucket: Cow<'a, str>,
        req client_email: Cow<'a, str>,
        req private_key: Cow<'a, str>,
        opt file: Cow<'a, str>,
        opt file_prefix: Cow<'a, str>,
        opt filename: Cow<'a, str>,
    }

    /// Import a document from OpenStack Object Storage (Swift).
    ///
    /// Docs: [api/v2/import#import-openstack-tasks](https://cloudconvert.com/api/v2/import#import-openstack-tasks)
    pub struct ImportOpenStack<'a> {
        operation: "import/openstack",

        req auth_url: Cow<'a, str>,
        req username: Cow<'a, str>,
        req password: Cow<'a, str>,
        req region: Cow<'a, str>,
        req container: Cow<'a, str>,
        opt tenant_name: Cow<'a, str>,
        opt file: Cow<'a, str>,
        opt file_prefix: Cow<'a, str>,
        opt filename: Cow<'a, str>,
    }

    /// Import a document from an SFTP server.
    ///
    /// Docs: [api/v2/import#import-sftp-tasks](https://cloudconvert.com/api/v2/import#import-sftp-tasks)
    pub struct ImportSFTP<'a> {
        operation: "import/sftp",

        req host: Cow<'a, str>,
        req username: Cow<'a, str>,
        req password: Cow<'a, str>,
        opt port: u16,
        opt private_key: Cow<'a, str>,
        opt file: Cow<'a, str>,
        opt path: Cow<'a, str>,
        opt filename: Cow<'a, str>,
    }

    /// Convert a document.
    ///
    /// Docs: [api/v2/convert](https://cloudconvert.com/api/v2/convert)
    ///
    /// **Note:** If you're importing from a URL, converting, then exporting to a URL, you can use
    /// [`crate::ImportConvertExport`] instead!
    pub struct Convert<'a> {
        operation: "convert",

        /// The ID of the task, or tasks, to convert.
        ///
        /// This is most likely an import task.
        req input: Input<'a, 'a>,
        req output_format: Format,
        opt input_format: Format,
        opt filename: Cow<'a, str>,
        opt engine: Cow<'a, str>,
        opt engine_version: Cow<'a, str>,
        opt timeout: u32,
    }

    /// Optimize/compress a PDF, PNG or JPG file.
    ///
    /// Docs: [api/v2/optimize](https://cloudconvert.com/api/v2/optimize)
    pub struct Optimize<'a> {
        operation: "optimize",

        /// The ID of the task, or tasks, to optimize.
        ///
        /// This is most likely an import task.
        req input: Input<'a, 'a>,
        opt input_format: Format,
        opt profile: OptimizationProfile,
        opt flatten_signatures: bool,
        opt colorspace: Colorspace,
        opt filename: Cow<'a, str>,
        opt engine: Cow<'a, str>,
        opt engine_version: Cow<'a, str>,
        opt timeout: u32,
    }

    /// Add a watermark to a PDF, image, or video.
    ///
    /// Docs: [api/v2/watermark](https://cloudconvert.com/api/v2/watermark)
    pub struct Watermark<'a> {
        operation: "watermark",

        /// The ID of the task, or tasks, to watermark.
        ///
        /// This is most likely an import task.
        req input: Input<'a, 'a>,
        opt input_format: Format,
        opt pages: Cow<'a, str>,
        opt layer: WatermarkLayer,
        opt text: Cow<'a, str>,
        opt font_size: u16,
        opt font_width_percent: u8,
        opt font_color: Cow<'a, str>,
        opt font_name: Cow<'a, str>,
        opt font_align: FontAlign,
        opt image: Cow<'a, str>,
        opt image_width: u32,
        opt image_width_percent: u8,
        opt position_vertical: VerticalPosition,
        opt position_horizontal: HorizontalPosition,
        opt margin_vertical: u32,
        opt opacity: u8,
        opt rotation: u16,
        opt filename: Cow<'a, str>,
        opt engine: Cow<'a, str>,
        opt engine_version: Cow<'a, str>,
        opt timeout: u32,
    }

    /// Create a thumbnail from a single input (PNG, JPG or WEBP).
    ///
    /// Docs: [api/v2/thumbnail](https://cloudconvert.com/api/v2/thumbnail)
    pub struct Thumbnail<'a> {
        operation: "thumbnail",

        /// The ID of the task, or tasks, to archive.
        ///
        /// This is most likely an import task.
        req input: Cow<'a, str>,
        /// This should be PNG, JPG or WEBP.
        req output_format: Format,
        opt input_format: Format,
        opt width: u32,
        opt height: u32,
        opt fit: ThumbnailFit,
        opt count: u32,
        opt timestamp: Cow<'a, str>,
        opt filename: Cow<'a, str>,
        opt engine: Cow<'a, str>,
        opt engine_version: Cow<'a, str>,
        opt timeout: u32,
    }

    /// Merge files into a single PDF.
    ///
    /// Docs: [api/v2/merge](https://cloudconvert.com/api/v2/merge)
    pub struct Merge<'a> {
        operation: "merge",

        /// The ID of the task, or tasks, to archive.
        ///
        /// This is most likely an import task.
        req input: Input<'a, 'a>,
        /// This should be `pdf`.
        req output_format: Format,
        opt filename: Cow<'a, str>,
        opt engine: Cow<'a, str>,
        opt engine_version: Cow<'a, str>,
        opt timeout: u32,
    }

    /// Create zip, rar, 7z, or tar archives.
    ///
    /// Docs: [api/v2/archive](https://cloudconvert.com/api/v2/archive)
    pub struct Archive<'a> {
        operation: "archive",

        /// The ID of the task, or tasks, to archive.
        ///
        /// This is most likely an import task.
        req input: Input<'a, 'a>,
        /// This should be `zip`, `rar`, `7z`, `tar`, `tar.gz`, or `tar.bz2`.
        req output_format: Format,
        opt filename: Cow<'a, str>,
        opt engine: Cow<'a, str>,
        opt engine_version: Cow<'a, str>,
        opt timeout: u32,
    }

    /// Capture a website as a PDF or image.
    ///
    /// Docs: [api/v2/capture-website](https://cloudconvert.com/api/v2/capture-website)
    pub struct Capture<'a> {
        operation: "capture-website",

        /// The URL to capture.
        req url: Cow<'a, str>,
        /// This should be `pdf`, `png` or `jpg`.
        req output_format: Format,
        req print_background: bool,
        req display_header_footer: bool,
        #[serde(skip_serializing_if = "HashMap::is_empty")]
        req headers: HashMap<String, String>,
        opt pages: Cow<'a, str>,
        opt zoom: f32,
        opt page_width: f32,
        opt page_height: f32,
        opt margin_top: f32,
        opt margin_bottom: f32,
        opt margin_left: f32,
        opt header_template: Cow<'a, str>,
        opt footer_template: Cow<'a, str>,
        opt wait_until: CaptureWaitUntil,
        opt wait_for_element: Cow<'a, str>,
        opt wait_time: u32,
        opt css_media_type: CssMediaType,
        opt filename: HashMap<String, String>,
        opt engine: Cow<'a, str>,
        opt engine_version: Cow<'a, str>,
        opt timeout: u32,
    }

    /// Generate a temporary URL to download files.
    ///
    /// Docs [api/v2/export#export-url-tasks](https://cloudconvert.com/api/v2/export#export-url-tasks)
    pub struct ExportUrl<'a> {
        operation: "export/url",

        /// The ID of the task, or tasks, to export.
        req input: Input<'a, 'a>,
        req inline: bool,
        req archive_multiple_files: bool,
    }

    /// Export files to an S3 compatible bucket.
    ///
    /// Docs [api/v2/export#export-s3-tasks](https://cloudconvert.com/api/v2/export#export-s3-tasks)
    pub struct ExportS3<'a> {
        operation: "export/s3",

        /// The ID of the task, or tasks, to export.
        req input: Input<'a, 'a>,
        req bucket: Cow<'a, str>,
        req region: Cow<'a, str>,
        req access_key_id: Cow<'a, str>,
        req secret_access_key: Cow<'a, str>,
        opt endpoint: Cow<'a, str>,
        opt key: Cow<'a, str>,
        opt key_prefix: Cow<'a, str>,
        opt session_token: Cow<'a, str>,
        opt acl: Cow<'a, str>,
        opt cache_control: Cow<'a, str>,
        opt content_disposition: Cow<'a, str>,
        opt content_type: Cow<'a, str>,
        opt metadata: serde_json::Value,
        opt server_side_encrpytion: Cow<'a, str>,
        opt tagging: serde_json::Value,
    }

    /// Export files to Azure Blob Storage
    ///
    /// Docs [api/v2/export#export-azure-blob-tasks](https://cloudconvert.com/api/v2/export#export-azure-blob-tasks)
    pub struct ExportAzureBlob<'a> {
        operation: "export/azure/blob",

        /// The ID of the task, or tasks, to export.
        req input: Input<'a, 'a>,
        req storage_account: Cow<'a, str>,
        req container: Cow<'a, str>,
        opt storage_access_key: Cow<'a, str>,
        opt sas_token: Cow<'a, str>,
        opt blob: Cow<'a, str>,
        opt blob_prefix: Cow<'a, str>,
        opt metadata: serde_json::Value,
    }

    /// Export files to Google Cloud Storage
    ///
    /// Docs [api/v2/export#export-google-cloud-storage-tasks](https://cloudconvert.com/api/v2/export#export-google-cloud-storage-tasks)
    pub struct ExportGoogleCloud<'a> {
        operation: "export/google-cloud-storage",

        /// The ID of the task, or tasks, to export.
        req input: Input<'a, 'a>,
        req project_id: Cow<'a, str>,
        req bucket: Cow<'a, str>,
        req client_email: Cow<'a, str>,
        req private_key: Cow<'a, str>,
        opt file: Cow<'a, str>,
        opt file_prefix: Cow<'a, str>,
    }

    /// Export files to OpenStack Object Storage (Swift)
    ///
    /// Docs [api/v2/export#export-openstack-tasks](https://cloudconvert.com/api/v2/export#export-openstack-tasks)
    pub struct ExportOpenStack<'a> {
        operation: "export/openstack",

        /// The ID of the task, or tasks, to export.
        req input: Input<'a, 'a>,
        req auth_url: Cow<'a, str>,
        req username: Cow<'a, str>,
        req password: Cow<'a, str>,
        req region: Cow<'a, str>,
        req container: Cow<'a, str>,
        opt tenant_name: Cow<'a, str>,
        opt file: Cow<'a, str>,
        opt file_prefix: Cow<'a, str>,
    }

    /// Export files to an SFTP server.
    ///
    /// Docs [api/v2/export#export-sftp-tasks](https://cloudconvert.com/api/v2/export#export-sftp-tasks)
    pub struct ExportSFTP<'a> {
        operation: "export/sftp",

        /// The ID of the task, or tasks, to export.
        req input: Input<'a, 'a>,
        req host: Cow<'a, str>,
        req username: Cow<'a, str>,
        opt port: u16,
        opt password: Cow<'a, str>,
        opt private_key: Cow<'a, str>,
        opt file: Cow<'a, str>,
        opt path: Cow<'a, str>,
    }
);

/// Enum for the `font_align` property of [`Watermark`] tasks.
///
/// Docs: [api/v2/watermark](https://cloudconvert.com/api/v2/watermark)
#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum FontAlign {
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "right")]
    Right,
    #[serde(rename = "center")]
    Center,
}

/// Enum for the `layer` property of [`Watermark`] tasks.
///
/// Docs: [api/v2/watermark](https://cloudconvert.com/api/v2/watermark)
#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum WatermarkLayer {
    #[serde(rename = "above")]
    Above,
    #[serde(rename = "below")]
    Below,
}

/// Enum for the `position_vertical` property of [`Watermark`] tasks.
///
/// Docs: [api/v2/watermark](https://cloudconvert.com/api/v2/watermark)
#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum VerticalPosition {
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "right")]
    Right,
    #[serde(rename = "center")]
    Center,
}

/// Enum for the `position_horizontal` property of [`Watermark`] tasks.
///
/// Docs: [api/v2/watermark](https://cloudconvert.com/api/v2/watermark)
#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum HorizontalPosition {
    #[serde(rename = "top")]
    Top,
    #[serde(rename = "bottom")]
    Bottom,
    #[serde(rename = "center")]
    Center,
}

/// Enum for the `fit` property of [`Thumbnail`] tasks.
///
/// Docs: [api/v2/thumbnail](https://cloudconvert.com/api/v2/thumbnail)
#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum ThumbnailFit {
    #[serde(rename = "max")]
    Max,
    #[serde(rename = "crop")]
    Crop,
    #[serde(rename = "scale")]
    Scale,
}

/// Enum for the `profile` property of [`Optimize`] tasks.
///
/// Docs: [api/v2/optimize](https://cloudconvert.com/api/v2/optimize)
#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum OptimizationProfile {
    #[serde(rename = "web")]
    Web,
    #[serde(rename = "print")]
    Print,
    #[serde(rename = "archive")]
    Archive,
    #[serde(rename = "mrc")]
    Mrc,
    #[serde(rename = "max")]
    Max,
}

/// Enum for the `colorspace` property of [`Optimize`] tasks.
///
/// Docs: [api/v2/optimize](https://cloudconvert.com/api/v2/optimize)
#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum Colorspace {
    #[serde(rename = "unchanged")]
    Unchanged,
    #[serde(rename = "rgb")]
    RGB,
    #[serde(rename = "cmyk")]
    CMYK,
    #[serde(rename = "greyscale")]
    Greyscale,
}

/// Enum for the `wait_until` property of [`Capture`] tasks.
///
/// Docs: [api/v2/capture-website](https://cloudconvert.com/api/v2/capture-website)
#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum CaptureWaitUntil {
    #[serde(rename = "Load")]
    Load,
    #[serde(rename = "domcontentloaded")]
    DOMContentLoaded,
    #[serde(rename = "networkidle0")]
    NetworkIdle0,
    #[serde(rename = "networkidle2")]
    NetworkIdle2,
}

/// Enum for the `css_media_type` property of [`CssMediaType`] tasks.
///
/// Docs: [api/v2/capture-website](https://cloudconvert.com/api/v2/capture-website)
#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum CssMediaType {
    #[serde(rename = "print")]
    Print,
    #[serde(rename = "screen")]
    Screen,
}

#[doc(hidden)]
#[derive(Deserialize)]
pub struct TasksOutput {
    pub data: Status,
}

impl From<TasksOutput> for Status {
    fn from(output: TasksOutput) -> Status {
        output.data
    }
}

/// The status of a task.
///
/// Docs: [api/v2/tasks](https://cloudconvert.com/api/v2/tasks#tasks-show)
#[derive(Debug, Deserialize)]
pub struct Status {
    /// The task ID
    pub id: String,

    /// The ID of the job containing this task, if there is one.
    #[serde(default)]
    pub job_id: Option<String>,

    /// The name of the task, if it's part of a job.
    #[serde(default)]
    pub name: Option<String>,

    /// The name of the operation, for example `convert` or `export/url`.
    pub operation: String,

    /// The status of the task.
    pub status: super::Status,

    /// The status message for the task.
    ///
    /// If the `status` is `Error`, then this contains the error details.
    #[serde(default)]
    #[serde(rename = "message")]
    pub status_message: Option<String>,

    /// If `status` is `Error`, the error code.
    #[serde(default)]
    #[serde(rename = "code")]
    pub error_code: Option<String>,

    /// If `status` is `Finished`, the number of credits consumed.
    #[serde(default)]
    #[serde(rename = "code")]
    pub credits: Option<u16>,

    // TODO: started_at
    // TODO: ended_at
    // TODO: depends_on_tasks
    /// If this task is a retry, the original task ID.
    #[serde(default)]
    pub retry_of_task_id: Option<String>,

    /// The list of task IDs that are retries of this task.
    ///
    /// This isn't available unless the `include` field on the `Show` request is `retries`.
    #[serde(default)]
    pub retries: Vec<String>,

    #[serde(default)]
    /// The engine used
    pub engine: Option<String>,

    #[serde(default)]
    /// The engine version used
    pub engine_version: Option<String>,

    /// The payload submitted to the task.
    #[serde(default)]
    pub payload: serde_json::Value,

    /// The results of the task.
    #[serde(default)]
    pub result: Option<HashMap<String, serde_json::Value>>,

    #[serde(default)]
    pub links: Option<HashMap<String, String>>,
}
