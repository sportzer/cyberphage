use super::*;

use rand::Rng;

const ROOM_XS: &[i32] = &[3, 9, 15, 21, 27, 33];
const ROOM_YS: &[i32] = &[3, 9, 15, 21];

// TODO: real level types
pub enum LevelType {
    Test,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum RoomStyle {
    Square,
    Rounded,
    Diamond,
    Hallway,
    Plus,
    RoundedPlus,
    Corner,
    Random,
    ExitSquare,
}

pub fn generate_map(level: &mut Level, typ: LevelType) {
    match typ {
        LevelType::Test => { test_map(level); }
    };
}

fn test_map(level: &mut Level) {
    for &y in ROOM_YS {
        for &x in ROOM_XS {
            let style = if x == 3 && y == 3 {
                RoomStyle::Square
            } else if x == 33 && y == 21 {
                RoomStyle::ExitSquare
            } else {
                RoomStyle::Random
            };
            place_room(level, Position { x, y }, style);
            // TODO: real passageway handling
            if x > 3 {
                level.set_tile(Position { x: x - 3, y }, Tile::Door);
            }
            if y > 3 {
                level.set_tile(Position { x, y: y - 3 }, Tile::Door);
            }
        }
    }
    level.set_tile(Position { x: 33, y: 21 }, Tile::Exit);
    level.move_entity(PLAYER, Position { x: 3, y: 3 });
    for _ in 0..64 {
        let pos = Position {
            x: level.rng.gen_range(1, 35),
            y: level.rng.gen_range(1, 23),
        };
        if level.is_open(pos) && level.get_sq(pos).tile != Tile::Door {
            place_entity(level, pos, EntityType::UnknownThing);
        }
    }
    level.log.messages.push(String::from("You are in some sort of server. It seems pretty quiet here."));
}

fn place_entity(level: &mut Level, pos: Position, t: EntityType) {
    match t {
        EntityType::Player => { /* TODO: handle this? */ }
        EntityType::Defender => {
            let defender = level.spawn_entity(EntityType::Defender, pos);
            defender.map(|d| level.decks.insert(d, vec![
                CardState{ card: Card::Defend(2), status: CardStatus::Active },
                CardState{ card: Card::Block, status: CardStatus::Active },
                CardState{ card: Card::Push, status: CardStatus::Active },
            ]));
        }
        EntityType::Hunter => {
            let hunter = level.spawn_entity(EntityType::Hunter, pos);
            hunter.map(|h| level.decks.insert(h, vec![
                CardState{ card: Card::Strike, status: CardStatus::Active },
                CardState{ card: Card::Dodge, status: CardStatus::Active },
            ]));
        }
        EntityType::Reaper => {
            let reaper = level.spawn_entity(EntityType::Reaper, pos);
            reaper.map(|r| level.decks.insert(r, vec![
                CardState{ card: Card::Kill(1), status: CardStatus::Active },
                CardState{ card: Card::Attack(1), status: CardStatus::Active },
            ]));
        }
        EntityType::UnknownThing => {
            let t = *level.rng.choose(&[
                EntityType::Defender,
                EntityType::Hunter,
                EntityType::Reaper,
            ]).unwrap();
            place_entity(level, pos, t);
        }
    }
}

// TODO: optional hallways that get pruned post level generation
fn place_room(level: &mut Level, pos: Position, style: RoomStyle) {
    let pattern = match style {
        RoomStyle::Random => {
            let style = *level.rng.choose(&[
                RoomStyle::Square,
                RoomStyle::Rounded,
                RoomStyle::Diamond,
                RoomStyle::Hallway,
                RoomStyle::Plus,
                RoomStyle::RoundedPlus,
                RoomStyle::Corner,
            ]).unwrap();
            place_room(level, pos, style);
            return;
        }
        RoomStyle::Square => b"\
.....\
.....\
.....\
.....\
.....",
        RoomStyle::Rounded => b"\
#...#\
.....\
.....\
.....\
#...#",
        RoomStyle::Diamond => b"\
##.##\
#...#\
.....\
#...#\
##.##",
        RoomStyle::Hallway => b"\
##.##\
##.##\
.....\
##.##\
##.##",
        RoomStyle::Plus => b"\
.....\
..#..\
.###.\
..#..\
.....",
        RoomStyle::RoundedPlus => b"\
#...#\
..#..\
.###.\
..#..\
#...#",
        RoomStyle::Corner => b"\
##..#\
##..#\
.....\
....#\
##.##",
        RoomStyle::ExitSquare => b"\
.....\
.....\
..>..\
.....\
.....",
    };

    let rotation = level.rng.gen_range(0, 8);
    for y in 0..5 {
        for x in 0..5 {
            let index = [
                y*5 + x, y*5 + (4 - x), (4 - y)*5 + x, (4 - y)*5 + (4 - x),
                x*5 + y, x*5 + (4 - y), (4 - x)*5 + y, (4 - x)*5 + (4 - y),
            ][rotation] as usize;
            let tile = match pattern[index] {
                b'.' => Tile::Floor,
                b'+' => Tile::Door,
                b'>' => Tile::Exit,
                _ => Tile::Wall,
            };
            level.set_tile(Position { x: pos.x-2+x, y: pos.y-2+y }, tile);
        }
    }
}
