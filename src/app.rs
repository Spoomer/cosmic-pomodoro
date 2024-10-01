// SPDX-License-Identifier: GPL-3.0-only

use crate::core::duration_extension::TimeDurationExt;
use crate::core::icon_cache;
use crate::core::pomodoro_timer::{PomodoroPhase, PomodoroState, PomodoroTimer};
use crate::fl;
use crate::views::settings::SettingMessage;
use cosmic::app::{Command, Core};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::time;
use cosmic::iced::{Alignment, ContentFit, Length, Subscription};
use cosmic::widget::{self, menu};
use cosmic::{cosmic_theme, iced_widget, theme, Application, ApplicationExt, Apply, Element};
use notify_rust::Notification;
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::Writer;
use std::collections::HashMap;
use std::io::Cursor;
use std::str;
use std::sync::atomic::Ordering;
use std::time::Duration;

const REPOSITORY: &str = "https://github.com/Spoomer/cosmic-pomodoro";
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// This is the struct that represents your application.
/// It is used to define the data that will be used by your application.
pub struct CosmicPomodoro {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    pomodoro_timer: PomodoroTimer,
}


/// This is the enum that contains all the possible variants that your application will need to transmit messages.
/// This is used to communicate between the different parts of your application.
/// If your application does not need to send messages, you can use an empty enum or `()`.
#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    StartTimer,
    Refresh,
    ChangeSetting(SettingMessage),
}

/// Identifies a context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    Settings,
}

impl ContextPage {
    fn title(&self) -> String {
        match self {
            Self::About => fl!("about"),
            Self::Settings => fl!("settings"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    Settings,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::Settings => { Message::ToggleContextPage(ContextPage::Settings) }
        }
    }
}

/// Implement the `Application` trait for your application.
/// This is where you define the behavior of your application.
///
/// The `Application` trait requires you to define the following types and constants:
/// - `Executor` is the async executor that will be used to run your application's commands.
/// - `Flags` is the data that your application needs to use before it starts.
/// - `Message` is the enum that contains all the possible variants that your application will need to transmit messages.
/// - `APP_ID` is the unique identifier of your application.
impl Application for CosmicPomodoro {
    type Executor = cosmic::executor::Default;

    type Flags = ();

    type Message = Message;

    const APP_ID: &'static str = "com.example.CosmicPomodoro";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// This is the entry point of your application, it is where you initialize your application.
    ///
    /// Any work that needs to be done before the application starts should be done here.
    ///
    /// - `core` is used to passed on for you by libcosmic to use in the core of your own application.
    /// - `flags` is used to pass in any data that your application needs to use before it starts.
    /// - `Command` type is used to send messages to your application. `Command::none()` can be used to send no messages to your application.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut app = CosmicPomodoro {
            core,
            context_page: ContextPage::default(),
            key_binds: HashMap::new(),
            pomodoro_timer: PomodoroTimer::new(),
        };

        let command = app.update_titles();

