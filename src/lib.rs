extern crate rand;

#[macro_use]
pub extern crate cursive;

mod game;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use cursive::views::*;
use cursive::{
    Cursive,
    Printer,
    align::HAlign,
    direction::{Absolute, Direction, Orientation},
    event::{Event, EventResult, Key},
    theme::{BaseColor, Color, ColorStyle, Effect, PaletteColor, Theme},
    utils::markup::StyledString,
    vec::Vec2,
    view::{Identifiable, ScrollStrategy, View, ViewWrapper},
};

const MAP_ID: &str = "map";
const MAIN_PANEL_ID: &str = "main_panel";

const CARDS_ID: &str = "cards";
const CARDS_DIALOG_ID: &str = "cards_dialog";

const INFO_ID: &str = "info";
const INFO_DIALOG_ID: &str = "info_dialog";

const QUIT_ID: &str = "quit";
const BUTTONS_ID: &str = "buttons";

// TODO: larger log window
const INFO_HEIGHT: usize = 8;

struct IsolateFocusView<T: View> {
    view: T,
}

impl<T: View> IsolateFocusView<T> {
    fn new(view: T) -> Self {
        Self { view }
    }
}

impl<T: View> ViewWrapper for IsolateFocusView<T> {
    wrap_impl!(self.view: T);

    fn wrap_take_focus(&mut self, source: Direction) -> bool {
        match source {
            // TODO: really return true?
            Direction::Abs(Absolute::None) => { self.view.take_focus(source); true },
            _ => false,
        }
    }
}

fn update_ui(s: &mut Cursive, game: &game::Game) {
    // TODO: improve this
    match *game {
        game::Game::Level(ref level) => {
            s.call_on_id(INFO_ID, |view: &mut TextView| {
                view.set_content(level.message_log());
            });
            s.call_on_id(CARDS_ID, |view: &mut TextView| {
                view.set_content(
                    level.player_deck().into_iter().enumerate().map(|(i, cv)| {
                        format!("{}] {:?} ({:?})\n", (i as u8 + b'a') as char, cv.card, cv.status)
                    }).collect::<String>()
                );
            });
        }
        game::Game::Transition(ref _trans) => {
            s.call_on_id(INFO_ID, |view: &mut TextView| {
                view.set_content("");
            });
            s.call_on_id(CARDS_ID, |view: &mut TextView| {
                view.set_content("");
            });
        }
        game::Game::Victory => {
            s.call_on_id(INFO_ID, |view: &mut TextView| {
                view.set_content("You won?");
            });
            s.call_on_id(CARDS_ID, |view: &mut TextView| {
                view.set_content("");
            });
        }
    };
}

fn process_map_event(game: &mut Rc<RefCell<game::Game>>, event: Event) -> EventResult {
    let action_cb = move |game: Rc<RefCell<game::Game>>, action| EventResult::with_cb(move |s| {
        let mut game = game.borrow_mut();
        let updated;
        {
            let level = match *game {
                game::Game::Level(ref mut level) => level,
                _ => { return; }
            };
            updated = level.step(action)
        }
        update_ui(s, &game);
        // TODO: make less terrible?
        if updated {
            let size = s.screen_size();
            s.screen_mut().layout(size);
            s.call_on_id(INFO_ID, |view: &mut TextView| {
                view.scroll_bottom();
            });
        }
    });
    let game = game.clone();
    match event {
        // TODO: include controls in button panel?
        // TODO: adjust controls? (wasd, vi-keys, rethink recover/wait, etc)
        Event::Char('m') => EventResult::with_cb(|s| s.focus_id(INFO_ID).unwrap()),
        Event::Char('i') => EventResult::with_cb(|s| s.focus_id(CARDS_ID).unwrap()),
        Event::Key(Key::Up) => action_cb(game, game::Action::Move(game::Direction::Up)),
        Event::Key(Key::Down) => action_cb(game, game::Action::Move(game::Direction::Down)),
        Event::Key(Key::Left) => action_cb(game, game::Action::Move(game::Direction::Left)),
        Event::Key(Key::Right) => action_cb(game, game::Action::Move(game::Direction::Right)),
        Event::Char('.') => action_cb(game, game::Action::Rest),
        Event::Char('r') => action_cb(game, game::Action::Rest),
        Event::Char('w') => action_cb(game, game::Action::Wait),
        Event::Char(' ') => EventResult::with_cb(move |s| {
            // TODO: handle level transitions
            let mut game = game.borrow_mut();
            game.update();
            update_ui(s, &game);
        }),
        _ => EventResult::Ignored,
    }
}

fn draw_map(game: &Rc<RefCell<game::Game>>, p: &Printer) {
    // TODO: camera movement
    let game = game.borrow();
    let level = match *game {
        game::Game::Level(ref level) => level,
        _ => { return; }
    };
    for x in 0..p.size.x {
        for y in 0..p.size.y {
            let g = level.view(game::Position { x: x as i32, y: y as i32 });
            let color = if g.is_visible() {
                if p.focused {
                    ColorStyle::secondary()
                } else {
                    ColorStyle::primary()
                }
            } else {
                ColorStyle::tertiary()
            };
            p.with_color(color, |p| {
                let ch = g.ch();
                if ch == '#' {
                    p.with_style(Effect::Reverse, |p| {
                        p.print(Vec2::new(x, y), "#");
                    });
                } else {
                    p.print(Vec2::new(x, y), &format!("{}", g.ch()));
                }
            });
        }
    }
}

struct ToggleInterceptorView<T: View> {
    view: T,
    game: Rc<RefCell<game::Game>>,
    focused: Rc<Cell<bool>>,
}

