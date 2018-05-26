use std::collections::{BTreeMap, HashSet};

use rand::{Rng, SeedableRng, StdRng};

mod gen;

pub const MAP_WIDTH: usize = 37;
pub const MAP_HEIGHT: usize = 25;

#[derive(Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
struct Entity(u64);

const PLAYER: Entity = Entity(0);

#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum Tile {
    Floor,
    Wall,
    Door,
    Exit,
}

impl Tile {
    fn render(self) -> char {
        match self {
            Tile::Floor => '.',
            Tile::Wall => '#',
            Tile::Door => '+',
            Tile::Exit => '>',
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum Visibility {
    Visible,
    Remembered,
    Unknown,
}

#[derive(Copy, Clone)]
struct Square {
    tile: Tile,
    entity: Option<Entity>,
    visibility: Visibility,
}

impl Square {
    fn is_open(self) -> bool {
        self.entity.is_none() && self.tile != Tile::Wall
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum EntityType {
    UnknownThing,
    Player,
    Defender,
    Hunter,
    Reaper,
}

impl EntityType {
    fn render(self) -> char {
        match self {
            EntityType::UnknownThing => '?',
            EntityType::Player => '@',
            EntityType::Defender => 'd',
            EntityType::Hunter => 'h',
            EntityType::Reaper => 'r',
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum Goal {
    Move(Position),
    // Attack(Entity),
    // Wait,
    // Rest,
}

// TODO: real display functionality
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub enum Card {
    Attack(i32),
    Defend(i32),
    Kill(i32),
    Strike,
    Dodge,
    Block,
    Push,
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum CardStatus {
    Active,
    Inactive,
    Discarded,
    PlayedOn(Entity),
}

impl CardStatus {
    fn in_hand(self) -> bool {
        match self {
            CardStatus::Active => true,
            CardStatus::Inactive => true,
            CardStatus::Discarded => false,
            CardStatus::PlayedOn(_) => false,
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
struct CardState {
    card: Card,
    status: CardStatus,
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum Modifier {
}

#[derive(Eq, PartialEq, Copy, Clone)]
struct Modification {
    source: Entity,
    source_index: usize,
    modifier: Modifier,
}

// TODO: make less dumb
#[derive(Eq, PartialEq, Copy, Clone)]
enum CardOutcome {
    Continue,
    Done,
    Discard,
    Cancel,
    DiscardAndCancel,
}

struct MessageLog {
    messages: Vec<String>,
}

impl MessageLog {
    fn new() -> MessageLog {
        MessageLog {
            messages: Vec::new(),
        }
    }
}

pub struct Level {
    level: i32,
    last_id: Entity,
    map: [[Square; MAP_WIDTH]; MAP_HEIGHT],
    positions: BTreeMap<Entity, Position>,

    types: BTreeMap<Entity, EntityType>,
    goals: BTreeMap<Entity, Goal>,
    decks: BTreeMap<Entity, Vec<CardState>>,
    modifiers: BTreeMap<Entity, Vec<Modification>>,

    collected: BTreeMap<Card, i32>,
    log: MessageLog,
    rng: StdRng,
}

// TODO: impl
pub struct LevelTransition {
    next_level: i32,
    deck: Vec<Card>,
    collected: BTreeMap<Card, i32>,
    mutations: [Card; 3],
    rng: StdRng,
}

pub enum Game {
    Level(Level),
    Transition(LevelTransition),
    Victory,
}

#[derive(Eq, PartialEq)]
pub enum Glyph {
    Unknown,
    Remembered(Tile),
    Visible(Tile, Option<EntityType>),
}

impl Glyph {
    pub fn ch(&self) -> char {
        match self {
            &Glyph::Unknown => ' ',
            &Glyph::Remembered(t) | &Glyph::Visible(t, None) => t.render(),
            &Glyph::Visible(_, Some(e)) => e.render(),
        }
    }

    pub fn is_visible(&self) -> bool {
        match self {
            &Glyph::Unknown | &Glyph::Remembered(_) => false,
            &Glyph::Visible(_, _) => true,
        }
    }
}

impl Game {
    pub fn new(seed: u32) -> Game {
        Game::Level(Level::new(seed))
    }

    pub fn update(&mut self) {
        let update = match *self {
            Game::Level(ref level) => {
                if level.is_complete() {
                    Some(if level.level == 6 {
                        Game::Victory
                    } else {
                        let deck = level.decks.get(&PLAYER).iter().flat_map(|v| v.iter())
                            .map(|cs| cs.card).collect();
                        Game::Transition(LevelTransition {
                            next_level: level.level + 1,
                            deck: deck,
                            collected: level.collected.clone(),
                            // TODO: real card choices
                            mutations: [Card::Attack(1), Card::Attack(2), Card::Attack(3)],
                            rng: level.rng.clone(),
                        })
                    })
                } else {
                    None
                }
            }
            Game::Transition(ref trans) => {
                // TODO: actual give the player new cards
                Some(Game::Level(Level::next(trans.next_level, trans.deck.clone(), trans.rng.clone())))
            }
            Game::Victory => None,
        };
        if let Some(update) = update {
            *self = update;
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Position {
    fn step(self, dir: Direction) -> Position {
        let Position { x, y } = self;
        match dir {
            Direction::Up => Position { x, y: y - 1 },
            Direction::Down => Position { x, y: y + 1 },
            Direction::Left => Position { x: x - 1, y },
            Direction::Right => Position { x: x + 1, y },
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum Action {
    Move(Direction),
    Rest,
    Wait,
    Toggle(usize),
}

#[derive(Debug)]
pub struct CardView {
    pub card: Card,
    pub status: KnownCardStatus,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum KnownCardStatus {
    Active,
    Inactive,
    Discarded,
    PlayedOnSelf,
    PlayedOnVisible(EntityType, Position),
    PlayedOnOther(EntityType),
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum Event {
    Move {
        destination: Position,
        direction: Option<Direction>,
    },
    Recover,
    Wait,
    Disable(usize),
    Enable(usize),
    Attack {
        target: Entity,
        damage: i32,
        direction: Option<Direction>,
    },
    Defend {
        source: Entity,
        damage: i32,
        direction: Option<Direction>,
    },
    None
}

impl Level {
    pub fn view(&self, pos: Position) -> Glyph {
        let sq = self.get_sq(pos);
        match sq.visibility {
            Visibility::Unknown => Glyph::Unknown,
            Visibility::Remembered => Glyph::Remembered(sq.tile),
            Visibility::Visible => Glyph::Visible(sq.tile, sq.entity.and_then(|e| self.types.get(&e)).cloned()),
        }
    }

    pub fn message_log(&self) -> String {
        self.log.messages.join("\n")
    }

    pub fn step(&mut self, action: Action) -> bool {
        if self.is_complete() {
            return false;
        }
        let success = self.do_action(PLAYER, action);
        if success {
            self.update_visibility(true);
            let entities: Vec<_> = self.types.keys().cloned().collect();
            for e in entities {
                self.take_turn(e);
            }
            self.update_visibility(false);
            if self.is_complete() {
                self.log.messages.push(String::from("Exiting level!"));
                self.log.messages.push(String::from("Press [Space] to continue..."));
            }
        }
        success
    }

    // TODO: set goals / have enemy memory
    fn take_turn(&mut self, entity: Entity) {
        if entity == PLAYER {
            return;
        }
        let t = self.type_of(entity);
        if t == EntityType::UnknownThing { return; }
        let pos = match self.positions.get(&entity) {
            Some(&pos) => pos,
            None => { return; }
        };

        let target_pos;
        {
            let player_pos = match self.positions.get(&PLAYER) {
                Some(&pos) => pos,
                None => { return; }
            };

            let vis = self.get_sq(pos).visibility == Visibility::Visible;
            if vis {
                target_pos = player_pos;
                self.goals.insert(entity, Goal::Move(player_pos));
            } else if let Some(&goal) = self.goals.get(&entity) {
                match goal {
                    Goal::Move(pos) => { target_pos = pos; }
                    _ => { return; }
                }
            } else {
                return;
            }
        }

        let (hdir, mut hweight) = if pos.x > target_pos.x {
            (Direction::Left, pos.x - target_pos.x)
        } else if pos.x < target_pos.x {
            (Direction::Right, target_pos.x - pos.x)
        } else {
            (Direction::Left, 0)
        };
        let hpos = pos.step(hdir);
        if hweight != 0 && hpos != target_pos && !self.is_open(hpos) {
            hweight = 0;
        }

        let (vdir, mut vweight) = if pos.y > target_pos.y {
            (Direction::Up, pos.y - target_pos.y)
        } else if pos.y < target_pos.y {
            (Direction::Down, target_pos.y - pos.y)
        } else {
            (Direction::Up, 0)
        };
        let vpos = pos.step(vdir);
        if vweight != 0 && vpos != target_pos && !self.is_open(vpos) {
            vweight = 0;
        }

        let action = if hweight + vweight > 0 {
            if self.rng.gen_range(0, hweight + vweight) >= hweight {
                Action::Move(vdir)
            } else {
                Action::Move(hdir)
            }
        } else {
            Action::Rest
        };
        self.do_action(entity, action);
    }

    pub fn is_complete(&self) -> bool {
        self.positions.get(&PLAYER).map(|&pos| self.get_sq(pos).tile == Tile::Exit).unwrap_or(false)
    }

    // TODO: be able to return modifications
    pub fn player_deck(&self) -> Vec<CardView> {
        self.decks.get(&PLAYER).iter().flat_map(|v| v.iter())
            .map(|cs| CardView {
                card: cs.card,
                status: match cs.status {
                    CardStatus::Active => KnownCardStatus::Active,
                    CardStatus::Inactive => KnownCardStatus::Inactive,
                    CardStatus::Discarded => KnownCardStatus::Discarded,
                    CardStatus::PlayedOn(PLAYER) => KnownCardStatus::PlayedOnSelf,
                    CardStatus::PlayedOn(entity) => match self.types.get(&entity) {
                        None => KnownCardStatus::Discarded,
                        Some(&typ) => match self.positions.get(&entity) {
                            None => KnownCardStatus::PlayedOnOther(typ),
                            Some(&pos) => match self.get_sq(pos).visibility {
                                Visibility::Visible => KnownCardStatus::PlayedOnVisible(typ, pos),
                                _ => KnownCardStatus::PlayedOnOther(typ),
                            },
                        },
                    },
                }
            }).collect()
    }

    fn new(seed: u32) -> Level {
        let mut s = [0; 32];
        s[0] = seed as u8;
        s[1] = (seed>>8) as u8;
        s[2] = (seed>>16) as u8;
        s[3] = (seed>>24) as u8;
        // TODO
        // let deck = vec![Card::Attack(1), Card::Block];
        let deck = vec![
            Card::Attack(1), Card::Kill(1), Card::Strike, Card::Push,
            Card::Dodge, Card::Defend(2), Card::Block,
        ];
        Level::next(0, deck, StdRng::from_seed(s))
    }

    fn next(next_level: i32, player_deck: Vec<Card>, rng: StdRng) -> Level {
        let mut level = Level {
            level: next_level,
            last_id: PLAYER,
            map: [[Square {
                tile: Tile::Wall,
                entity: None,
                visibility: Visibility::Unknown,
            }; MAP_WIDTH]; MAP_HEIGHT],
            positions: BTreeMap::new(),

            types: BTreeMap::new(),
            goals: BTreeMap::new(),
            decks: BTreeMap::new(),
            modifiers: BTreeMap::new(),

            collected: BTreeMap::new(),
            log: MessageLog::new(),
            rng,
        };
        level.types.insert(PLAYER, EntityType::Player);
        level.decks.insert(PLAYER, player_deck.into_iter().map(
            |c| CardState { card: c, status: CardStatus::Active }
        ).collect());
        level.generate();
        level.update_visibility(true);
        level
    }

    fn get_sq(&self, pos: Position) -> Square {
        self.map
            .get(pos.y as usize)
            .and_then(|r| r.get(pos.x as usize))
            .cloned()
            .unwrap_or(Square {
                tile: Tile::Wall,
                entity: None,
                visibility: Visibility::Unknown,
            })
    }

    fn do_action(&mut self, entity: Entity, action: Action) -> bool {
        if self.types.get(&entity).is_none() {
            return false;
        }
        let pos = match self.positions.get(&entity) {
            Some(&pos) => pos,
            None => { return false; }
        };
        let event = match action {
            Action::Move(dir) => {
                let new_pos = pos.step(dir);
                let dst = self.get_sq(new_pos);
                if let Some(target) = dst.entity {
                    Event::Attack { target, damage: 1, direction: Some(dir) }
                } else if self.is_open(new_pos) {
                    Event::Move { destination: new_pos, direction: Some(dir) }
                } else {
                   return false;
                }
            },
            Action::Wait => Event::Wait,
            Action::Rest => Event::Recover,
            Action::Toggle(index) => match self.get_card_status(entity, index) {
                Some(CardStatus::Active) => Event::Disable(index),
                Some(CardStatus::Inactive) => Event::Enable(index),
                _ => { return false; }
            },
        };
        if entity == PLAYER && self.log.messages.last().map(|s| &**s) != Some("---") {
            self.log.messages.push(format!("---"));
        }
        self.process(entity, event);
        true
    }

    fn is_open(&self, pos: Position) -> bool {
        self.get_sq(pos).is_open()
    }

    // TODO: be more careful about overwriting existing entities?
    fn move_entity(&mut self, entity: Entity, pos: Position) -> bool {
        if !self.is_open(pos) {
            return false;
        }
        self.remove_entity(entity);
        self.map[pos.y as usize][pos.x as usize].entity = Some(entity);
        self.positions.insert(entity, pos);
        true
    }

    fn remove_entity(&mut self, entity: Entity) -> Option<Position> {
        if let Some(&old_pos) = self.positions.get(&entity) {
            self.map[old_pos.y as usize][old_pos.x as usize].entity = None;
            Some(old_pos)
        } else {
            None
        }
    }

    fn destroy_entity(&mut self, entity: Entity) {
        self.remove_entity(entity);
        self.types.remove(&entity);
        self.goals.remove(&entity);
        // TODO: handle removing modifiers
        self.decks.remove(&entity);
        self.modifiers.remove(&entity);
    }

    fn update_visibility(&mut self, clear: bool) {
        if clear {
            for y in 0..MAP_HEIGHT {
                for x in 0..MAP_WIDTH {
                    if self.map[y][x].visibility == Visibility::Visible {
                        self.map[y][x].visibility = Visibility::Remembered;
                    }
                }
            }
        }
        let ppos = match self.positions.get(&PLAYER) {
            None => { return; }
            Some(&pos) => pos,
        };
        for &(pdir, sdir) in &[
            (Direction::Up, Direction::Left),
            (Direction::Up, Direction::Right),
            (Direction::Down, Direction::Left),
            (Direction::Down, Direction::Right),
            (Direction::Left, Direction::Up),
            (Direction::Left, Direction::Down),
            (Direction::Right, Direction::Up),
            (Direction::Right, Direction::Down),
        ] {
            let mut visited = HashSet::new();
            let mut pending = Vec::new();
            pending.push(ppos);
            while let Some(pos) = pending.pop() {
                self.mark_visible(pos);
                let sq = self.get_sq(pos);
                if sq.tile == Tile::Wall || sq.tile == Tile::Door && sq.entity.is_none() {
                    continue;
                }
                let fwd = pos.step(pdir);
                if visited.insert(fwd) {
                    pending.push(fwd);
                }
                let diag = fwd.step(sdir);
                if visited.insert(diag) {
                    pending.push(diag);
                }
            }
        }
    }

    fn mark_visible(&mut self, pos: Position) {
        self.map[pos.y as usize][pos.x as usize].visibility = Visibility::Visible;
    }

    // TODO: level type selection
    fn generate(&mut self) {
        gen::generate_map(self, gen::LevelType::Test);
    }

    fn make_entity(&mut self, typ: EntityType) -> Entity {
        self.last_id.0 += 1;
        self.types.insert(self.last_id, typ);
        self.last_id
    }

    fn spawn_entity(&mut self, typ: EntityType, pos: Position) -> Option<Entity> {
        if !self.is_open(pos) {
            return None;
        }
        let entity = self.make_entity(typ);
        self.move_entity(entity, pos);
        Some(entity)
    }

    fn set_tile(&mut self, pos: Position, tile: Tile) {
        self.map[pos.y as usize][pos.x as usize].tile = tile;
    }

    fn type_of(&self, entity: Entity) -> EntityType {
        self.types.get(&entity).cloned().unwrap_or(EntityType::UnknownThing)
    }

    fn try_trigger(&mut self, entity: Entity, index: usize, event: &mut Event) -> bool {
        // TODO: figure out how to apply modifiers
        let c = self.decks.get(&entity)
            .and_then(|d| d.get(index))
            .cloned().unwrap();
        if c.status != CardStatus::Active { return false; }
        let triggered = match self.activate_card(entity, c.card, event) {
            CardOutcome::Continue => false,
            CardOutcome::Done => true,
            CardOutcome::Discard => {
                self.decks.get_mut(&entity)
                    .and_then(|d| d.get_mut(index))
                    .unwrap().status = CardStatus::Discarded;
                true
            }
            CardOutcome::Cancel => {
                *event = Event::None;
                true
            }
            CardOutcome::DiscardAndCancel => {
                self.decks.get_mut(&entity)
                    .and_then(|d| d.get_mut(index))
                    .unwrap().status = CardStatus::Discarded;
                *event = Event::None;
                true
            }
        };
        triggered
    }

    fn activate_card(&mut self, entity: Entity, card: Card, event: &mut Event) -> CardOutcome {
        // TODO: clean up logging crap
        let t = self.type_of(entity);
        match (card, event) {
            (Card::Attack(atk), Event::Attack { damage, .. }) => {
                self.log.messages.push(format!("({:?}'s {:?} card activated)", t, card));
                *damage += atk;
                CardOutcome::Done
            }
            (Card::Defend(def), Event::Defend { damage, .. }) =>  {
                if *damage >= def {
                    self.log.messages.push(format!("({:?}'s {:?} card activated and was discarded)", t, card));
                    *damage -= def;
                    CardOutcome::Discard
                } else {
                    CardOutcome::Continue
                }
            }
            (Card::Kill(atk), &mut Event::Attack { target, damage, direction }) => {
                let mod_count = self.modifiers.get_mut(&target).map(|m| m.len()).unwrap_or(0);
                let hand_size = self.decks.get_mut(&target).map(|d| {
                    d.iter_mut().filter(|c| c.status.in_hand()).count()
                }).unwrap_or(0);
                let health = (hand_size + mod_count) as i32;
                if damage + atk > health {
                    self.log.messages.push(format!("({:?}'s {:?} card activated)", t, card));
                    self.execute(target, Event::Defend {
                        source: entity,
                        damage: damage + atk,
                        direction,
                    });
                    CardOutcome::Cancel
                } else {
                    CardOutcome::Continue
                }
            }
            (Card::Strike, &mut Event::Move { destination, direction: Some(dir) }) => {
                let target_pos = destination.step(dir);
                if let Some(target) = self.get_sq(target_pos).entity {
                    self.log.messages.push(format!("({:?}'s {:?} card activated)", t, card));
                    self.execute(entity, Event::Move {
                        destination,
                        direction: Some(dir),
                    });
                    self.execute(entity, Event::Attack {
                        target,
                        damage: 1,
                        direction: Some(dir),
                    });
                    CardOutcome::Cancel
                } else {
                    CardOutcome::Continue
                }
            }
            (Card::Dodge, &mut Event::Defend { direction: Some(dir), .. }) => {
                let new_pos = match self.positions.get(&entity) {
                    Some(&pos) => pos,
                    None => { return CardOutcome::Continue; }
                } .step(dir);
                let dst = self.get_sq(new_pos);
                if dst.tile == Tile::Wall {
                    return CardOutcome::Continue;
                }
                self.log.messages.push(format!("({:?}'s {:?} card activated)", t, card));
                if let Some(target) = dst.entity {
                    self.execute(entity, Event::Attack {
                        target,
                        damage: 1,
                        direction: Some(dir),
                    });
                    CardOutcome::Discard
                } else {
                    self.execute(entity, Event::Move {
                        destination: new_pos,
                        direction: Some(dir),
                    });
                    CardOutcome::DiscardAndCancel
                }
            }
            (Card::Block, Event::Defend { .. }) =>  {
                self.log.messages.push(format!("({:?}'s {:?} card activated and was discarded)", t, card));
                CardOutcome::DiscardAndCancel
            }
            (Card::Push, &mut Event::Attack { target, direction: Some(dir), .. }) => {
                let target_pos = match self.positions.get(&target) {
                    Some(&pos) => pos,
                    None => { return CardOutcome::Continue; }
                };
                let new_pos = target_pos.step(dir);
                let dst = self.get_sq(new_pos);
                if dst.tile == Tile::Wall {
                    return CardOutcome::Continue;
                }
                self.log.messages.push(format!("({:?}'s {:?} card activated)", t, card));
                if let Some(secondary_target) = dst.entity {
                    self.execute(target, Event::Attack {
                        target: secondary_target,
                        damage: 1,
                        direction: Some(dir),
                    });
                } else {
                    self.execute(target, Event::Move {
                        destination: new_pos,
                        direction: Some(dir),
                    });
                    self.execute(entity, Event::Move {
                        destination: target_pos,
                        direction: Some(dir),
                    });
                }
                CardOutcome::Done
            }
            _ => CardOutcome::Continue,
        }
    }

    fn process(&mut self, entity: Entity, mut event: Event) -> Event {
        // TODO: include modifiers
        // TODO: optimize shuffling to already exclude non-active cards
        let shuffled = if let Some(d) = self.decks.get(&entity) {
            let mut v: Vec<usize> = (0..d.len()).collect();
            self.rng.shuffle(&mut v);
            v
        } else {
            Vec::new()
        };
        for index in shuffled {
            if self.try_trigger(entity, index, &mut event) {
                break;
            }
        }
        self.execute(entity, event);
        event
    }

    // TODO: (more) player action messages
    fn execute(&mut self, entity: Entity, event: Event) {
        // TODO: message only for visible attacks?
        let et = self.type_of(entity);
        match event {
            Event::Move { destination, .. } => {
                self.move_entity(entity, destination);
            }
            Event::Recover => {
                self.recover(entity);
            }
            Event::Wait => {}
            Event::Disable(index) => {
                self.set_card_status(entity, index, CardStatus::Inactive);
            }
            Event::Enable(index) => {
                self.set_card_status(entity, index, CardStatus::Active);
            }
            Event::Attack { target, damage, direction } => {
                let tt = self.type_of(target);
                self.log.messages.push(format!("{:?} attacks the {:?} for {} damage!", et, tt, damage));
                self.process(target, Event::Defend { source: entity, damage, direction });
            }
            Event::Defend { source, damage, .. } => {
                let st = self.type_of(source);
                self.log.messages.push(format!("{:?} hits the {:?} for {} damage!", st, et, damage));
                let fatal = self.take_damage(entity, damage);
                if fatal {
                    self.destroy_entity(entity);
                    self.log.messages.push(format!("{:?} kills the {:?}!", st, et));
                }
            }
            Event::None => {}
        }
    }

    fn set_card_status(&mut self, entity: Entity, index: usize, status: CardStatus) {
        self.decks.get_mut(&entity).and_then(|d| d.get_mut(index)).map(|cs| {
            cs.status = status;
        });
    }

    fn get_card_status(&self, entity: Entity, index: usize) -> Option<CardStatus> {
        self.decks.get(&entity).and_then(|d| d.get(index)).map(|cs| cs.status)
    }

    fn take_damage(&mut self, entity: Entity, damage: i32) -> bool {
        let t = self.type_of(entity);
        let (tmp1, tmp2) = (&mut Vec::new(), &mut Vec::new());
        let deck = self.decks.get_mut(&entity).unwrap_or(tmp1);
        let mods = self.modifiers.get_mut(&entity).unwrap_or(tmp2);
        let mut hand: Vec<_> = deck.iter_mut().filter(|c| c.status.in_hand()).collect();
        for _ in 0..damage {
            let hand_size = hand.len();
            let mod_count = mods.len();
            let option_count = hand_size + mod_count;
            if option_count == 0 {
                return true;
            }
            let selection = self.rng.gen_range(0, option_count);
            if selection < hand_size {
                {
                    let c = &mut hand[selection];
                    self.log.messages.push(format!("({:?}'s {:?} card was discarded by damage)", t, c.card));
                    c.status = CardStatus::Discarded;
                }
                hand.remove(selection);
            } else {
                mods.remove(option_count - hand_size);
            }
        }
        false
    }

    fn recover(&mut self, entity: Entity) {
        // let mut card = None;
        if let Some(deck) = self.decks.get_mut(&entity) {
            let mut discard: Vec<_> = deck.iter_mut().filter(|c| !c.status.in_hand()).collect();
            if let Some(sel) = self.rng.choose_mut(&mut discard) {
                // TODO: handle removing modifiers
                sel.status = CardStatus::Active;
                // card = Some(sel.card);
            }
        }
        // TODO: message?
        // if let Some(card) = card {
        //     let t = self.type_of(entity);
        //     self.log.messages.push(format!("({:?} draws its {:?} card)", t, card));
        // }
    }
}
