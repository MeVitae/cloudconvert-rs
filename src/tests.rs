#[test]
fn test_event_kind_deserialize() {
    use crate::webhook::EventKind;
    assert_eq!(
        serde_json::from_str::<EventKind>("\"job.created\"").unwrap(),
        EventKind::JobCreated
    );
    assert_eq!(
        serde_json::from_str::<EventKind>("\"job.finished\"").unwrap(),
        EventKind::JobFinished
    );
    assert_eq!(
        serde_json::from_str::<EventKind>("\"job.failed\"").unwrap(),
        EventKind::JobFailed
    );
    assert_ne!(
        serde_json::from_str::<EventKind>("\"job.created\"").unwrap(),
        EventKind::JobFailed
    );
}

#[test]
fn test_font_align_serialize() {
    use crate::task::FontAlign;
    assert_eq!(serde_json::to_string(&FontAlign::Left).unwrap(), "\"left\"",);
    assert_eq!(
        serde_json::to_string(&FontAlign::Right).unwrap(),
        "\"right\"",
    );
    assert_eq!(
        serde_json::to_string(&FontAlign::Center).unwrap(),
        "\"center\"",
    );
}

#[test]
fn test_format_serialize() {
    use crate::Format;
    assert_eq!(serde_json::to_string(&Format::Pdf).unwrap(), "\"pdf\"");
    assert_ne!(serde_json::to_string(&Format::Doc).unwrap(), "\"pdf\"");
    assert_eq!(serde_json::to_string(&Format::Doc).unwrap(), "\"doc\"");
    assert_eq!(serde_json::to_string(&Format::Docx).unwrap(), "\"docx\"");
}

#[test]
fn test_webhook_parsing_and_verification() {
    use crate::webhook::{Event, EventKind, ParseError};

    let example_webhook = r#"{
  "event": "job.finished",
  "job": {
    "id": "4b6ee8e2-e293-4805-b48e-a03876d1ec66",
    "tag": "myjob-123",
    "status": null,
    "created_at": "2019-04-13T21:18:47+00:00",
    "started_at": null,
    "ended_at": null,
    "tasks": [
      {
        "id": "acdf8096-10a1-4ab7-b009-539f5f329cad",
        "name": "export-1",
        "operation": "export/url",
        "status": "finished",
        "message": null,
        "percent": 100,
        "result": {
          "files": [
            {
              "filename": "file.pdf",
              "url": "https://storage.cloudconvert.com/eed87242-577e-4e3e-8178-9edbe51975dd/file.pdf?temp_url_sig=79c2db4d884926bbcc5476d01b4922a19137aee9&temp_url_expires=1545962104"
            }
          ]
        },
        "created_at": "2019-04-13T21:18:47+00:00",
        "started_at": "2019-04-13T21:18:47+00:00",
        "ended_at": "2019-04-13T21:18:47+00:00",
        "depends_on_task_ids": [
        ],
        "links": {
          "self": "https://api.cloudconvert.com/v2/tasks/acdf8096-10a1-4ab7-b009-539f5f329cad"
        }
      }
    ],
    "links": {
      "self": "https://api.cloudconvert.com/v2/jobs/4b6ee8e2-e293-4805-b48e-a03876d1ec66"
    }
  }
}"#;
    assert!(matches!(
        Event::from_json(
            example_webhook.as_bytes(),
            "363495aa6b142fa06a3015aa7cb53fec870ebece9fa7cc35b99409685ba250",
            &[1]
        ),
        Err(ParseError::HexDecodeSignature(_))
    ));
    assert!(matches!(
        Event::from_json(
            example_webhook.as_bytes(),
            "363495aa6b142fa06a3015aa7cb53fec870ebece9fa7cc35b99409685ba250da",
            &[1]
        ),
        Err(ParseError::SignatureMismatch)
    ));

    // Now generate a valid signature
    use hmac::{Hmac, Mac};
    let signing_secret = &[1, 2, 3, 9, 2];
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(signing_secret).unwrap();
    mac.update(example_webhook.as_bytes());
    let signature = mac.finalize();
    let mut signature: [u8; 32] = signature.into_bytes().into();
    let signature_string = hex::encode(&signature);
    let event = Event::from_json(
        example_webhook.as_bytes(),
        &signature_string,
        signing_secret,
    )
    .unwrap();
    assert_eq!(event.event, EventKind::JobFinished);
    assert_eq!(event.job.id, "4b6ee8e2-e293-4805-b48e-a03876d1ec66");
    assert_eq!(
        event.job.tasks[0].id,
        "acdf8096-10a1-4ab7-b009-539f5f329cad"
    );
    assert_eq!(
        event.job.tasks[0].result.as_ref().unwrap()["files"]
            .get(0)
            .unwrap()
            .get("filename"),
        Some(&serde_json::Value::String("file.pdf".to_string()))
    );

    // Break the signature
    signature[3] = 0;
    let signature_string = hex::encode(&signature);
    assert!(matches!(
        Event::from_json(
            example_webhook.as_bytes(),
            &signature_string,
            signing_secret
        ),
        Err(ParseError::SignatureMismatch)
    ));
}

#[allow(unused)]
//#[tokio::test]
async fn test_client() {
    use crate::{task::ImportUrl, Client, Format, ImportConvertExport};
    use std::borrow::Cow;

    let bearer_token = std::env::vars().find(|(name, _)| name == "CLOUDCONVERT_TOKEN").unwrap().1;
    let mut client = Client::default_client(&bearer_token);
    let call = ImportConvertExport {
        tag: Some(Cow::Borrowed("test")),
        webhook_url: None,
        import: ImportUrl {
            url: Cow::Borrowed(
                "https://storage.googleapis.com/mevitae-external-pdf-results/fail.pdf",
            ),
            filename: None,
            headers: None,
        },
        input_format: Format::Pdf,
        output_format: Format::Doc,
        export_inline: false,
        timeout: None,
    };
    dbg!(&call);
    println!("{:?}", client.call(call).await.unwrap());
}
