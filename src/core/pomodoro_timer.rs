use std::sync::{mpsc, Arc};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use crate::views::settings::Settings;

pub(crate) struct PomodoroTimer {
    pub settings: Settings,
    pub pomodoro_lengths: Vec<PomodoroLength>,
    pub position: usize,
    pub pomodoro_state: PomodoroState,
    pub pomodoro_phase: PomodoroPhase,
    pub remaining_sec: Arc<AtomicU32>,
    counter_pipe: Sender<bool>,
}

impl PomodoroTimer {
    pub fn new() -> Self {
        let (to_pomodoro_timer, from_countdown) = mpsc::channel::<bool>();
        //test
        let pomodoro_lengths = vec![
            PomodoroLength::new(10, 5),
            PomodoroLength::new(7, 4)
        ];
        // let pomodoro_lengths = vec![
        //     PomodoroLength::new(25 * 60, 5 * 60),
        //     PomodoroLength::new(25 * 60, 5 * 60),
        //     PomodoroLength::new(25 * 60, 5 * 60),
        //     PomodoroLength::new(25 * 60, 5 * 60),
        //     PomodoroLength::new(25 * 60, 15 * 60),
        // ];
        let remaining_sec = Arc::new(AtomicU32::new(pomodoro_lengths[0].focus));
        let remaining_sec_clone = remaining_sec.clone();

        thread::spawn(move || {
            let mut is_active = false;
            loop {
                is_active = match from_countdown.try_recv() {
                    Ok(state) => { state }
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

        Self {
            settings: Settings::new(),
            pomodoro_lengths,
            position: 0,
            pomodoro_state: PomodoroState::Stop,
            pomodoro_phase: PomodoroPhase::BeforeFocus,
            remaining_sec,
            counter_pipe: to_pomodoro_timer,
        }
    }

    pub fn start(&mut self) {
        self.counter_pipe.send(true).unwrap();
        self.pomodoro_state = PomodoroState::Run;
    }

    pub fn pause(&mut self) {
        self.counter_pipe.send(false).unwrap();
        self.pomodoro_state = PomodoroState::Pause;
    }

    pub fn resume(&mut self) {
        self.counter_pipe.send(true).unwrap();
        self.pomodoro_state = PomodoroState::Run;
    }

    pub fn stop(&mut self) {
        self.counter_pipe.send(false).unwrap();
        self.pomodoro_state = PomodoroState::Stop;
    }
    pub fn reset(&mut self) {
        self.stop();
        self.remaining_sec.store(0, Ordering::SeqCst);
        self.position = 0;
    }
}
pub(crate) struct PomodoroLength {
    pub focus: u32,
    pub relax: u32,
}

impl PomodoroLength {
    fn new(focus: u32, relax: u32) -> Self {
        Self {
            focus,
            relax,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum PomodoroState {
    Stop,
    Run,
    Pause,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum PomodoroPhase {
    BeforeFocus,
    Focus,
    BeforeRelax,
    Relax,
}