//! Tools for verifying and parsing webhook calls.

use serde::Deserialize;

use super::job::Job;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum EventKind {
    #[serde(rename = "job.created")]
    JobCreated,
    #[serde(rename = "job.finished")]
    JobFinished,
    #[serde(rename = "job.failed")]
    JobFailed,
}

/// A signed webhook event.
///
/// This can only be publicly created using [`Event::from_json`] which checks the signature, so it's
/// difficult (though not impossible) to accidentally create a false one.
#[derive(Debug)]
pub struct Event {
    pub event: EventKind,
    pub job: Job,
    /// A field to ensure you can't construct this struct publically without using a method that
    /// checks the signature.
    signature: [u8; 32],
}

/// An error returned by [`Event::from_json`].
#[derive(Debug)]
pub enum ParseError {
    /// The signature did not match.
    SignatureMismatch,

    /// The provided signature string could not be parsed.
    HexDecodeSignature(hex::FromHexError),

    /// The webhook event JSON could not be parsed.
    Json(serde_json::Error),
}

impl Event {
    /// Return the signature of the event
    pub fn signature(&self) -> [u8; 32] {
        self.signature
    }

    /// Parse an event and verify the signature.
    ///
    /// The `signature` string should be taken from the `CloudConvert-Signature` HTTP request
    /// header.
    pub fn from_json(
        json: &[u8],
        signature: &str,
        signing_secret: &[u8],
    ) -> Result<Event, ParseError> {
        use hmac::{Hmac, Mac};

        // Parse the input signature
        let mut expected_signature = [0; 32];
        hex::decode_to_slice(signature, &mut expected_signature[..])
            .map_err(ParseError::HexDecodeSignature)?;

        // Compute the actual HMAC
        let mut mac = Hmac::<sha2::Sha256>::new_from_slice(signing_secret).unwrap();
        mac.update(json);
        let actual_signature = mac.finalize();
        let actual_signature: [u8; 32] = actual_signature.into_bytes().into();

        // Check they match
        if actual_signature != expected_signature {
            return Err(ParseError::SignatureMismatch);
        }

        // Parse the event JSON
        #[derive(Deserialize)]
        struct EventJson {
            event: EventKind,
            job: Job,
        }

        let event: EventJson = serde_json::from_slice(json).map_err(ParseError::Json)?;

        Ok(Event {
            event: event.event,
            job: event.job,
            signature: actual_signature,
        })
    }
}
