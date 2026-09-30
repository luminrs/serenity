use std::borrow::Cow;

use reqwest::multipart::{Form, Part};

use crate::builder::CreateAttachment;
use crate::internal::prelude::*;

impl CreateAttachment {
    fn into_part(self) -> Result<Part> {
        let mut part = Part::bytes(self.data);
        part = guess_mime_str(part, &self.filename)?;
        part = part.file_name(self.filename);
        Ok(part)
    }
}

#[derive(Clone, Debug)]
pub enum MultipartUpload {
    /// A file sent with the form data as an individual upload. For example, a sticker.
    File(CreateAttachment),
    /// Files sent with the form as message attachments.
    Attachments(Vec<CreateAttachment>),
}

/// Holder for multipart body. Contains upload data, multipart fields, and payload_json for
/// creating requests with attachments.
#[derive(Clone, Debug)]
pub struct Multipart {
    pub upload: MultipartUpload,
    /// Multipart text fields that are sent with the form data as individual fields. If a certain
    /// endpoint does not support passing JSON body via `payload_json`, this must be used instead.
    pub fields: Vec<(Cow<'static, str>, Cow<'static, str>)>,
    /// JSON body that will set as the form value as `payload_json`.
    pub payload_json: Option<String>,
}

impl Multipart {
    pub(crate) fn build_form(self) -> Result<Form> {
        let mut multipart = Form::new();

        match self.upload {
            MultipartUpload::File(upload_file) => {
                multipart = multipart.part("file", upload_file.into_part()?);
            },
            MultipartUpload::Attachments(attachment_files) => {
                for file in attachment_files {
                    multipart = multipart.part(format!("files[{}]", file.id), file.into_part()?);
                }
            },
        }

        for (name, value) in self.fields {
            multipart = multipart.text(name, value);
        }

        if let Some(payload_json) = self.payload_json {
            multipart = multipart.text("payload_json", payload_json);
        }

        Ok(multipart)
    }
}

fn guess_mime_str(part: Part, filename: &str) -> Result<Part> {
    // This is required for certain endpoints like create sticker, otherwise the Discord API will
    // respond with a 500 Internal Server Error. The mime type chosen is the same as what reqwest
    // does internally when using Part::file(), but it is not done for any of the other methods we
    // use.
    // https://datatracker.ietf.org/doc/html/rfc7578#section-4.4
    let mime_type = mime_guess::from_path(filename).first_or_octet_stream();
    part.mime_str(mime_type.essence_str()).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use futures::StreamExt as _;

    use super::*;

    async fn form_body(multipart: Multipart) -> String {
        let mut stream = multipart.build_form().unwrap().into_stream();
        let mut body = Vec::new();
        while let Some(chunk) = stream.next().await {
            body.extend_from_slice(&chunk.unwrap());
        }
        String::from_utf8_lossy(&body).into_owned()
    }

    /// Ori CV2 fork (F1): with `CreateAttachment::id`, several files passed to a raw `Http` method
    /// get distinct `files[n]` parts instead of all being `files[0]`.
    #[tokio::test]
    async fn multipart_names_files_by_id() {
        let files = vec![
            CreateAttachment::bytes(b"first".to_vec(), "a.png").id(0),
            CreateAttachment::bytes(b"second".to_vec(), "b.png").id(1),
        ];
        let body = form_body(Multipart {
            upload: MultipartUpload::Attachments(files),
            fields: vec![],
            payload_json: Some(r#"{"attachments":[{"id":0},{"id":1}]}"#.into()),
        })
        .await;
        assert!(body.contains(r#"name="files[0]"; filename="a.png""#), "{body}");
        assert!(body.contains(r#"name="files[1]"; filename="b.png""#), "{body}");
        assert!(body.contains(r#"name="payload_json""#));

        // Without the setter every file is `files[0]`, as in 0.12.5.
        let files = vec![
            CreateAttachment::bytes(b"first".to_vec(), "a.png"),
            CreateAttachment::bytes(b"second".to_vec(), "b.png"),
        ];
        let body = form_body(Multipart {
            upload: MultipartUpload::Attachments(files),
            fields: vec![],
            payload_json: None,
        })
        .await;
        assert_eq!(body.matches(r#"name="files[0]""#).count(), 2);
    }
}