        (app, command)
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<Element<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => self.about(),
            ContextPage::Settings => self.pomodoro_timer.settings.get_settings_view()
        })
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("menu")),
            menu::items(
                &self.key_binds,
                vec![
                    menu::Item::Button(fl!("about"), MenuAction::About),
                    menu::Item::Button(fl!("settings"), MenuAction::Settings)
                ],
            ),
        )]);

        vec![menu_bar.into()]
    }

    /// Application messages are handled here. The application state can be modified based on
    /// what message was received. Commands may be returned for asynchronous execution on a
    /// background thread managed by the application's executor.
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::LaunchUrl(url) => {
                let _result = open::that_detached(url);
            }

            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    // Close the context drawer if the toggled context page is the same.
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }

                // Set the title of the context drawer.
                self.set_context_title(context_page.title());
            }
            Message::StartTimer => {
                match self.pomodoro_timer.pomodoro_state {
                    PomodoroState::Stop => {
                        self.pomodoro_timer.pomodoro_phase = match self.pomodoro_timer.pomodoro_phase {
                            PomodoroPhase::BeforeFocus => PomodoroPhase::Focus,
                            PomodoroPhase::Focus => PomodoroPhase::BeforeRelax,
                            PomodoroPhase::BeforeRelax => PomodoroPhase::Relax,
                            PomodoroPhase::Relax => PomodoroPhase::BeforeFocus,
                        };
                        self.pomodoro_timer.start()
                    }
                    PomodoroState::Run => {
                        self.pomodoro_timer.pause()
                    }
                    PomodoroState::Pause => {
                        self.pomodoro_timer.resume()
                    }
                }
            }
            Message::Refresh => {
                if self.pomodoro_timer.remaining_sec.load(Ordering::SeqCst) == 0u32 {
                    match self.pomodoro_timer.pomodoro_phase {
                        PomodoroPhase::BeforeFocus => {}
                        PomodoroPhase::Focus => {
                            self.pomodoro_timer.pomodoro_phase = PomodoroPhase::BeforeRelax;
                            self.pomodoro_timer.stop();
                            self.pomodoro_timer.remaining_sec.store(self.pomodoro_timer.pomodoro_lengths[self.pomodoro_timer.position].relax, Ordering::SeqCst);
                            _ = Notification::new()
                                .summary(&fl!("before-relax"))
                                .sound_name("window-attention-inactive")
                                .show();
                            if self.is_focused() {
                                self.pomodoro_timer.pomodoro_phase = PomodoroPhase::Relax;
                                self.pomodoro_timer.start();
                            }
                        }
                        PomodoroPhase::BeforeRelax => {}
                        PomodoroPhase::Relax => {
                            self.pomodoro_timer.position += 1;
                            if self.pomodoro_timer.position >= self.pomodoro_timer.pomodoro_lengths.len() {
                                self.pomodoro_timer.position = 0;
                            }
                            self.pomodoro_timer.pomodoro_phase = PomodoroPhase::BeforeFocus;
                            self.pomodoro_timer.stop();
                            self.pomodoro_timer.remaining_sec.store(self.pomodoro_timer.pomodoro_lengths[self.pomodoro_timer.position].focus, Ordering::SeqCst);
                            _ = Notification::new()
                                .summary(&fl!("after-relax"))
                                .body(&fl!("before-focus"))
                                .sound_name("alarm-clock-elapsed")
                                .show();
                        }
                    }
                }
            }
            Message::ChangeSetting(setting_message) => {
                self.pomodoro_timer.settings.update(setting_message);
            }
        }
        Command::none()
    }
    fn subscription(&self) -> Subscription<Self::Message> {
        match self.pomodoro_timer.pomodoro_state {
            PomodoroState::Run => {
                time::every(Duration::from_millis(250))
                    .map(|_| Message::Refresh)
            }
            PomodoroState::Stop => { Subscription::none() }
            PomodoroState::Pause => { Subscription::none() }
        }
    }
    /// This is the main view of your application, it is the root of your widget tree.
    ///
    /// The `Element` type is used to represent the visual elements of your application,
    /// it has a `Message` associated with it, which dictates what type of message it can send.
    ///
    /// To get a better sense of which widgets are available, check out the `widget` module.
    fn view(&self) -> Element<Self::Message> {
        let mut initial_secs: u32 = 0;
        if self.pomodoro_timer.pomodoro_phase == PomodoroPhase::Relax {
            initial_secs = self.pomodoro_timer.pomodoro_lengths[self.pomodoro_timer.position].relax;
        } else if self.pomodoro_timer.pomodoro_phase == PomodoroPhase::Focus {
            initial_secs = self.pomodoro_timer.pomodoro_lengths[self.pomodoro_timer.position].focus;
        }
        let remaining_secs = self.pomodoro_timer.remaining_sec.load(Ordering::SeqCst);
        let cosmic_theme::Spacing { space_m, .. } = theme::active().cosmic().spacing;
        let mut root = widget::column::with_capacity(3).spacing(space_m);
        let play_pause_button: widget::button::Button<'static, Message>;
        match self.pomodoro_timer.pomodoro_state {
            PomodoroState::Pause | PomodoroState::Stop => {
                play_pause_button = CosmicPomodoro::get_play_pause_button("play", initial_secs, remaining_secs);
            }
            PomodoroState::Run => {
                play_pause_button = CosmicPomodoro::get_play_pause_button("pause", initial_secs, remaining_secs);
            }
        }
        match self.pomodoro_timer.pomodoro_phase {
            PomodoroPhase::BeforeFocus => {
                root = root.push(widget::text::heading(fl!("before-focus"))
                    .size(26)
                    .width(Length::Fill)
                    .horizontal_alignment(Horizontal::Center))
            }
            PomodoroPhase::Focus => {
                root = root.push(widget::text::heading(fl!("focus-running"))
                    .size(26)
                    .width(Length::Fill)
                    .horizontal_alignment(Horizontal::Center))
            }
            PomodoroPhase::BeforeRelax => {
                root = root.push(widget::text::heading(fl!("before-relax"))
                    .size(26)
                    .width(Length::Fill)
                    .horizontal_alignment(Horizontal::Center))
            }
            PomodoroPhase::Relax => {
                root = root.push(widget::text::heading(fl!("relax-running"))
                    .size(26)
                    .width(Length::Fill)
                    .horizontal_alignment(Horizontal::Center))
            }
        }
        root = root.push(widget::row::with_children(
            vec![widget::column().width(Length::Fill).into(),
                 play_pause_button.width(Length::FillPortion(2)).into(),
                 widget::column().width(Length::Fill).into()
            ]
        ));
        let remaining_duration = Duration::from_secs(remaining_secs as u64);

        let formated_remaining = format!("{:02}:{:02}", remaining_duration.as_minutes(), remaining_duration.as_seconds());
        root = root.push(widget::text::heading(formated_remaining)
            .size(26)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center)
        );


        root.apply(widget::container)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }
}

