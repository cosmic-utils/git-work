app-title = GitHub Notifications
about = About
notifications = Notifications
loading = Loading notifications...
error = Error
no-notifications = 🎉 All caught up!
no-notifications-desc = No new notifications
show-all = Show all
mark-all-read = Mark all read
mark-as-read = Mark as read
refresh = Refresh
notifications-count = { $count ->
    [one] { $count } notification
    *[other] { $count } notifications
}

# Error messages
error-no-token = GitHub token not found. Please set GITHUB_TOKEN environment variable.
error-token-setup = To fix this:
error-token-step1 = 1. Create a Personal Access Token on GitHub
error-token-step2 = 2. Set GITHUB_TOKEN environment variable
error-token-step3 = 3. Restart the applet
error-network = Network error occurred
error-api = GitHub API error
error-mark-read = Failed to mark as read

# Notification reasons
reason-assign = You were assigned
reason-author = You authored this
reason-comment = You commented
reason-invitation = You were invited
reason-manual = You subscribed
reason-mention = You were mentioned
reason-review-requested = Review requested
reason-security-alert = Security alert
reason-state-change = State changed
reason-subscribed = You're subscribed
reason-team-mention = Team mentioned

# Time formatting
time-just-now = Just now
time-minutes-ago = { $minutes ->
    [one] { $minutes } minute ago
    *[other] { $minutes } minutes ago
}
time-hours-ago = { $hours ->
    [one] { $hours } hour ago
    *[other] { $hours } hours ago
}
time-days-ago = { $days ->
    [one] { $days } day ago
    *[other] { $days } days ago
}

# Notification types
type-pull-request = Pull Request
type-issue = Issue
type-release = Release
type-discussion = Discussion
type-security-alert = Security Alert
type-repository-invitation = Repository Invitation
