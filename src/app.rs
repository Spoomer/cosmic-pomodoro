// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;
use std::sync::{mpsc, Arc};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use crate::{fl, icon_cache};
use cosmic::app::{Command, Core};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, ContentFit, Length, Subscription};
use cosmic::widget::{self, container, icon, menu, nav_bar, svg};
use cosmic::{cosmic_theme, theme, Application, ApplicationExt, Apply, Element};
use cosmic::iced::window::Id;
use cosmic::iced_core::widget::operation::Focusable;
use cosmic::theme::iced;
use cosmic::widget::menu::Item::Button;
use cosmic::iced::time;
use crate::duration_extension::TimeDurationExt;
use crate::icon_cache::IconCache;

const REPOSITORY: &str = "https://github.com/Spoomer/cosmic-pomodoro";

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

struct PomodoroTimer{
    pomodoro_lengths: Vec<PomodoroLength>,
    position: usize,
    pomodoro_state: PomodoroState,
    pomodoro_phase: PomodoroPhase,
    remaining_sec: Arc<AtomicU32>,
    counter_pipe: Sender<bool>
}
impl PomodoroTimer{
    fn new() -> Self {
        let (to_pomodoro_timer, from_countdown) = mpsc::channel::<bool>();
    
        let remaining_sec = Arc::new(AtomicU32::new(0));
        let remaining_sec_clone = remaining_sec.clone();
        
        thread::spawn(move || {
            let mut is_active = false;
            loop {
                is_active =match from_countdown.try_recv(){
                    Ok(state) => {state}
                    Err(_) => {
                        is_active
                    }
                };
                if is_active && remaining_sec_clone.as_ref().load(Ordering::SeqCst) > 0u32 {
                    remaining_sec_clone.fetch_sub(1, Ordering::SeqCst);
                }
                sleep(Duration::from_secs(1));
            }
        });

        Self{
            pomodoro_lengths : vec![
                PomodoroLength::new(25 * 60, 5  * 60),
                PomodoroLength::new(25  * 60, 5  * 60),
                PomodoroLength::new(25  * 60, 5  * 60),
                PomodoroLength::new(25  * 60, 5  * 60),
                PomodoroLength::new(25  * 60, 15  * 60),
            ],
            position: 0,
            pomodoro_state: PomodoroState::Stop,
            pomodoro_phase: PomodoroPhase::Focus,
            remaining_sec,
            counter_pipe: to_pomodoro_timer
        }
    }
    
    fn start(&mut self) {
        self.remaining_sec.store(self.pomodoro_lengths[self.position].focus, Ordering::SeqCst);
        self.counter_pipe.send(true).unwrap();
        self.pomodoro_state = PomodoroState::Run;
    }
    
    fn pause(&mut self){
        self.counter_pipe.send(false).unwrap();
        self.pomodoro_state = PomodoroState::Pause;
    }
    
    fn resume(&mut self){
        self.counter_pipe.send(true).unwrap();
        self.pomodoro_state = PomodoroState::Run;

    }
    
    fn stop (&mut self){
        self.counter_pipe.send(false).unwrap();
        self.remaining_sec.store(0, Ordering::SeqCst);
        self.position = 0;
        self.pomodoro_state = PomodoroState::Stop;
    }
}
struct PomodoroLength {
    focus:u32,
    relax:u32,
}

impl PomodoroLength {
    fn new(focus : u32, relax: u32) -> Self{
        Self{
            focus,
            relax
        }
    }
}



/// This is the enum that contains all the possible variants that your application will need to transmit messages.
/// This is used to communicate between the different parts of your application.
/// If your application does not need to send messages, you can use an empty enum or `()`.
#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    StartTimer,
    Refresh
}

/// Identifies a context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}

impl ContextPage {
    fn title(&self) -> String {
        match self {
            Self::About => fl!("about"),
        }
    }
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum PomodoroState{
    Stop,
    Run,
    Pause,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum PomodoroPhase{
    Focus,
    Relax
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
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
        })
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("menu")),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), MenuAction::About)],
            ),
        )]);

        vec![menu_bar.into()]
    }

    /// Instructs the cosmic runtime to use this model as the nav bar model.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        None
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
                if self.pomodoro_timer.remaining_sec.load(Ordering::SeqCst) == 0u32{
                    match self.pomodoro_timer.pomodoro_phase{
                        PomodoroPhase::Focus => {
                            self.pomodoro_timer.pomodoro_phase = PomodoroPhase::Relax;
                            self.pomodoro_timer.remaining_sec.store(self.pomodoro_timer.pomodoro_lengths[self.pomodoro_timer.position].relax, Ordering::SeqCst);
                        }
                        PomodoroPhase::Relax => {
                            self.pomodoro_timer.position += 1;
                            if self.pomodoro_timer.position >= self.pomodoro_timer.pomodoro_lengths.len(){
                                self.pomodoro_timer.position = 0;
                            }
                            self.pomodoro_timer.pomodoro_phase = PomodoroPhase::Focus;
                            self.pomodoro_timer.remaining_sec.store(self.pomodoro_timer.pomodoro_lengths[self.pomodoro_timer.position].relax, Ordering::SeqCst);
                        }
                    }
                }
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
            PomodoroState::Stop => {Subscription::none()}
            PomodoroState::Pause => {Subscription::none()}
        }
    }
    /// This is the main view of your application, it is the root of your widget tree.
    ///
    /// The `Element` type is used to represent the visual elements of your application,
    /// it has a `Message` associated with it, which dictates what type of message it can send.
    ///
    /// To get a better sense of which widgets are available, check out the `widget` module.
    fn view(&self) -> Element<Self::Message> {
        
        let mut root: Vec<Element<Message>> = Vec::new();
        let play_pause_button : widget::button::Button<'static, Message>;
        match self.pomodoro_timer.pomodoro_state {
            PomodoroState::Pause | PomodoroState::Stop =>{
                play_pause_button = CosmicPomodoro::get_play_button();
            }
            PomodoroState::Run => {
                play_pause_button = CosmicPomodoro::get_pause_button();
            }
        }
        
        root.push(widget::row::with_children(
            vec![widget::column().width(Length::Fill).into(),
                 play_pause_button.width(Length::FillPortion(2)).into(),
                 widget::column().width(Length::Fill).into()
            ]
        ).into());
        
        let remaining =Duration::from_secs(self.pomodoro_timer.remaining_sec.load(Ordering::SeqCst) as u64);
        let formated_remaining =format!("{:02}:{:02}",remaining.as_minutes(), remaining.as_seconds());
        root.push(widget::text(formated_remaining).size(18).width(Length::Fill).horizontal_alignment(Horizontal::Center).into());

        
        widget::column::with_children(root)
            .apply(widget::container)
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

        let link = widget::button::link(REPOSITORY)
            .on_press(Message::LaunchUrl(REPOSITORY.to_string()))
            .padding(0);

        widget::column()
            .push(icon)
            .push(title)
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

    fn get_play_button() -> widget::button::Button<'static, Message> {
        let icon_handle = icon_cache::get_icon_cache_handle("play");
        widget::button(widget::svg(icon_handle).content_fit(ContentFit::Contain))
            .width(Length::Fill)
            .style(cosmic::style::Button::IconVertical)
            .on_press(Message::StartTimer)
    }

    fn get_pause_button() -> widget::button::Button<'static, Message> {
        let icon_handle = icon_cache::get_icon_cache_handle("pause");
        widget::button(widget::svg(icon_handle).content_fit(ContentFit::Contain))
            .width(Length::Fill)
            .style(cosmic::style::Button::IconVertical)
            .on_press(Message::StartTimer)
    }
}
