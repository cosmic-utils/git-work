// SPDX-License-Identifier: GPL-3.0-only

use crate::config::Config;
use crate::github::*;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::{window::Id, Alignment, Length, Limits, Subscription, Task};
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::prelude::*;
use cosmic::theme::spacing;
use cosmic::widget;
use octocrab::models::activity::Notification;
use octocrab::models::NotificationId;
use octocrab::Octocrab;

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
    client: Option<Octocrab>,
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
    NotificationMarkedAsRead(Result<Option<NotificationId>, String>),
    ToggleShowAll(bool),
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
        let client = || -> Result<Octocrab, Box<dyn std::error::Error>> {
            let token = std::env::var("GITHUB_TOKEN")
                .map_err(|_|
                    "GITHUB_TOKEN environment variable not found. Please set your GitHub personal access token."
                )?;

            let client = octocrab::OctocrabBuilder::new()
                .personal_token(token)
                .build()?;
            Ok(client)
        };

        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            client: client().ok(),
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => config,
                })
                .unwrap_or_default(),
            ..Default::default()
        };

        let task = if app.client.is_some() {
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
            .push(widget::text("GitHub Notifications").size(spacing().space_s))
            .push(widget::horizontal_space().width(Length::Fill))
            .push(
                widget::button::icon(widget::icon::from_name("checkbox-checked-symbolic"))
                    .tooltip("Mark all read")
                    .padding([spacing().space_xxxs, spacing().space_xxs])
                    .on_press_maybe(
                        (!self.notifications.is_empty()).then_some(Message::MarkAllAsRead),
                    ),
            )
            .push(
                widget::button::icon(cosmic::widget::icon::from_name("view-refresh-symbolic"))
                    .padding(spacing().space_xxs)
                    .on_press_maybe((!self.is_loading).then_some(Message::RefreshNotifications)),
            )
            .spacing(spacing().space_xxs)
            .align_y(Alignment::Center);

        let content = if let Some(error) = &self.error_message {
            widget::column()
                .push(header)
                .push(
                    widget::container(
                        widget::column()
                            .push(widget::text("Error").size(spacing().space_s))
                            .push(widget::text(error).size(spacing().space_xs))
                            .push(if error.contains("GITHUB_TOKEN") {
                                widget::column()
                                    .push(widget::text("To fix this:").size(spacing().space_xs))
                                    .push(
                                        widget::text("1. Create a Personal Access Token on GitHub")
                                            .size(spacing().space_xs),
                                    )
                                    .push(
                                        widget::text("2. Set GITHUB_TOKEN environment variable")
                                            .size(spacing().space_xs),
                                    )
                                    .push(
                                        widget::text("3. Restart the applet")
                                            .size(spacing().space_xs),
                                    )
                                    .spacing(spacing().space_xxxs)
                            } else {
                                widget::column()
                            })
                            .width(Length::Fill)
                            .spacing(spacing().space_xxs),
                    )
                    .padding(spacing().space_s)
                    .class(cosmic::theme::Container::Card),
                )
                .spacing(spacing().space_xs)
        } else if self.is_loading {
            widget::column()
                .push(header)
                .push(
                    widget::container(
                        widget::row()
                            .push(widget::text("Loading notifications...").size(spacing().space_xs))
                            .push(widget::horizontal_space().width(Length::Fill))
                            .align_y(Alignment::Center),
                    )
                    .padding(spacing().space_s)
                    .class(cosmic::theme::Container::Card),
                )
                .spacing(spacing().space_xs)
        } else if self.notifications.is_empty() {
            widget::column()
                .push(header)
                .push(
                    widget::container(
                        widget::column()
                            .push(widget::text("🎉 All caught up!").size(spacing().space_s))
                            .push(widget::text("No new notifications").size(spacing().space_xs))
                            .spacing(spacing().space_xxs)
                            .align_x(Alignment::Center)
                            .width(Length::Fill),
                    )
                    .padding(spacing().space_l)
                    .class(cosmic::theme::Container::Card),
                )
                .spacing(spacing().space_xs)
        } else {
            let controls = widget::row()
                .push(
                    widget::text(format!("{} notifications", self.notifications.len()))
                        .size(spacing().space_xs),
                )
                .push(widget::horizontal_space().width(Length::Fill))
                .push(
                    widget::toggler(self.show_all)
                        .label("Show all")
                        .spacing(spacing().space_xxs)
                        .on_toggle(Message::ToggleShowAll),
                )
                .align_y(Alignment::Center)
                .spacing(spacing().space_xxs)
                .apply(widget::container)
                .class(cosmic::style::Container::Card)
                .padding(spacing().space_xxs);

            let mut notifications_list = widget::column().spacing(spacing().space_xxxs);
            for notification in &self.notifications {
                notifications_list = notifications_list.push(self.notification_item(notification));
            }
            let notifications = widget::scrollable(
                widget::container(notifications_list)
                    .padding([spacing().space_none, spacing().space_xxxs]),
            )
            .height(Length::Fixed(400.0));

            widget::column()
                .push(header)
                .push(notifications)
                .push(controls)
                .spacing(spacing().space_xxs)
        };

        self.core
            .applet
            .popup_container(
                widget::container(content)
                    .padding(spacing().space_xs)
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
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
        ])
    }

    /// Handles messages emitted by the application and its widgets.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        let mut tasks = vec![];

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
                        .max_width(1000.0)
                        .min_width(1000.0)
                        .min_height(1000.0)
                        .max_height(1000.0);
                    get_popup(popup_settings)
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::RefreshNotifications => {
                if let Some(client) = &self.client {
                    self.is_loading = true;
                    self.error_message = None;
                    let all = self.show_all;
                    let client = client.clone();
                    return Task::perform(
                        async move {
                            client
                                .activity()
                                .notifications()
                                .list()
                                .all(all)
                                .send()
                                .await
                                .map(|r| r.items)
                                .map_err(|e| e.to_string())
                        },
                        |result| cosmic::Action::App(Message::NotificationsLoaded(result)),
                    );
                }
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
                if let Some(client) = &self.client {
                    if let Some(url) = get_notification_url(&notification) {
                        let _ = open::that_detached(url);
                    }

                    // Mark as read if it was unread
                    if notification.unread {
                        let notification_id = notification.id.clone();
                        let client = client.clone();
                        return Task::perform(
                            async move {
                                client
                                    .activity()
                                    .notifications()
                                    .mark_as_read(notification_id.into())
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok(Some(notification_id))
                            },
                            |result| cosmic::Action::App(Message::NotificationMarkedAsRead(result)),
                        );
                    }
                }
            }
            Message::MarkAsRead(notification_id) => {
                if let Some(client) = &self.client {
                    let client = client.clone();
                    return Task::perform(
                        async move {
                            client
                                .activity()
                                .notifications()
                                .mark_as_read(notification_id.into())
                                .await
                                .map_err(|e| e.to_string())?;
                            Ok(Some(notification_id))
                        },
                        |result| cosmic::Action::App(Message::NotificationMarkedAsRead(result)),
                    );
                }
            }
            Message::MarkAllAsRead => {
                if let Some(client) = &self.client {
                    let client = client.clone();
                    return Task::perform(
                        async move {
                            client
                                .activity()
                                .notifications()
                                .mark_all_as_read(None)
                                .await
                                .map_err(|e| e.to_string())?;
                            Ok(None)
                        },
                        |result| cosmic::Action::App(Message::NotificationMarkedAsRead(result)),
                    );
                }
            }
            Message::NotificationMarkedAsRead(result) => match result {
                Ok(None) => {
                    tasks.push(cosmic::task::message(Message::RefreshNotifications));
                }
                Ok(Some(notification_id)) => {
                    self.notifications
                        .iter_mut()
                        .find(|n| n.id == notification_id)
                        .map(|n| n.unread = !n.unread);
                }
                Err(error) => {
                    self.error_message = Some(format!("Failed to mark as read: {}", error));
                }
            },
            Message::ToggleShowAll(show_all) => {
                self.show_all = show_all;
                tasks.push(cosmic::task::message(Message::RefreshNotifications));
            }
        }
        Task::batch(tasks)
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

