use crate::app::Message;
use crate::fl;
use cosmic::iced::alignment::Vertical;
use cosmic::{widget, Element};
use strum::{Display, EnumIter, IntoEnumIterator};

pub(crate) struct Settings {
    end_of_focus_sound: usize,
    end_of_relax_sound: usize,
    sound_names: Vec<String>,
}


impl Settings {
    pub fn new() -> Self {
        Self {
            end_of_focus_sound: 0,
            end_of_relax_sound: 0,
            sound_names: SoundName::iter().map(|x| x.to_string()).collect(),
        }
    }
    pub fn get_end_of_focus_sound(&self) -> &str { &self.sound_names[self.end_of_focus_sound] }
    pub fn get_end_of_relax_sound(&self) -> &str { &self.sound_names[self.end_of_relax_sound] }

    pub fn get_settings_view(&self) -> Element<Message> {
        let title = widget::text::title3(fl!("settings"));

        let mut root = widget::column().push(title);
        let mut settings = Vec::new();
        //EndOfFocusSound
        let selection = Some(self.end_of_focus_sound);
        let dropdown = widget::dropdown(&self.sound_names, selection, |x| Message::ChangeSetting(SettingMessage::EndOfFocusSoundChanged(x)));
        settings.push((fl!("settings","end-of-focus-sound"), dropdown));

        //EndOfRelaxSound
        let selection = Some(self.end_of_relax_sound);
        let dropdown = widget::dropdown(&self.sound_names, selection, |x| Message::ChangeSetting(SettingMessage::EndOfRelaxSoundChanged(x)));
        settings.push((fl!("settings","end-of-relax-sound"), dropdown));

        for (setting_name, dropdown) in settings {
            root = root.push(widget::row::with_capacity(2)
                .push(widget::text::text(setting_name).vertical_alignment(Vertical::Center))
                .push(dropdown)
                .spacing(10)
                );
        }
        root.into()
    }

    pub fn update(&mut self, message: SettingMessage) {
        match message {
            SettingMessage::EndOfFocusSoundChanged(index) => {
                self.end_of_focus_sound = index;
            }
            SettingMessage::EndOfRelaxSoundChanged(index) => {
                self.end_of_relax_sound = index;
            }
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) enum SettingMessage {
    EndOfFocusSoundChanged(usize),
    EndOfRelaxSoundChanged(usize),
}

#[derive(Display, Debug, EnumIter)]
enum SoundName {
    MessageNewInstant,
    MessageNewEmail,
    CompleteMediaBurn,
    CompleteMediaBurnTest,
    CompleteMediaRip,
    CompleteMediaFormat,
    CompleteDownload,
    CompleteCopy,
    CompleteScan,
    PhoneIncomingCall,
    PhoneOutgoingBusy,
    PhoneHangup,
    PhoneFailure,
    NetworkConnectivityEstablished,
    SystemBootup,
    SystemReady,
    SystemShutdown,
    SearchResults,
    SearchResultsEmpty,
    DesktopLogin,
    DesktopLogout,
    DesktopScreenLock,
    ServiceLogin,
    ServiceLogout,
    BatteryCaution,
    BatteryFull,
    DialogWarning,
    DialogInformation,
    DialogQuestion,
    SoftwareUpdateAvailable,
    DeviceAdded,
    DeviceAddedAudio,
    DeviceAddedMedia,
    DeviceRemoved,
    DeviceRemovedMedia,
    DeviceRemovedAudio,
    WindowNew,
    PowerPlug,
    PowerUnplug,
    SuspendStart,
    SuspendResume,
    LidOpen,
    LidClose,
    AlarmClockElapsed,
    WindowAttentionActive,
    WindowAttentionInactive,
}