impl<T: View> ToggleInterceptorView<T> {
    fn new(view: T, game: Rc<RefCell<game::Game>>, focused: Rc<Cell<bool>>) -> Self {
        Self { view, game, focused }
    }
}

impl<T: View> ViewWrapper for ToggleInterceptorView<T> {
    wrap_impl!(self.view: T);

    fn wrap_on_event(&mut self, e: Event) -> EventResult {
        match e {
            Event::Char(ch) => {
                if 'a' <= ch && ch <= 'z' {
                    let game = self.game.clone();
                    return EventResult::with_cb(move |_s| {
                        let mut game = game.borrow_mut();
                        {
                            let _level = match *game {
                                game::Game::Level(ref mut level) => level,
                                _ => { return; }
                            };
                            // TODO: explain card (if any)
                        }
                    });
                }
                if 'A' <= ch && ch <= 'Z' {
                    let game = self.game.clone();
                    return EventResult::with_cb(move |s| {
                        let mut game = game.borrow_mut();
                        let success;
                        {
                            let level = match *game {
                                game::Game::Level(ref mut level) => level,
                                _ => { return; }
                            };
                            success = level.step(game::Action::Toggle((ch as u8 - b'A') as usize));
                        }
                        if success {
                            update_ui(s, &game);
                        }
                    });
                }
            }
            _ => {}
        };
        self.view.on_event(e)
    }

    fn wrap_draw(&self, printer: &Printer) {
        self.focused.set(printer.focused);
        self.view.draw(printer);
    }
}

pub fn build_ui(siv: &mut Cursive, seed: u32) {
    let game = Rc::new(RefCell::new(game::Game::new(seed)));

    let mut theme = Theme::default();
    theme.palette[PaletteColor::Secondary] = Color::Dark(BaseColor::Blue);
    theme.palette[PaletteColor::Tertiary] = Color::Dark(BaseColor::Cyan);
    siv.set_theme(theme);

    // TODO: add button and key binding for new game
    siv.add_global_callback(Event::CtrlChar('q'), |s| s.quit());
    siv.add_global_callback('?', |_| ());
    siv.add_global_callback(Key::Esc, |s| s.focus_id(MAP_ID).unwrap());
    siv.add_global_callback(' ', |s| s.focus_id(MAP_ID).unwrap());

    siv.add_fullscreen_layer(BoxView::with_full_screen(
        LinearLayout::new(Orientation::Vertical)
            .child(BoxView::with_full_width(
                IsolateFocusView::new(
                    // TODO: add buttons for rest and wait
                    LinearLayout::new(Orientation::Horizontal)
                        .child(BoxView::with_fixed_width(1, DummyView))
                        // TODO: Make quit button conditional on target (hide for WASM)
                        // .child(Button::new("[Ctrl+q] Quit", |s| s.quit()).with_id(QUIT_ID))
                        // .child(BoxView::with_fixed_width(2, DummyView))
                        // TODO: actually implement Help dialog
                        .child(Button::new("[?] Help", |_| ()))
                        // .child(BoxView::with_fixed_width(2, DummyView))
                        // .child(Button::new("[Space] Restore focus", |s| s.focus_id(MAP_ID).unwrap()))
                        .with_id(BUTTONS_ID)
                )
            ))
            .child(BoxView::with_full_screen(
                LinearLayout::new(Orientation::Horizontal)
                    .child(BoxView::with_full_screen(
                        Panel::new(
                            Canvas::new(game.clone())
                                .with_take_focus(|_, dir| dir == Direction::Abs(Absolute::None))
                                .with_on_event(process_map_event)
                                .with_draw(draw_map)
                                .with_id(MAP_ID)
                        ).with_id(MAIN_PANEL_ID)
                    ))
                    .child(BoxView::with_fixed_width(
                        41,
                        IsolateFocusView::new({
                            let focused = Rc::new(Cell::new(false));
                            ToggleInterceptorView::new(
                                Dialog::around(
                                    LinearLayout::new(Orientation::Vertical)
                                        .child(BoxView::with_full_height(
                                            TextView::new("")
                                                .scrollable(true)
                                                .with_id(CARDS_ID)
                                        ))
                                        .child(BoxView::with_fixed_height(
                                            1,
                                            // TODO: Don't include Toggle if exiting level
                                            Canvas::wrap(TextView::new(
                                                StyledString::single_span(
                                                    "[a-z] Examine  [A-Z] Toggle",
                                                    ColorStyle::secondary().into(),
                                                )
                                            ))
                                                .with_draw({
                                                    let focused = focused.clone();
                                                    move |v, p| {
                                                        if focused.get() {
                                                            v.draw(p);
                                                        }
                                                    }
                                                }),
                                        ))
                                )
                                    .title("Deck")
                                    .title_position(HAlign::Left)
                                    .with_id(CARDS_DIALOG_ID),
                                game.clone(),
                                focused,
                            )
                        }),
                    ))
            ))
            .child(BoxView::with_fixed_height(
                INFO_HEIGHT,
                IsolateFocusView::new(
                    OnEventView::new(
                        Dialog::around(
                            TextView::new("")
                                .scrollable(true)
                                .scroll_strategy(ScrollStrategy::StickToBottom)
                                .with_id(INFO_ID)
                        )
                            .title("Log")
                            .title_position(HAlign::Left)
                            .with_id(INFO_DIALOG_ID)
                    ).on_event('i', |s| s.focus_id(CARDS_ID).unwrap())
                ),
            ))
    ));

    update_ui(siv, &game.borrow());
    // siv.focus_id(QUIT_ID).unwrap();
    siv.focus_id(MAP_ID).unwrap();
}
