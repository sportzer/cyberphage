#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate cyberphage;
extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::collections::VecDeque;

use cyberphage::cursive::{
    Cursive,
    event::{Event, Key, MouseButton, MouseEvent},
    backend::Backend,
    theme::{BaseColor, Color, ColorPair, Effect},
    vec::Vec2,
};

// TODO: care about Effects? (I do use reverse...)
// TODO: care about proper unicode rendering?
#[derive(Copy, Clone)]
struct Glyph {
    ch: char,
    colors: ColorPair,
}

impl Default for Glyph {
    fn default() -> Glyph {
        Glyph {
            ch: ' ',
            colors: ColorPair {
                front: Color::TerminalDefault,
                back: Color::TerminalDefault,
            },
        }
    }
}

struct FakeTerminal {
    width: usize,
    height: usize,
    glyphs: Vec<Glyph>,
    events: VecDeque<Event>,
}

// TODO: initialize with width and height?
impl FakeTerminal {
    fn new() -> FakeTerminal {
        FakeTerminal {
            width: 0,
            height: 0,
            glyphs: Vec::new(),
            events: VecDeque::new(),
        }
    }
}

struct FakeTerminalBackend {
    colors: Cell<ColorPair>,
    terminal: Rc<RefCell<FakeTerminal>>,
}

impl Backend for FakeTerminalBackend {
    fn has_colors(&self) -> bool {
        true
    }

    fn screen_size(&self) -> Vec2 {
        let term = self.terminal.borrow();
        Vec2 { x: term.width, y: term.height }
    }

    fn poll_event(&mut self) -> Event {
        self.terminal.borrow_mut().events.pop_front().unwrap_or(Event::Exit)
    }

    fn print_at(&self, pos: Vec2, s: &str) {
        let mut term = self.terminal.borrow_mut();
        for (i, ch) in s.chars().enumerate() {
            let (x, y) = (pos.x + i, pos.y);
            if x < term.width && y < term.height {
                let index = x + term.width * y;
                term.glyphs[index] = Glyph { ch, colors: self.colors.get() };
            }
        }
    }

    fn clear(&self, color: Color) {
        let glyph = Glyph {
            ch: ' ',
            colors: ColorPair {
                front: Color::TerminalDefault,
                back: color,
            },
        };
        let mut term = self.terminal.borrow_mut();
        for g in &mut term.glyphs {
            *g = glyph;
        }
    }

    fn set_color(&self, colors: ColorPair) -> ColorPair {
        self.colors.replace(colors)
    }

    fn finish(&mut self) {}
    fn refresh(&mut self) {}
    fn set_refresh_rate(&mut self, _: u32) {}
    fn set_effect(&self, _: Effect) {}
    fn unset_effect(&self, _: Effect) {}
}

#[wasm_bindgen]
pub struct Game {
    terminal: Rc<RefCell<FakeTerminal>>,
    ui: Cursive,
}

impl Game {
    fn get_glyph(&self, x: usize, y: usize) -> Glyph {
        let term = self.terminal.borrow();
        if x < term.width && y < term.height {
            term.glyphs[x + y * term.width]
        } else {
            Glyph::default()
        }
    }

    fn push_event(&self, event: Event) {
        self.terminal.borrow_mut().events.push_back(event);
    }
}

fn key_code_to_key(key_code: u32) -> Option<Key> {
    Some(match key_code {
        13 => Key::Enter,
        9 => Key::Tab,
        8 => Key::Backspace,
        27 => Key::Esc,
        37 => Key::Left,
        39 => Key::Right,
        38 => Key::Up,
        40 => Key::Down,
        45 => Key::Ins,
        46 => Key::Del,
        36 => Key::Home,
        35 => Key::End,
        33 => Key::PageUp,
        34 => Key::PageDown,
        19 => Key::PauseBreak,
        12 => Key::NumpadCenter,
        112 => Key::F1,
        113 => Key::F2,
        114 => Key::F3,
        115 => Key::F4,
        116 => Key::F5,
        117 => Key::F6,
        118 => Key::F7,
        119 => Key::F8,
        120 => Key::F9,
        121 => Key::F10,
        122 => Key::F11,
        123 => Key::F12,
        _ => { return None }
    })
}

fn color_to_rgb(color: Color) -> Option<[u8; 3]> {
    Some(match color {
        Color::TerminalDefault => { return None; }
        Color::Dark(color) => match color {
            BaseColor::Black => [0x00, 0x00, 0x00],
            BaseColor::Red => [0x80, 0x00, 0x00],
            BaseColor::Green => [0x00, 0x80, 0x00],
            BaseColor::Yellow => [0x80, 0x80, 0x00],
            BaseColor::Blue => [0x00, 0x00, 0x80],
            BaseColor::Magenta => [0x80, 0x00, 0x80],
            BaseColor::Cyan => [0x00, 0x80, 0x80],
            BaseColor::White => [0xc0, 0xc0, 0xc0],
        },
        Color::Light(color) => match color {
            BaseColor::Black => [0x80, 0x80, 0x80],
            BaseColor::Red => [0xff, 0x00, 0x00],
            BaseColor::Green => [0x00, 0xff, 0x00],
            BaseColor::Yellow => [0xff, 0xff, 0x00],
            BaseColor::Blue => [0x00, 0x00, 0xff],
            BaseColor::Magenta => [0xff, 0x00, 0xff],
            BaseColor::Cyan => [0x00, 0xff, 0xff],
            BaseColor::White => [0xff, 0xff, 0xff],
        },
        Color::Rgb(r, g, b) => [r, g, b],
        Color::RgbLowRes(r, g, b) => [r*51, g*51, b*51],
    })
}