impl AppModel {
    fn notification_item<'a>(&self, notification: &'a Notification) -> Element<'a, Message> {
        let reason = format_reason(&notification.reason);

        let header = widget::row()
            .push_maybe(
                notification.unread.then_some(
                    widget::button::icon(cosmic::widget::icon::from_name(
                        "mail-mark-read-symbolic",
                    ))
                    .padding(spacing().space_xxxs)
                    .on_press(Message::MarkAsRead(notification.id.clone()))
                    .class(cosmic::theme::Button::Text),
                ),
            )
            .push(
                widget::column()
                    .push(
                        widget::button::link(notification.subject.title.clone())
                            .padding(spacing().space_none)
                            .on_press(Message::OpenNotification(notification.clone())),
                    )
                    .push_maybe(
                        notification
                            .repository
                            .full_name
                            .as_ref()
                            .map(|name| widget::text(name).size(spacing().space_xs)),
                    )
                    .spacing(spacing().space_xxxs),
            )
            .spacing(spacing().space_xxs)
            .align_y(Alignment::Center);

        let time_ago = format_time_ago(&notification.updated_at);

        let footer = widget::row()
            .push(widget::text(time_ago).size(11))
            .push(widget::horizontal_space().width(Length::Fill))
            .push(widget::text(reason).size(11));

        widget::container(
            widget::column()
                .push(header)
                .push(footer)
                .spacing(spacing().space_xxxs),
        )
        .class(cosmic::style::Container::Card)
        .padding(spacing().space_xxs)
        .width(Length::Fill)
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
