use anyhow::Result;
use chrono::{DateTime, Utc};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::sync::Mutex;
use tracing::{error, info, warn};

use crate::config::{EmailConfig, NotificationsConfig};
use crate::models::Alert;

pub trait Notifier: Send + Sync {
    fn name(&self) -> &str;
    fn send(&self, alert: &Alert) -> Result<()>;
}

// --- Webhook ---

pub struct WebhookNotifier {
    url: String,
    client: reqwest::Client,
    runtime: tokio::runtime::Handle,
}

impl WebhookNotifier {
    pub fn new(url: String) -> Self {
        Self {
            url,
            client: reqwest::Client::new(),
            runtime: tokio::runtime::Handle::current(),
        }
    }
}

impl Notifier for WebhookNotifier {
    fn name(&self) -> &str {
        "webhook"
    }

    fn send(&self, alert: &Alert) -> Result<()> {
        let url = self.url.clone();
        let client = self.client.clone();
        let payload = serde_json::to_value(alert)?;

        self.runtime.spawn(async move {
            match client.post(&url).json(&payload).send().await {
                Ok(resp) => info!(status = %resp.status(), "Webhook notification sent"),
                Err(e) => error!("Failed to send webhook notification: {}", e),
            }
        });

        Ok(())
    }
}

// --- Desktop ---

pub struct DesktopNotifier;

impl Notifier for DesktopNotifier {
    fn name(&self) -> &str {
        "desktop"
    }

    fn send(&self, alert: &Alert) -> Result<()> {
        let title = format!("Aurora Alert — {}", alert.alert_type);
        let body = format!(
            "Aurora viewline at {:.1}°N (your location: {:.1}°N)\nKp: {:.1}",
            alert.viewline_lat, alert.user_lat, alert.kp
        );

        notify_rust::Notification::new()
            .summary(&title)
            .body(&body)
            .appname("Sunrise Winds")
            .show()?;

        info!("Desktop notification sent");
        Ok(())
    }
}

// --- Email ---

pub struct EmailNotifier {
    config: EmailConfig,
}

impl EmailNotifier {
    pub fn new(config: EmailConfig) -> Self {
        Self { config }
    }
}

impl Notifier for EmailNotifier {
    fn name(&self) -> &str {
        "email"
    }

    fn send(&self, alert: &Alert) -> Result<()> {
        let subject = format!("Aurora Alert — {}", alert.alert_type);
        let body = format!(
            "Aurora viewline has reached {:.1}°N at your longitude.\n\
             Your location: {:.1}°N\n\
             Current Kp: {:.1}\n\
             Time: {}",
            alert.viewline_lat, alert.user_lat, alert.kp, alert.timestamp
        );

        let email = Message::builder()
            .from(
                self.config
                    .smtp_user
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Invalid from address: {}", e))?,
            )
            .to(self
                .config
                .to_address
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid to address: {}", e))?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body)?;

        let creds = Credentials::new(self.config.smtp_user.clone(), self.config.smtp_pass.clone());

        let mailer = SmtpTransport::relay(&self.config.smtp_host)?
            .port(self.config.smtp_port)
            .credentials(creds)
            .build();

        mailer.send(&email)?;
        info!("Email notification sent");
        Ok(())
    }
}

// --- Notification Manager ---

pub struct NotificationManager {
    notifiers: Vec<Box<dyn Notifier>>,
    cooldown_minutes: u64,
    last_notification: Mutex<Option<DateTime<Utc>>>,
}

impl NotificationManager {
    pub fn new(config: &NotificationsConfig, email_config: &EmailConfig) -> Self {
        let mut notifiers: Vec<Box<dyn Notifier>> = Vec::new();

        if !config.webhook_url.is_empty() {
            notifiers.push(Box::new(WebhookNotifier::new(config.webhook_url.clone())));
        }

        if config.desktop_enabled {
            notifiers.push(Box::new(DesktopNotifier));
        }

        if config.email_enabled && !email_config.smtp_host.is_empty() {
            notifiers.push(Box::new(EmailNotifier::new(email_config.clone())));
        }

        info!(
            notifier_count = notifiers.len(),
            "Notification manager initialized"
        );

        Self {
            notifiers,
            cooldown_minutes: config.cooldown_minutes,
            last_notification: Mutex::new(None),
        }
    }

    /// Send alert through all configured notifiers, respecting cooldown.
    /// Returns the list of notifier names that were used, or None if in cooldown.
    pub fn notify(&self, alert: &mut Alert) -> Option<Vec<String>> {
        let now = Utc::now();
        let mut last = self.last_notification.lock().unwrap();

        if let Some(last_time) = *last {
            let elapsed = now.signed_duration_since(last_time);
            if elapsed.num_minutes() < self.cooldown_minutes as i64 {
                info!(
                    cooldown_remaining_mins = self.cooldown_minutes as i64 - elapsed.num_minutes(),
                    "Notification suppressed (cooldown)"
                );
                return None;
            }
        }

        let mut used: Vec<String> = Vec::new();

        for notifier in &self.notifiers {
            match notifier.send(alert) {
                Ok(()) => used.push(notifier.name().to_string()),
                Err(e) => warn!(notifier = notifier.name(), "Notification failed: {}", e),
            }
        }

        if !used.is_empty() {
            *last = Some(now);
            alert.notified_via = used.clone();
        }

        Some(used)
    }
}