impl CosmicPomodoro {
    /// The about page for this app.
    pub fn about(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let icon = widget::svg(widget::svg::Handle::from_memory(
            &include_bytes!("../res/icons/hicolor/128x128/apps/com.example.CosmicPomodoro.svg")
                [..],
        ));

        let title = widget::text::title3(fl!("app-title"));
        let version = widget::text::title4(fl!("app-version") + ": " + VERSION);
        let link = widget::button::link(REPOSITORY)
            .on_press(Message::LaunchUrl(REPOSITORY.to_string()))
            .padding(0);

        widget::column()
            .push(icon)
            .push(title)
            .push(version)
            .push(link)
            .align_items(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }

    /// Updates the header and window titles.
    pub fn update_titles(&mut self) -> Command<Message> {
        let window_title = fl!("app-title");

        self.set_window_title(window_title)
    }

    fn get_play_pause_button(button_name : &'static str, initial_secs: u32, remaining_secs: u32) -> widget::button::Button<'static, Message> {
        let percentage = 1.0 -  remaining_secs as f32 / initial_secs as f32;
        let radian = 2.0 * std::f32::consts::PI * percentage;
        let icon_svg = icon_cache::get_icon_cache_svg(button_name);
        let content = str::from_utf8(icon_svg.as_ref()).unwrap();
        let mut reader = Reader::from_str(content);
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        loop {
            match reader.read_event() {
                Ok(Event::Empty(e)) if e.attributes().any(|attr|
                    {
                        if !attr.is_ok() {
                            return false;
                        }
                        let attr = attr.unwrap();
                        attr.key.local_name().as_ref() == b"id" && attr.value.as_ref() == b"progress-circle"
                    }) => {

                    let mut elem = BytesStart::new("path");

                    // collect existing attributes except d
                    elem.extend_attributes(e.attributes()
                        .map(|attr| attr.unwrap())
                        .filter(|attr| attr.key.local_name().as_ref() != b"d")
                    );

                    let data = e.try_get_attribute("d").unwrap().unwrap();
                    let data_string = str::from_utf8(data.value.as_ref()).unwrap();
                    let mut parts = data_string.split(' ').collect::<Vec<_>>();

                    let a_position = parts.iter().position(|&part| part.eq("A"));
                    if a_position.is_none() {
                        continue;
                    }

                    let a_position = a_position.unwrap();
                    let radius = parts[a_position + 1].parse::<f32>().unwrap();
                    let large_arc_postion = a_position + 4;
                    let x_position = a_position + 6;
                    let y_position = a_position + 7;
                    if percentage > 0.5 {
                        parts[large_arc_postion] = "1";
                    } else {
                        parts[large_arc_postion] = "0";
                    }
                    let x = (260.0 + radian.cos() * radius).to_string();
                    parts[x_position] = &x;
                    let y = (260.0 + radian.sin() * radius).to_string();
                    parts[y_position] = &y;
                    let path = parts.join(" ");
                    elem.push_attribute(("d", path.as_str()));
                    // writes the event to the writer
                    writer.write_event(Event::Empty(elem)).expect("xml writer error");
                }
                Ok(Event::Eof) => break,
                // we can either move or borrow the event to write, depending on your use-case
                Ok(e) => assert!(writer.write_event(e).is_ok()),
                Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
            }
        }
        let icon_svg = writer.into_inner().into_inner();
        widget::button(widget::svg(iced_widget::svg::Handle::from_memory(icon_svg)).content_fit(ContentFit::Contain))
            .width(Length::Fill)
            .style(cosmic::style::Button::IconVertical)
            .on_press(Message::StartTimer)
    }
    fn is_focused(&self) -> bool {
        match self.core.focused_window() {
            Some(_) => true,
            None => false,
        }
    }
}
