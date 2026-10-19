//! gilrs boundary and deterministic input mapping. Tuning is proposed, needs Deck validation.
use gilrs::{Axis, Button, EventType, Gilrs};
use std::{collections::BTreeMap, time::Duration};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Back,
    X,
    Y,
    PreviousTab,
    NextTab,
    /// L2/R2 on Discover: previous/next category (spec 015, R4).
    PreviousCategory,
    NextCategory,
    Menu,
    Tab(usize),
}
impl Action {
    pub fn from_code(code: i32) -> Option<Self> {
        Some(match code {
            0 => Self::Up,
            1 => Self::Down,
            2 => Self::Left,
            3 => Self::Right,
            4 => Self::Confirm,
            5 => Self::Back,
            6 => Self::X,
            7 => Self::Y,
            8 => Self::PreviousTab,
            9 => Self::NextTab,
            10 => Self::Menu,
            11 => Self::PreviousCategory,
            12 => Self::NextCategory,
            20..=23 => Self::Tab((code - 20) as usize),
            _ => return None,
        })
    }
}
/// Coordinator-approved starting values, all Needs Deck Validation.
#[derive(Clone, Copy)]
pub struct InputConfig {
    pub deadzone: f32,
    pub delay: Duration,
    pub rate: Duration,
    pub poll: Duration,
}
pub const CONFIG: InputConfig = InputConfig {
    deadzone: 0.35,
    delay: Duration::from_millis(350),
    rate: Duration::from_millis(100),
    poll: Duration::from_millis(16),
};
#[derive(Clone, Copy)]
pub enum Signal {
    Axis(Axis, f32),
    Button(Button, bool),
    Connected,
    Disconnected,
    Ignore,
}
impl From<EventType> for Signal {
    fn from(event: EventType) -> Self {
        match event {
            EventType::AxisChanged(a, v, _) => Self::Axis(a, v),
            EventType::ButtonPressed(b, _) => Self::Button(b, true),
            EventType::ButtonReleased(b, _) => Self::Button(b, false),
            EventType::Connected => Self::Connected,
            EventType::Disconnected => Self::Disconnected,
            _ => Self::Ignore,
        }
    }
}
#[derive(Default)]
struct Pad {
    x: f32,
    y: f32,
    dpad: [bool; 4],
    held: Option<Action>,
    next: Duration,
    blocked: bool,
    buttons: Vec<Button>,
}
impl Pad {
    fn direction(&self) -> Option<Action> {
        for (i, held) in self.dpad.iter().enumerate() {
            if *held {
                return Some([Action::Up, Action::Down, Action::Left, Action::Right][i]);
            }
        }
        if self.x.abs().max(self.y.abs()) < CONFIG.deadzone {
            return None;
        }
        Some(if self.x.abs() > self.y.abs() {
            if self.x > 0. {
                Action::Right
            } else {
                Action::Left
            }
        } else if self.y > 0. {
            Action::Up
        } else {
            Action::Down
        })
    }
    fn sample(&mut self, now: Duration) -> Option<Action> {
        let direction = if self.buttons.contains(&Button::LeftTrigger2) {
            Some(Action::PreviousCategory)
        } else if self.buttons.contains(&Button::RightTrigger2) {
            Some(Action::NextCategory)
        } else {
            self.direction()
        };
        if self.blocked {
            return None;
        }
        if direction != self.held {
            self.held = direction;
            self.next = now
                + if matches!(
                    direction,
                    Some(Action::PreviousCategory | Action::NextCategory)
                ) {
                    Duration::from_millis(500)
                } else {
                    CONFIG.delay
                };
            return direction;
        }
        if direction.is_some() && now >= self.next {
            self.next = now + CONFIG.rate;
            return direction;
        }
        None
    }
}
#[derive(Default)]
pub struct Mapper {
    pads: BTreeMap<usize, Pad>,
}
impl Mapper {
    pub fn event(&mut self, id: usize, event: Signal, now: Duration) -> Option<Action> {
        if matches!(event, Signal::Disconnected | Signal::Connected) {
            self.pads.remove(&id);
            if matches!(event, Signal::Connected) {
                self.pads.insert(
                    id,
                    Pad {
                        blocked: true,
                        ..Default::default()
                    },
                );
            }
            return None;
        }
        let pad = self.pads.entry(id).or_default();
        match event {
            Signal::Axis(Axis::LeftStickX, value) => {
                pad.x = if value.is_finite() { value } else { 0. }
            }
            Signal::Axis(Axis::LeftStickY, value) => {
                pad.y = if value.is_finite() { value } else { 0. }
            }
            Signal::Button(button, down) => {
                if down {
                    if !pad.buttons.contains(&button) {
                        pad.buttons.push(button);
                    }
                } else {
                    pad.buttons.retain(|b| *b != button);
                }
                if let Some(i) = [
                    Button::DPadUp,
                    Button::DPadDown,
                    Button::DPadLeft,
                    Button::DPadRight,
                ]
                .iter()
                .position(|b| *b == button)
                {
                    pad.dpad[i] = down;
                } else if !matches!(button, Button::LeftTrigger2 | Button::RightTrigger2)
                    && down
                    && !pad.blocked
                {
                    return match button {
                        Button::South => Some(Action::Confirm),
                        Button::East => Some(Action::Back),
                        Button::West => Some(Action::X),
                        Button::North => Some(Action::Y),
                        Button::LeftTrigger => Some(Action::PreviousTab),
                        Button::RightTrigger => Some(Action::NextTab),
                        Button::Start => Some(Action::Menu),
                        _ => None,
                    };
                }
            }
            // gilrs repeats are deliberately ignored: our timer is the sole repeat source.
            _ => return None,
        }
        if pad.blocked && pad.direction().is_none() && pad.buttons.is_empty() {
            pad.blocked = false;
        }
        pad.sample(now)
    }
    pub fn suspend(&mut self) {
        for pad in self.pads.values_mut() {
            pad.held = None;
            pad.blocked = true;
        }
    }
    pub fn repeat(&mut self, now: Duration) -> Vec<Action> {
        self.pads
            .values_mut()
            .filter_map(|p| p.sample(now))
            .collect()
    }
}
/// Retained by the main loop. Failing to open devices leaves keyboard navigation usable.
pub struct Gamepads {
    gilrs: Option<Gilrs>,
    mapper: Mapper,
    epoch: std::time::Instant,
}
impl Gamepads {
    pub fn new() -> Self {
        let gilrs = match gilrs::GilrsBuilder::new()
            .with_force_feedback(false)
            .build()
        {
            Ok(g) => Some(g),
            Err(e) => {
                tracing::warn!(error=%e,"controller backend unavailable");
                None
            }
        };
        Self {
            gilrs,
            mapper: Mapper::default(),
            epoch: std::time::Instant::now(),
        }
    }
    pub fn poll(&mut self, active: bool) -> (Vec<Action>, Vec<bool>) {
        let now = self.epoch.elapsed();
        if !active {
            self.mapper.suspend();
        }
        let mut actions = vec![];
        let mut connections = vec![];
        if let Some(g) = &mut self.gilrs {
            while let Some(event) = g.next_event() {
                match event.event {
                    EventType::Connected => connections.push(true),
                    EventType::Disconnected => connections.push(false),
                    _ => {}
                }
                if let Some(a) = self
                    .mapper
                    .event(usize::from(event.id), event.event.into(), now)
                    && active
                {
                    actions.push(a);
                }
            }
        }
        if active {
            actions.extend(self.mapper.repeat(now));
        }
        (actions, connections)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stick_deadzone_direction_repeat_and_neutral() {
        let mut p = Pad {
            x: CONFIG.deadzone - 0.01,
            ..Default::default()
        };
        assert_eq!(p.sample(Duration::ZERO), None);
        p.x = 0.8;
        assert_eq!(p.sample(Duration::ZERO), Some(Action::Right));
        assert_eq!(p.sample(CONFIG.delay - Duration::from_millis(1)), None);
        assert_eq!(p.sample(CONFIG.delay), Some(Action::Right));
        assert_eq!(
            p.sample(CONFIG.delay + CONFIG.rate - Duration::from_millis(1)),
            None
        );
        assert_eq!(p.sample(CONFIG.delay + CONFIG.rate), Some(Action::Right));
        p.x = 0.;
        assert_eq!(p.sample(CONFIG.delay + CONFIG.rate), None);
        p.y = -0.9;
        assert_eq!(p.sample(CONFIG.delay + CONFIG.rate), Some(Action::Down));
    }
    #[test]
    fn mapper_axes_disconnect_and_focus_loss_require_neutral() {
        let mut mapper = Mapper::default();
        assert_eq!(
            mapper.event(7, Signal::Axis(Axis::LeftStickX, 0.1), Duration::ZERO),
            None
        );
        assert_eq!(
            mapper.event(7, Signal::Axis(Axis::LeftStickX, 0.8), Duration::ZERO),
            Some(Action::Right)
        );
        mapper.suspend();
        assert!(mapper.repeat(CONFIG.delay).is_empty());
        assert_eq!(
            mapper.event(7, Signal::Button(Button::South, true), CONFIG.delay),
            None
        );
        mapper.event(7, Signal::Axis(Axis::LeftStickX, 0.), CONFIG.delay);
        assert!(mapper.repeat(CONFIG.delay).is_empty());
        mapper.event(7, Signal::Button(Button::South, false), CONFIG.delay);
        assert_eq!(
            mapper.event(7, Signal::Button(Button::South, true), CONFIG.delay),
            Some(Action::Confirm)
        );
        mapper.event(7, Signal::Disconnected, CONFIG.delay);
        mapper.event(7, Signal::Connected, CONFIG.delay);
        assert_eq!(
            mapper.event(7, Signal::Axis(Axis::LeftStickY, 1.), CONFIG.delay),
            None
        );
        mapper.event(7, Signal::Axis(Axis::LeftStickY, 0.), CONFIG.delay);
        assert_eq!(
            mapper.event(7, Signal::Axis(Axis::LeftStickY, 1.), CONFIG.delay),
            Some(Action::Up)
        );
        assert_eq!(
            mapper.event(8, Signal::Axis(Axis::RightStickY, 1.), CONFIG.delay),
            None
        );
    }
    #[test]
    fn category_hold_delays_repeats_and_release_or_suspend_stops() {
        for (button, action) in [
            (Button::LeftTrigger2, Action::PreviousCategory),
            (Button::RightTrigger2, Action::NextCategory),
        ] {
            let mut mapper = Mapper::default();
            assert_eq!(
                mapper.event(0, Signal::Button(button, true), Duration::ZERO),
                Some(action)
            );
            assert!(mapper.repeat(Duration::from_millis(499)).is_empty());
            assert_eq!(mapper.repeat(Duration::from_millis(500)), vec![action]);
            assert!(mapper.repeat(Duration::from_millis(599)).is_empty());
            assert_eq!(mapper.repeat(Duration::from_millis(600)), vec![action]);
            mapper.event(0, Signal::Button(button, false), Duration::from_millis(650));
            assert!(mapper.repeat(Duration::from_secs(1)).is_empty());
            mapper.event(0, Signal::Button(button, true), Duration::from_secs(1));
            mapper.suspend();
            assert!(mapper.repeat(Duration::from_secs(2)).is_empty());
        }
    }
    #[test]
    fn missing_gamepad_backend_is_idle() {
        let mut devices = Gamepads {
            gilrs: None,
            mapper: Mapper::default(),
            epoch: std::time::Instant::now(),
        };
        assert_eq!(devices.poll(true), (vec![], vec![]));
        assert_eq!(devices.poll(false), (vec![], vec![]));
        assert!(matches!(
            Signal::from(EventType::Connected),
            Signal::Connected
        ));
        assert!(matches!(
            Signal::from(EventType::Disconnected),
            Signal::Disconnected
        ));
    }
    #[test]
    fn buttons_map_once_and_hotplug_clears_hold() {
        let mut m = Mapper::default();
        for (b, a) in [
            (Button::South, Action::Confirm),
            (Button::East, Action::Back),
            (Button::West, Action::X),
            (Button::North, Action::Y),
            (Button::LeftTrigger, Action::PreviousTab),
            (Button::RightTrigger, Action::NextTab),
            (Button::LeftTrigger2, Action::PreviousCategory),
            (Button::RightTrigger2, Action::NextCategory),
            (Button::Start, Action::Menu),
        ] {
            assert_eq!(m.event(0, Signal::Button(b, true), Duration::ZERO), Some(a));
            assert_eq!(m.event(0, Signal::Button(b, false), Duration::ZERO), None);
            assert_eq!(m.event(0, Signal::Ignore, Duration::ZERO), None);
        }
        assert_eq!(
            m.event(0, Signal::Button(Button::DPadDown, true), Duration::ZERO),
            Some(Action::Down)
        );
        assert_eq!(m.repeat(CONFIG.delay), vec![Action::Down]);
        m.event(0, Signal::Disconnected, CONFIG.delay);
        assert!(m.repeat(CONFIG.delay + CONFIG.rate).is_empty());
        m.event(0, Signal::Connected, CONFIG.delay + CONFIG.rate);
        assert!(m.repeat(CONFIG.delay + CONFIG.rate).is_empty());
    }
}
