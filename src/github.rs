// SPDX-License-Identifier: GPL-3.0-only

use octocrab::{
    models::{activity::Notification, NotificationId},
    Octocrab,
};
use std::env;

#[derive(Clone)]
pub struct GitHubService {
    client: Octocrab,
}

impl GitHubService {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let token = env::var("GITHUB_TOKEN")
            .map_err(|_|
                "GITHUB_TOKEN environment variable not found. Please set your GitHub personal access token."
            )?;

        let client = octocrab::OctocrabBuilder::new()
            .personal_token(token)
            .build()?;

        Ok(Self { client })
    }

    pub async fn get_notifications(
        &self,
        _all: bool,
        _participating: bool,
    ) -> Result<Vec<Notification>, Box<dyn std::error::Error>> {
        let notifications = self.client.activity().notifications().list().send().await?;

        Ok(notifications.items)
    }

    pub async fn mark_as_read(
        &self,
        notification_id: impl Into<NotificationId>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.client
            .activity()
            .notifications()
            .mark_as_read(notification_id.into())
            .await?;

        Ok(())
    }

    pub async fn mark_all_as_read(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.client
            .activity()
            .notifications()
            .mark_all_as_read(None)
            .await?;

        Ok(())
    }

    pub fn get_notification_url(&self, notification: &Notification) -> Option<String> {
        if let Some(url) = &notification.subject.url {
            let url = url
                .to_string()
                .replace("api.github.com/repos", "github.com");

            // Convert API URL to web URL
            if url.contains("/pulls/") {
                Some(
                    url.replace("api.github.com/repos", "github.com")
                        .replace("/pulls/", "/pull/"),
                )
            } else if url.contains("/issues/") {
                Some(
                    url.replace("api.github.com/repos", "github.com")
                        .replace("/issues/", "/issues/"),
                )
            } else {
                notification.repository.html_url.clone().map(Into::into)
            }
        } else {
            notification.repository.html_url.clone().map(Into::into)
        }
    }

    pub fn get_notification_icon(&self, notification: &Notification) -> &'static str {
        match notification.subject.r#type.as_str() {
            "PullRequest" => "git-merge-symbolic",
            "Issue" => "dialog-question-symbolic",
            "Release" => "software-update-available-symbolic",
            "RepositoryInvitation" => "mail-send-symbolic",
            "SecurityAlert" => "dialog-warning-symbolic",
            "Discussion" => "user-available-symbolic",
            _ => "mail-unread-symbolic",
        }
    }

    pub fn format_reason(&self, reason: &str) -> String {
        match reason {
            "assign" => "You were assigned".to_string(),
            "author" => "You authored this".to_string(),
            "comment" => "You commented".to_string(),
            "invitation" => "You were invited".to_string(),
            "manual" => "You subscribed".to_string(),
            "mention" => "You were mentioned".to_string(),
            "review_requested" => "Review requested".to_string(),
            "security_alert" => "Security alert".to_string(),
            "state_change" => "State changed".to_string(),
            "subscribed" => "You're subscribed".to_string(),
            "team_mention" => "Team mentioned".to_string(),
            _ => reason.to_string(),
        }
    }
}
