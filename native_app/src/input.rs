use common::FastSet;
use geom::{vec2, Vec2};
use std::fmt::Debug;
use winit::event::{ElementState, KeyboardInput, MouseScrollDelta, VirtualKeyCode, WindowEvent};

#[derive(Default)]
pub struct InputContext {
    pub mouse: MouseInfo,
    pub keyboard: KeyboardInfo,
}

impl InputContext {
    pub fn end_frame(&mut self) {
        self.mouse.just_pressed.clear();
        self.keyboard.just_pressed.clear();
        self.keyboard.last_characters.clear();
        self.mouse.wheel_delta = 0.0;
    }

    pub fn handle(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::ReceivedCharacter(c) => {
                self.keyboard.last_characters.push(*c);
                true
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(kc),
                        ..
                    },
                ..
            } => {
                let code = KeyCode::from(*kc);
                match state {
                    ElementState::Pressed => {
                        self.keyboard.pressed.insert(code);
                        self.keyboard.just_pressed.insert(code);
                    }
                    ElementState::Released => {
                        self.keyboard.pressed.remove(&code);
                    }
                };
                true
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse.screen = vec2(position.x as f32, position.y as f32);
                true
            }
            WindowEvent::MouseInput { button, state, .. } => {
                let b = MouseButton::from(*button);
                match state {
                    ElementState::Pressed => {
                        self.mouse.just_pressed.insert(b);
                        self.mouse.pressed.insert(b);
                    }
                    ElementState::Released => {
                        self.mouse.pressed.remove(&b);
                    }
                };
                true
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, y),
                ..
            } => {
                self.mouse.wheel_delta = *y;
                true
            }
            _ => false,
        }
    }
}

register_resource_noserialize!(MouseInfo);
#[derive(Clone, Default)]
pub struct MouseInfo {
    pub wheel_delta: f32,
    pub screen: Vec2,
    pub unprojected: Vec2,
    pub pressed: FastSet<MouseButton>,
    pub just_pressed: FastSet<MouseButton>,
}

register_resource_noserialize!(KeyboardInfo);
#[derive(Clone, Default)]
pub struct KeyboardInfo {
    pub just_pressed: FastSet<KeyCode>,
    pub pressed: FastSet<KeyCode>,
    pub last_characters: Vec<char>,
}

impl From<winit::event::MouseButton> for MouseButton {
    fn from(x: winit::event::MouseButton) -> MouseButton {
        match x {
            winit::event::MouseButton::Left => MouseButton::Left,
            winit::event::MouseButton::Right => MouseButton::Right,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            winit::event::MouseButton::Other(v) => MouseButton::Other(v),
        }
    }
}

