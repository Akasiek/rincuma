use super::{Notification, NotificationError, NotificationProvider};
use crate::db::NotificationSettings;
use reqwest::header::{AUTHORIZATION, HeaderValue};
use std::{future::Future, pin::Pin};

pub(super) struct Ntfy {
    client: reqwest::Client,
}

impl Ntfy {
    pub(super) fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

impl NotificationProvider for Ntfy {
    fn send<'a>(
        &'a self,
        settings: &'a NotificationSettings,
        notification: &'a Notification,
    ) -> Pin<Box<dyn Future<Output = Result<(), NotificationError>> + Send + 'a>> {
        Box::pin(async move {
            let mut request = self
                .client
                .post(&settings.url)
                .header("Content-Type", "text/plain; charset=utf-8")
                .header("Title", &notification.title)
                .body(notification.message.clone());

            if let Some(value) = &settings.authorization {
                let mut header =
                    HeaderValue::from_str(value).map_err(|_| NotificationError::Configuration)?;
                header.set_sensitive(true);
                request = request.header(AUTHORIZATION, header);
            }

            // Do not retain transport errors: their URLs may contain private topic names.
            let response = request
                .send()
                .await
                .map_err(|_| NotificationError::Transport)?;

            if !response.status().is_success() {
                return Err(NotificationError::Status(response.status()));
            }

            Ok(())
        })
    }
}

#[cfg(test)]
#[allow(clippy::panic_in_result_fn)]
mod tests {
    use super::*;
    use crate::db::NotificationProviderKind;
    use axum::{
        Router,
        body::Bytes,
        http::{HeaderMap, StatusCode},
        routing::post,
    };
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn publishes_utf8_and_auth_and_rejects_non_success_statuses()
    -> Result<(), Box<dyn std::error::Error>> {
        let (sender, mut receiver) = mpsc::channel(4);
        let app = Router::new()
            .route(
                "/tasks",
                post(move |headers: HeaderMap, body: Bytes| {
                    let sender = sender.clone();
                    async move {
                        sender
                            .send((headers, body))
                            .await
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                        Ok::<_, StatusCode>(StatusCode::OK)
                    }
                }),
            )
            .route("/unauthorized", post(|| async { StatusCode::UNAUTHORIZED }))
            .route(
                "/redirect",
                post(|| async { (StatusCode::TEMPORARY_REDIRECT, [("Location", "/tasks")]) }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let server = tokio::spawn(async move { axum::serve(listener, app).await });
        let providers = super::super::Providers::new()?;
        let provider = providers.get(NotificationProviderKind::Ntfy);
        let now = jiff::Timestamp::now();
        let mut settings = NotificationSettings {
            owner_id: 1,
            owner: toasty::Deferred::default(),
            provider: NotificationProviderKind::Ntfy.as_str().to_owned(),
            enabled: true,
            url: format!("http://{address}/tasks"),
            authorization: Some("Bearer secret".to_owned()),
            created_at: now,
            updated_at: now,
        };
        let notification = Notification {
            title: "Task due".to_owned(),
            message: "Prepare report 🦀".to_owned(),
        };
        provider.send(&settings, &notification).await?;
        let (headers, body) = receiver.recv().await.ok_or("missing request")?;
        assert_eq!(
            headers.get("Authorization").ok_or("missing auth")?,
            "Bearer secret"
        );
        assert_eq!(headers.get("Title").ok_or("missing title")?, "Task due");
        assert_eq!(body.as_ref(), notification.message.as_bytes());
        settings.url = format!("http://{address}/unauthorized");
        assert!(matches!(
            provider.send(&settings, &notification).await,
            Err(NotificationError::Status(StatusCode::UNAUTHORIZED))
        ));
        settings.url = format!("http://{address}/redirect");
        assert!(matches!(
            provider.send(&settings, &notification).await,
            Err(NotificationError::Status(StatusCode::TEMPORARY_REDIRECT))
        ));
        assert!(receiver.try_recv().is_err());
        server.abort();
        Ok(())
    }
}
