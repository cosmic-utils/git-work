// SPDX-License-Identifier: GPL-3.0-only

use crate::config::Config;
use crate::github::GitHubService;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::futures::SinkExt;
use cosmic::iced::{window::Id, Alignment, Length, Limits, Subscription, Task};
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::prelude::*;
use cosmic::widget;
use octocrab::models::activity::Notification;
use octocrab::models::NotificationId;
use std::time::Duration;

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
#[derive(Default)]
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// The popup id.
    popup: Option<Id>,
    /// Configuration data that persists between application runs.
    config: Config,
    /// GitHub service for API interactions
    github_service: Option<GitHubService>,
    /// Current notifications
    notifications: Vec<Notification>,
    /// Loading state
    is_loading: bool,
    /// Error state
    error_message: Option<String>,
    /// Filter state
    show_all: bool,
    /// Last refresh time
    last_refresh: Option<std::time::Instant>,
    /// Unread count for the icon
    unread_count: usize,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    RefreshNotifications,
    NotificationsLoaded(Result<Vec<Notification>, String>),
    OpenNotification(Notification),
    MarkAsRead(NotificationId),
    MarkAllAsRead,
    NotificationMarkedAsRead(Result<(), String>),
    ToggleShowAll(bool),
    AutoRefresh,
    UpdateConfig(Config),
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "dev.edfloreshz.GitWork";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let github_service = match GitHubService::new() {
            Ok(service) => Some(service),
            Err(_) => None,
        };

        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            github_service,
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => config,
                })
                .unwrap_or_default(),
            ..Default::default()
        };

        let task = if app.github_service.is_some() {
            app.is_loading = true;
            Task::perform(async {}, |_| {
                cosmic::Action::App(Message::RefreshNotifications)
            })
        } else {
            app.error_message = Some(
                "GitHub token not found. Please set GITHUB_TOKEN environment variable.".to_string(),
            );
            Task::none()
        };

        (app, task)
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<'_, Self::Message> {
        let icon = if self.unread_count > 0 {
            "mail-unread-symbolic"
        } else {
            "mail-read-symbolic"
        };

        self.core
            .applet
            .icon_button(icon)
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let header = widget::row()
            .push(widget::text("GitHub Notifications").size(18))
            .push(widget::horizontal_space().width(Length::Fill))
            .push(if self.is_loading {
                widget::button::icon(cosmic::widget::icon::from_name("view-refresh-symbolic"))
                    .padding(8)
            } else {
                widget::button::icon(cosmic::widget::icon::from_name("view-refresh-symbolic"))
                    .padding(8)
                    .on_press(Message::RefreshNotifications)
            })
            .spacing(8)
            .align_y(Alignment::Center);

        let content = if let Some(error) = &self.error_message {
            widget::column()
                .push(header)
                .push(
                    widget::container(
                        widget::column()
                            .push(widget::text("Error").size(16))
                            .push(widget::text(error).size(14))
                            .push(if error.contains("GITHUB_TOKEN") {
                                widget::column()
                                    .push(widget::text("To fix this:").size(14))
                                    .push(
                                        widget::text("1. Create a Personal Access Token on GitHub")
                                            .size(12),
                                    )
                                    .push(
                                        widget::text("2. Set GITHUB_TOKEN environment variable")
                                            .size(12),
                                    )
                                    .push(widget::text("3. Restart the applet").size(12))
                                    .spacing(4)
                            } else {
                                widget::column()
                            })
                            .spacing(8),
                    )
                    .padding(16)
                    .class(cosmic::theme::Container::Card),
                )
                .spacing(12)
        } else if self.is_loading {
            widget::column()
                .push(header)
                .push(
                    widget::container(
                        widget::row()
                            .push(widget::text("Loading notifications...").size(14))
                            .push(widget::horizontal_space().width(Length::Fill))
                            .align_y(Alignment::Center),
                    )
                    .padding(16)
                    .class(cosmic::theme::Container::Card),
                )
                .spacing(12)
        } else if self.notifications.is_empty() {
            widget::column()
                .push(header)
                .push(
                    widget::container(
                        widget::column()
                            .push(widget::text("🎉 All caught up!").size(16))
                            .push(widget::text("No new notifications").size(14))
                            .spacing(8)
                            .align_x(Alignment::Center),
                    )
                    .padding(32)
                    .class(cosmic::theme::Container::Card),
                )
                .spacing(12)
        } else {
            let controls = widget::row()
                .push(widget::text(format!("{} notifications", self.notifications.len())).size(12))
                .push(widget::horizontal_space().width(Length::Fill))
                .push(
                    widget::toggler(self.show_all)
                        .label("Show all")
                        .on_toggle(Message::ToggleShowAll),
                )
                .push(
                    widget::button::text("Mark all read")
                        .padding([4, 8])
                        .on_press(Message::MarkAllAsRead),
                )
                .spacing(8)
                .align_y(Alignment::Center);

            let mut notifications_list = widget::column();
            for notification in &self.notifications {
                if self.show_all || notification.unread {
                    notifications_list =
                        notifications_list.push(self.notification_item(notification));
                }
            }

            widget::column()
                .push(header)
                .push(controls)
                .push(
                    widget::scrollable(widget::container(notifications_list).padding([0, 4]))
                        .height(Length::Fixed(400.0)),
                )
                .spacing(8)
        };

        self.core
            .applet
            .popup_container(
                widget::container(content)
                    .padding(12)
                    .class(cosmic::theme::Container::Dialog),
            )
            .into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They are started at the
    /// beginning of the application, and persist through its lifetime.
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            // Auto-refresh every 30 seconds
            Subscription::run_with_id(
                std::any::TypeId::of::<()>(),
                cosmic::iced::stream::channel(1, |mut output| async move {
                    loop {
                        tokio::time::sleep(Duration::from_secs(30)).await;
                        let _ = output.send(Message::AutoRefresh).await;
                    }
                }),
            ),
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
        ])
    }

    /// Handles messages emitted by the application and its widgets.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::UpdateConfig(config) => {
                self.config = config;
            }
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(450.0)
                        .min_width(400.0)
                        .min_height(300.0)
                        .max_height(600.0);
                    get_popup(popup_settings)
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::RefreshNotifications => {
                if let Some(service) = &self.github_service {
                    self.is_loading = true;
                    self.error_message = None;
                    let service = service.clone();
                    return Task::perform(
                        async move {
                            service
                                .get_notifications(false, false)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        |result| cosmic::Action::App(Message::NotificationsLoaded(result)),
                    );
                }
            }
            Message::AutoRefresh => {
                // Only auto-refresh if we haven't refreshed in the last 25 seconds
                if let Some(last_refresh) = self.last_refresh {
                    if last_refresh.elapsed() < Duration::from_secs(25) {
                        return Task::none();
                    }
                }
                return Task::perform(async {}, |_| {
                    cosmic::Action::App(Message::RefreshNotifications)
                });
            }
            Message::NotificationsLoaded(result) => {
                self.is_loading = false;
                self.last_refresh = Some(std::time::Instant::now());
                match result {
                    Ok(notifications) => {
                        self.unread_count = notifications.iter().filter(|n| n.unread).count();
                        self.notifications = notifications;
                        self.error_message = None;
                    }
                    Err(error) => {
                        self.error_message = Some(error);
                    }
                }
            }
            Message::OpenNotification(notification) => {
                if let Some(service) = &self.github_service {
                    if let Some(url) = service.get_notification_url(&notification) {
                        let _ = open::that_detached(url);
                    }

                    // Mark as read if it was unread
                    if notification.unread {
                        let notification_id = notification.id.clone();
                        let service = service.clone();
                        return Task::perform(
                            async move {
                                service
                                    .mark_as_read(notification_id)
                                    .await
                                    .map_err(|e| e.to_string())
                            },
                            |result| cosmic::Action::App(Message::NotificationMarkedAsRead(result)),
                        );
                    }
                }
            }
            Message::MarkAsRead(notification_id) => {
                if let Some(service) = &self.github_service {
                    let service = service.clone();
                    return Task::perform(
                        async move {
                            service
                                .mark_as_read(notification_id)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        |result| cosmic::Action::App(Message::NotificationMarkedAsRead(result)),
                    );
                }
            }
            Message::MarkAllAsRead => {
                if let Some(service) = &self.github_service {
                    let service = service.clone();
                    return Task::perform(
                        async move { service.mark_all_as_read().await.map_err(|e| e.to_string()) },
                        |result| cosmic::Action::App(Message::NotificationMarkedAsRead(result)),
                    );
                }
            }
            Message::NotificationMarkedAsRead(result) => {
                match result {
                    Ok(()) => {
                        // Refresh notifications after marking as read
                        return Task::perform(async {}, |_| {
                            cosmic::Action::App(Message::RefreshNotifications)
                        });
                    }
                    Err(error) => {
                        self.error_message = Some(format!("Failed to mark as read: {}", error));
                    }
                }
            }
            Message::ToggleShowAll(show_all) => {
                self.show_all = show_all;
            }
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

impl AppModel {
    fn notification_item<'a>(&self, notification: &'a Notification) -> Element<'a, Message> {
        let service = self.github_service.as_ref().unwrap();
        let icon = service.get_notification_icon(notification);
        let reason = service.format_reason(&notification.reason);

        let unread_indicator = if notification.unread {
            widget::container(widget::text("●").size(8))
        } else {
            widget::container(widget::text(""))
        };

        let header = widget::row()
            .push(unread_indicator)
            .push(
                widget::button::icon(cosmic::widget::icon::from_name(icon))
                    .padding(4)
                    .class(cosmic::theme::Button::Text),
            )
            .push(
                widget::column()
                    .push(widget::text(&notification.subject.title).size(14))
                    .push(
                        widget::text(
                            notification
                                .repository
                                .full_name
                                .as_ref()
                                .map(|name| format!("{} • {}", name, reason))
                                .unwrap_or(reason),
                        )
                        .size(12),
                    )
                    .spacing(2),
            )
            .push(widget::horizontal_space().width(Length::Fill))
            .push(if notification.unread {
                widget::button::icon(cosmic::widget::icon::from_name("mail-mark-read-symbolic"))
                    .padding(4)
                    .on_press(Message::MarkAsRead(notification.id.clone()))
                    .class(cosmic::theme::Button::Text)
            } else {
                widget::button::icon(cosmic::widget::icon::from_name("mail-read-symbolic"))
                    .padding(4)
                    .class(cosmic::theme::Button::Text)
            })
            .spacing(8)
            .align_y(Alignment::Center);

        let time_ago = format_time_ago(&notification.updated_at);
        let subject_type = notification.subject.r#type.clone();

        let footer = widget::row()
            .push(widget::text(time_ago).size(11))
            .push(widget::horizontal_space().width(Length::Fill))
            .push(widget::text(subject_type).size(11));

        widget::button::custom(
            widget::container(widget::column().push(header).push(footer).spacing(4))
                .padding(12)
                .width(Length::Fill),
        )
        .class(cosmic::theme::Button::Text)
        .on_press(Message::OpenNotification(notification.clone()))
        .into()
    }
}

fn format_time_ago(datetime: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(*datetime);

    if duration.num_days() > 0 {
        format!("{} days ago", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{} minutes ago", duration.num_minutes())
    } else {
        "Just now".to_string()
    }
}