fn rgb_to_u32(rgb: [u8; 3]) -> u32 {
    (rgb[0] as u32) + ((rgb[1] as u32) << 8) + ((rgb[2] as u32) << 16)
}

#[wasm_bindgen]
impl Game {
    pub fn new(seed: u32) -> Game {
        let term = Rc::new(RefCell::new(FakeTerminal::new()));
        let backend = FakeTerminalBackend {
            colors: Cell::new(ColorPair {
                front: Color::TerminalDefault,
                back: Color::TerminalDefault,
            }),
            terminal: term.clone(),
        };
        let mut siv = Cursive::new(Box::new(backend));
        // TODO: hide quit button
        cyberphage::build_ui(&mut siv, seed);
        Game { terminal: term, ui: siv }
    }

    // TODO: don't just clear everything on resize?
    pub fn set_size(&self, width: usize, height: usize) {
        let mut term = self.terminal.borrow_mut();
        term.width = width;
        term.height = height;
        term.glyphs.clear();
        term.glyphs.reserve(width*height);
        for _ in 0..width*height {
            term.glyphs.push(Glyph::default());
        }
        term.events.push_back(Event::WindowResize);
    }

    pub fn run(&mut self) {
        self.push_event(Event::Refresh);
        self.ui.run();
    }

    pub fn get_ch(&self, x: usize, y: usize) -> u32 {
        self.get_glyph(x, y).ch as u32
    }

    pub fn get_fg(&self, x: usize, y: usize) -> u32 {
        let color = self.get_glyph(x, y).colors.front;
        let rgb = color_to_rgb(color).unwrap_or([0xff, 0xff, 0xff]);
        rgb_to_u32(rgb)
    }

    pub fn get_bg(&self, x: usize, y: usize) -> u32 {
        let color = self.get_glyph(x, y).colors.back;
        let rgb = color_to_rgb(color).unwrap_or([0x00, 0x00, 0x00]);
        rgb_to_u32(rgb)
    }

    // TODO: drop ctrl+alt+shift instead of mapping it to ctrl+alt?
    pub fn push_keydown_event(&self, key_code: u32, ctrl: bool, alt: bool, shift: bool) {
        key_code_to_key(key_code).map(|key| {
            self.push_event(match (ctrl, alt, shift) {
                (false, false, false) => Event::Key(key),
                (true, true, _) => Event::CtrlAlt(key),
                (true, false, true) => Event::CtrlShift(key),
                (false, true, true) => Event::AltShift(key),
                (true, false, false) => Event::Ctrl(key),
                (false, true, false) => Event::Alt(key),
                (false, false, true) => Event::Shift(key),
            });
        });
    }

    // TODO: drop ctrl+alt instead of mapping it to ctrl?
    pub fn push_keypress_event(&self, char_code: u32, ctrl: bool, alt: bool) {
        std::char::from_u32(char_code).map(|ch| {
            self.push_event(match (ctrl, alt) {
                (false, false) => Event::Char(ch),
                (true, _) => Event::CtrlChar(ch),
                (false, true) => Event::AltChar(ch),
            });
        });
    }

    pub fn push_mouse_press_event(&self, x: usize, y: usize, button: u32) {
        let button = match button {
            0 => MouseButton::Left,
            1 => MouseButton::Middle,
            2 => MouseButton::Right,
            3 => MouseButton::Button4,
            4 => MouseButton::Button5,
            _ => { return; }
        };
        let mut term = self.terminal.borrow_mut();
        if x < term.width && y < term.height {
            term.events.push_back(Event::Mouse {
                offset: Vec2 { x: 0, y: 0 },
                position: Vec2 { x, y },
                event: MouseEvent::Press(button),
            });
        }
    }

    // TODO: dedup with push_mouse_press_event
    pub fn push_mouse_release_event(&self, x: usize, y: usize, button: u32) {
        let button = match button {
            0 => MouseButton::Left,
            1 => MouseButton::Middle,
            2 => MouseButton::Right,
            3 => MouseButton::Button4,
            4 => MouseButton::Button5,
            _ => { return; }
        };
        let mut term = self.terminal.borrow_mut();
        if x < term.width && y < term.height {
            term.events.push_back(Event::Mouse {
                offset: Vec2 { x: 0, y: 0 },
                position: Vec2 { x, y },
                event: MouseEvent::Release(button),
            });
        }
    }

    // TODO: implement MouseEvent::Hold support

    pub fn push_mouse_wheel_event(&self, x: usize, y: usize, delta: i32) {
        let event = Event::Mouse {
            offset: Vec2 { x: 0, y: 0 },
            position: Vec2 { x, y },
            event: if delta > 0 { MouseEvent::WheelDown } else { MouseEvent::WheelUp },
        };
        let mut term = self.terminal.borrow_mut();
        for _ in 0..delta.abs() {
            if x < term.width && y < term.height {
                term.events.push_back(event.clone());
            }
        }
    }
}