impl From<VirtualKeyCode> for KeyCode {
    fn from(x: VirtualKeyCode) -> KeyCode {
        match x {
            winit::event::VirtualKeyCode::Key1 => KeyCode::Key1,
            winit::event::VirtualKeyCode::Key2 => KeyCode::Key2,
            winit::event::VirtualKeyCode::Key3 => KeyCode::Key3,
            winit::event::VirtualKeyCode::Key4 => KeyCode::Key4,
            winit::event::VirtualKeyCode::Key5 => KeyCode::Key5,
            winit::event::VirtualKeyCode::Key6 => KeyCode::Key6,
            winit::event::VirtualKeyCode::Key7 => KeyCode::Key7,
            winit::event::VirtualKeyCode::Key8 => KeyCode::Key8,
            winit::event::VirtualKeyCode::Key9 => KeyCode::Key9,
            winit::event::VirtualKeyCode::Key0 => KeyCode::Key0,
            winit::event::VirtualKeyCode::A => KeyCode::A,
            winit::event::VirtualKeyCode::B => KeyCode::B,
            winit::event::VirtualKeyCode::C => KeyCode::C,
            winit::event::VirtualKeyCode::D => KeyCode::D,
            winit::event::VirtualKeyCode::E => KeyCode::E,
            winit::event::VirtualKeyCode::F => KeyCode::F,
            winit::event::VirtualKeyCode::G => KeyCode::G,
            winit::event::VirtualKeyCode::H => KeyCode::H,
            winit::event::VirtualKeyCode::I => KeyCode::I,
            winit::event::VirtualKeyCode::J => KeyCode::J,
            winit::event::VirtualKeyCode::K => KeyCode::K,
            winit::event::VirtualKeyCode::L => KeyCode::L,
            winit::event::VirtualKeyCode::M => KeyCode::M,
            winit::event::VirtualKeyCode::N => KeyCode::N,
            winit::event::VirtualKeyCode::O => KeyCode::O,
            winit::event::VirtualKeyCode::P => KeyCode::P,
            winit::event::VirtualKeyCode::Q => KeyCode::Q,
            winit::event::VirtualKeyCode::R => KeyCode::R,
            winit::event::VirtualKeyCode::S => KeyCode::S,
            winit::event::VirtualKeyCode::T => KeyCode::T,
            winit::event::VirtualKeyCode::U => KeyCode::U,
            winit::event::VirtualKeyCode::V => KeyCode::V,
            winit::event::VirtualKeyCode::W => KeyCode::W,
            winit::event::VirtualKeyCode::X => KeyCode::X,
            winit::event::VirtualKeyCode::Y => KeyCode::Y,
            winit::event::VirtualKeyCode::Z => KeyCode::Z,
            winit::event::VirtualKeyCode::Escape => KeyCode::Escape,
            winit::event::VirtualKeyCode::F1 => KeyCode::F1,
            winit::event::VirtualKeyCode::F2 => KeyCode::F2,
            winit::event::VirtualKeyCode::F3 => KeyCode::F3,
            winit::event::VirtualKeyCode::F4 => KeyCode::F4,
            winit::event::VirtualKeyCode::F5 => KeyCode::F5,
            winit::event::VirtualKeyCode::F6 => KeyCode::F6,
            winit::event::VirtualKeyCode::F7 => KeyCode::F7,
            winit::event::VirtualKeyCode::F8 => KeyCode::F8,
            winit::event::VirtualKeyCode::F9 => KeyCode::F9,
            winit::event::VirtualKeyCode::F10 => KeyCode::F10,
            winit::event::VirtualKeyCode::F11 => KeyCode::F11,
            winit::event::VirtualKeyCode::F12 => KeyCode::F12,
            winit::event::VirtualKeyCode::F13 => KeyCode::F13,
            winit::event::VirtualKeyCode::F14 => KeyCode::F14,
            winit::event::VirtualKeyCode::F15 => KeyCode::F15,
            winit::event::VirtualKeyCode::F16 => KeyCode::F16,
            winit::event::VirtualKeyCode::F17 => KeyCode::F17,
            winit::event::VirtualKeyCode::F18 => KeyCode::F18,
            winit::event::VirtualKeyCode::F19 => KeyCode::F19,
            winit::event::VirtualKeyCode::F20 => KeyCode::F20,
            winit::event::VirtualKeyCode::F21 => KeyCode::F21,
            winit::event::VirtualKeyCode::F22 => KeyCode::F22,
            winit::event::VirtualKeyCode::F23 => KeyCode::F23,
            winit::event::VirtualKeyCode::Snapshot => KeyCode::Snapshot,
            winit::event::VirtualKeyCode::F24 => KeyCode::F24,
            winit::event::VirtualKeyCode::Scroll => KeyCode::Scroll,
            winit::event::VirtualKeyCode::Pause => KeyCode::Pause,
            winit::event::VirtualKeyCode::Insert => KeyCode::Insert,
            winit::event::VirtualKeyCode::Home => KeyCode::Home,
            winit::event::VirtualKeyCode::Delete => KeyCode::Delete,
            winit::event::VirtualKeyCode::End => KeyCode::End,
            winit::event::VirtualKeyCode::PageDown => KeyCode::PageDown,
            winit::event::VirtualKeyCode::PageUp => KeyCode::PageUp,
            winit::event::VirtualKeyCode::Left => KeyCode::Left,
            winit::event::VirtualKeyCode::Up => KeyCode::Up,
            winit::event::VirtualKeyCode::Right => KeyCode::Right,
            winit::event::VirtualKeyCode::Down => KeyCode::Down,
            winit::event::VirtualKeyCode::Back => KeyCode::Backspace,
            winit::event::VirtualKeyCode::Return => KeyCode::Return,
            winit::event::VirtualKeyCode::Space => KeyCode::Space,
            winit::event::VirtualKeyCode::Compose => KeyCode::Compose,
            winit::event::VirtualKeyCode::Caret => KeyCode::Caret,
            winit::event::VirtualKeyCode::Numlock => KeyCode::Numlock,
            winit::event::VirtualKeyCode::Numpad0 => KeyCode::Numpad0,
            winit::event::VirtualKeyCode::Numpad1 => KeyCode::Numpad1,
            winit::event::VirtualKeyCode::Numpad2 => KeyCode::Numpad2,
            winit::event::VirtualKeyCode::Numpad3 => KeyCode::Numpad3,
            winit::event::VirtualKeyCode::Numpad4 => KeyCode::Numpad4,
            winit::event::VirtualKeyCode::Numpad5 => KeyCode::Numpad5,
            winit::event::VirtualKeyCode::Numpad6 => KeyCode::Numpad6,
            winit::event::VirtualKeyCode::Numpad7 => KeyCode::Numpad7,
            winit::event::VirtualKeyCode::Numpad8 => KeyCode::Numpad8,
            winit::event::VirtualKeyCode::Numpad9 => KeyCode::Numpad9,
            winit::event::VirtualKeyCode::AbntC1 => KeyCode::AbntC1,
            winit::event::VirtualKeyCode::AbntC2 => KeyCode::AbntC2,
            winit::event::VirtualKeyCode::NumpadAdd => KeyCode::Add,
            winit::event::VirtualKeyCode::Apostrophe => KeyCode::Apostrophe,
            winit::event::VirtualKeyCode::Apps => KeyCode::Apps,
            winit::event::VirtualKeyCode::At => KeyCode::At,
            winit::event::VirtualKeyCode::Ax => KeyCode::Ax,
            winit::event::VirtualKeyCode::Backslash => KeyCode::Backslash,
            winit::event::VirtualKeyCode::Calculator => KeyCode::Calculator,
            winit::event::VirtualKeyCode::Capital => KeyCode::Capital,
            winit::event::VirtualKeyCode::Colon => KeyCode::Colon,
            winit::event::VirtualKeyCode::Comma => KeyCode::Comma,
            winit::event::VirtualKeyCode::Convert => KeyCode::Convert,
            winit::event::VirtualKeyCode::NumpadDecimal => KeyCode::Decimal,
            winit::event::VirtualKeyCode::NumpadDivide => KeyCode::Divide,
            winit::event::VirtualKeyCode::Equals => KeyCode::Equals,
            winit::event::VirtualKeyCode::Grave => KeyCode::Grave,
            winit::event::VirtualKeyCode::Kana => KeyCode::Kana,
            winit::event::VirtualKeyCode::Kanji => KeyCode::Kanji,
            winit::event::VirtualKeyCode::LAlt => KeyCode::LAlt,
            winit::event::VirtualKeyCode::LBracket => KeyCode::LBracket,
            winit::event::VirtualKeyCode::LControl => KeyCode::LControl,
            winit::event::VirtualKeyCode::LShift => KeyCode::LShift,
            winit::event::VirtualKeyCode::LWin => KeyCode::LWin,
            winit::event::VirtualKeyCode::Mail => KeyCode::Mail,
            winit::event::VirtualKeyCode::MediaSelect => KeyCode::MediaSelect,
            winit::event::VirtualKeyCode::MediaStop => KeyCode::MediaStop,
            winit::event::VirtualKeyCode::Minus => KeyCode::Minus,
            winit::event::VirtualKeyCode::NumpadMultiply => KeyCode::Multiply,
            winit::event::VirtualKeyCode::Mute => KeyCode::Mute,
            winit::event::VirtualKeyCode::MyComputer => KeyCode::MyComputer,
            winit::event::VirtualKeyCode::NavigateForward => KeyCode::NavigateForward,
            winit::event::VirtualKeyCode::NavigateBackward => KeyCode::NavigateBackward,
            winit::event::VirtualKeyCode::NextTrack => KeyCode::NextTrack,
            winit::event::VirtualKeyCode::NoConvert => KeyCode::NoConvert,
            winit::event::VirtualKeyCode::NumpadComma => KeyCode::NumpadComma,
            winit::event::VirtualKeyCode::NumpadEnter => KeyCode::NumpadEnter,
            winit::event::VirtualKeyCode::NumpadEquals => KeyCode::NumpadEquals,
            winit::event::VirtualKeyCode::OEM102 => KeyCode::OEM102,
            winit::event::VirtualKeyCode::Period => KeyCode::Period,
            winit::event::VirtualKeyCode::PlayPause => KeyCode::PlayPause,
            winit::event::VirtualKeyCode::Power => KeyCode::Power,
            winit::event::VirtualKeyCode::PrevTrack => KeyCode::PrevTrack,
            winit::event::VirtualKeyCode::RAlt => KeyCode::RAlt,
            winit::event::VirtualKeyCode::RBracket => KeyCode::RBracket,
            winit::event::VirtualKeyCode::RControl => KeyCode::RControl,
            winit::event::VirtualKeyCode::RShift => KeyCode::RShift,
            winit::event::VirtualKeyCode::RWin => KeyCode::RWin,
            winit::event::VirtualKeyCode::Semicolon => KeyCode::Semicolon,
            winit::event::VirtualKeyCode::Slash => KeyCode::Slash,
            winit::event::VirtualKeyCode::Sleep => KeyCode::Sleep,
            winit::event::VirtualKeyCode::Stop => KeyCode::Stop,
            winit::event::VirtualKeyCode::NumpadSubtract => KeyCode::Subtract,
            winit::event::VirtualKeyCode::Sysrq => KeyCode::Sysrq,
            winit::event::VirtualKeyCode::Tab => KeyCode::Tab,
            winit::event::VirtualKeyCode::Underline => KeyCode::Underline,
            winit::event::VirtualKeyCode::Unlabeled => KeyCode::Unlabeled,
            winit::event::VirtualKeyCode::VolumeDown => KeyCode::VolumeDown,
            winit::event::VirtualKeyCode::VolumeUp => KeyCode::VolumeUp,
            winit::event::VirtualKeyCode::Wake => KeyCode::Wake,
            winit::event::VirtualKeyCode::WebBack => KeyCode::WebBack,
            winit::event::VirtualKeyCode::WebFavorites => KeyCode::WebFavorites,
            winit::event::VirtualKeyCode::WebForward => KeyCode::WebForward,
            winit::event::VirtualKeyCode::WebHome => KeyCode::WebHome,
            winit::event::VirtualKeyCode::WebRefresh => KeyCode::WebRefresh,
            winit::event::VirtualKeyCode::WebSearch => KeyCode::WebSearch,
            winit::event::VirtualKeyCode::WebStop => KeyCode::WebStop,
            winit::event::VirtualKeyCode::Yen => KeyCode::Yen,
            winit::event::VirtualKeyCode::Copy => KeyCode::Copy,
            winit::event::VirtualKeyCode::Paste => KeyCode::Paste,
            winit::event::VirtualKeyCode::Cut => KeyCode::Cut,
            winit::event::VirtualKeyCode::Plus => KeyCode::Plus,
            winit::event::VirtualKeyCode::Asterisk => KeyCode::Asterisk,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

impl KeyCode {
    fn is_modifier(&self) -> bool {
        use KeyCode::*;
        matches!(self, LShift | RShift | LControl | RControl | LAlt | RAlt)
    }
}

/// Symbolic name for a keyboard key.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum KeyCode {
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Key0,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    /// The Escape key, next to F1.
    Escape,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    /// Print Screen/SysRq.
    Snapshot,
    /// Scroll Lock.
    Scroll,
    /// Pause/Break key, next to Scroll lock.
    Pause,

    /// `Insert`, next to Backspace.
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,

    Left,
    Up,
    Right,
    Down,

    /// The Backspace key, right over Enter.
    Backspace,
    /// The Enter key.
    Return,
    /// The space bar.
    Space,

    /// The "Compose" key on Linux.
    Compose,

    Caret,

    Plus,
    Asterisk,

    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,

    AbntC1,
    AbntC2,
    Add,
    Apostrophe,
    Apps,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Decimal,
    Divide,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Multiply,
    Mute,
    MyComputer,
    NavigateForward,  // also called "Prior"
    NavigateBackward, // also called "Next"
    NextTrack,
    NoConvert,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    OEM102,
    Period,
    PlayPause,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Subtract,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,
}
