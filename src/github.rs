// SPDX-License-Identifier: GPL-3.0-only

use octocrab::models::activity::Notification;

pub fn get_notification_url(notification: &Notification) -> Option<String> {
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

pub fn format_reason(reason: &str) -> String {
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
        "ci_activity" => "Workflow activity".to_string(),
        _ => reason.to_string(),
    }
}
