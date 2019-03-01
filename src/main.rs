use std::io;
use std::io::Write;

use faerie::{Direction, GameMap};

fn main() {
    let user1name = "glenn".to_string();
    let mut map = create_basic_map(user1name.clone());

    loop {
        // TODO: move all of this into dedicated lib functionality
        // TODO: have commands, which override directions
        println!();
        println!();
        print!(">>> ");
        io::stdout().flush().unwrap();
        let mut buf = String::new();
        io::stdin().read_line(&mut buf).unwrap();
        let lastchar = buf.pop();

        // You don't want this forever, but for now breaking with Ctrl-D is nice
        if lastchar.is_none() {
            break;
        }

        map.attempt_move(&user1name, &buf);
    }
}

fn create_basic_map(user1name: String) -> GameMap {
    let mut map = GameMap::new();

    let room1name = "Starting Point".to_string();
    let room2name = "North of Start".to_string();
    let room3name = "More North".to_string();
    let room4name = "Over West".to_string();
    let room5name = "The Odd Little Woods".to_string();
    map.create_empty_room(
        &room1name,
        "This seems like a nice place to start an adventure.".to_string(),
    );
    map.create_empty_room(
        &room2name,
        "You're on a grassy plain. It's windy, but not uncomfortably so.".to_string(),
    );
    map.create_empty_room(
        &room3name,
        "A large swamp spreads out before you. It smells like sulfur farts.".to_string(),
    );
    map.create_empty_room(
        &room4name,
        "The secret glen doesn't seem all that secret, but the amber sunlight filtering "
            .to_string()
            + "through the trees really speaks to your soul. Maybe you should take a nap here.",
    );
    map.create_empty_room(
        &room5name,
        "Ah, the real secret of this little township of the woods.".to_string(),
    );

    map.add_path(&room1name, &room2name, Direction::North);
    map.add_path(&room2name, &room3name, Direction::North);
    map.add_path(&room3name, &room4name, Direction::West);
    map.add_path(&room4name, &room5name, Direction::NorthWest);

    map.create_basic_user_in_room(&user1name, &room1name);
    map
}
