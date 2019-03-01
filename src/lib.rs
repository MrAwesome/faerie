use std::collections::HashMap;
use std::collections::HashSet;

type RoomPassFunc = Option<Box<dyn FnMut(&mut User) -> Result<Option<String>, String>>>;
type RoomName = String;
type UserName = String;
type PathName = String;

#[derive(Debug)]
pub enum UserType {
    Civilian,
    Viking,
    ElfLord,
}

// Another way of bypassing the unfortunate box ownership issues for calling Fns stored in them.
//pub trait FnBox {
//    fn call_box(self: Box<Self>, user: &mut User) -> Result<Option<String>, String>;
//}
//
//impl<F: FnMut(&mut User) -> Result<Option<String>, String>> FnBox for F {
//    fn call_box(mut self: Box<F>, user: &mut User) -> Result<Option<String>, String> {
//        (*self)(user)
//    }
//}
//
//type Job = Box<FnBox + 'static>;

fn mk_callback<F: 'static>(f: F) -> RoomPassFunc
where
    F: FnMut(&mut User) -> Result<Option<String>, String>,
{
    Some(Box::new(f))
}

#[derive(Debug)]
pub struct BasicAttributes {
    pub hp: i32,
    pub mp: i32,
}

impl BasicAttributes {
    fn default(user_type: &UserType) -> BasicAttributes {
        match user_type {
            UserType::Civilian => BasicAttributes { hp: 20, mp: 7 },
            UserType::Viking => BasicAttributes { hp: 220, mp: 9 },
            UserType::ElfLord => BasicAttributes { hp: 80, mp: 28 },
        }
    }
}

#[derive(Debug)]
pub enum SpecialAttributes {
    Civilian { needlessly_chatter: usize },
    Viking { brutish_swing: u8 },
    ElfLord { fuck_infusion: u8 },
}

impl SpecialAttributes {
    fn default(user_type: &UserType) -> SpecialAttributes {
        match user_type {
            UserType::Civilian => SpecialAttributes::Civilian {
                needlessly_chatter: 20,
            },
            UserType::Viking => SpecialAttributes::Viking { brutish_swing: 2 },
            UserType::ElfLord => SpecialAttributes::ElfLord { fuck_infusion: 3 },
        }
    }
}

struct SuccessfulMove {
    pub messages: Vec<String>,
}

struct UnsuccessfulMove {
    pub message: String,
}

#[derive(Debug)]
pub struct User {
    pub name: UserName,
    pub room_name: RoomName,
    pub basic_attributes: BasicAttributes,
    pub special_attributes: SpecialAttributes,
}

impl User {
    fn new(name: UserName, starting_room_name: RoomName, user_type: UserType) -> User {
        assert!(!name.is_empty(), "Empty user names are not allowed!");
        let basic_attributes = BasicAttributes::default(&user_type);
        let special_attributes = SpecialAttributes::default(&user_type);
        User {
            name,
            room_name: starting_room_name,
            basic_attributes,
            special_attributes,
        }
    }
}

pub struct GameMap {
    rooms: RoomCollection,
    users: UserCollection,
}

struct RoomCollection {
    rooms: HashMap<RoomName, Room>,
}

impl RoomCollection {
    fn new() -> RoomCollection {
        RoomCollection {
            rooms: HashMap::new(),
        }
    }

    fn get_room(&self, room_name: &RoomName) -> &Room {
        let room = self
            .rooms
            .get(room_name)
            .expect(&format!("Failed to find room named {}!", room_name));
        room
    }

    fn get_room_mut(&mut self, room_name: &RoomName) -> &mut Room {
        let room = self.rooms.get_mut(room_name).expect(&format!(
            "Failed to find room named {} for mutation!",
            room_name
        ));
        room
    }

    fn check_room_exists(&self, room_name: &RoomName) {
        assert!(
            self.rooms.contains_key(room_name),
            format!("No room named {} exists!", room_name)
        );
    }
}

struct UserCollection {
    users: HashMap<UserName, User>,
}

impl UserCollection {
    fn new() -> UserCollection {
        UserCollection {
            users: HashMap::new(),
        }
    }

    fn get_user(&self, user_name: &UserName) -> &User {
        let user = self
            .users
            .get(user_name)
            .expect(&format!("Failed to find user named {}!", user_name));
        user
    }

    fn get_user_mut(&mut self, user_name: &UserName) -> &mut User {
        let user = self.users.get_mut(user_name).expect(&format!(
            "Failed to find user named {} for mutation!",
            user_name
        ));
        user
    }

    fn check_user_exists(&self, user_name: &UserName) {
        assert!(
            self.users.contains_key(user_name),
            format!("No user named {} exists!", user_name)
        );
    }
}

impl GameMap {
    pub fn new() -> GameMap {
        GameMap {
            rooms: RoomCollection::new(),
            users: UserCollection::new(),
        }
    }

    pub fn print_debug_map(&self) {
        println!("Rooms:");
        for (_roomname, room) in &self.rooms.rooms {
            println!("  {}: ", room.name);
            println!("    paths:");
            for (_pathname, path) in &room.exits {
                println!("      * {} -> {}", path.path_name, path.target_room_name);
            }
            println!("    users:");
            for username in &room.users {
                println!("       @ {}", username);
            }
            println!();
        }

        println!();
        println!("Users:");

        for (_username, user) in &self.users.users {
            println!(" {}", user.name);
        }
    }

    pub fn print_room(&self, username: &UserName) {
        let user = self.users.get_user(username);
        let room = self.rooms.get_room(&user.room_name);
        let desc = room.description.clone();

        println!("{}", &user.room_name);
        println!("  {}", &desc);
        println!();
        println!("Exits: ");
        for (_path_name, exit) in &room.exits {
            println!("* {}", exit.path_name);
        }
    }

    pub fn create_empty_room(&mut self, name: &RoomName, desc: String) {
        let room = Room::new(name.clone(), desc);
        // TODO: make this an action on the roomcollection directly?
        self.rooms.rooms.insert(name.clone(), room);
    }

    pub fn add_path(
        &mut self,
        source_room_name: &RoomName,
        target_room_name: &RoomName,
        direction: Direction,
    ) {
        self.add_path_impl(source_room_name, target_room_name, direction.clone());

        if let Some(d) = Direction::get_reverse(direction.clone()) {
            self.add_path_impl(target_room_name, source_room_name, d);
        }
    }

    fn add_path_impl(
        &mut self,
        source_room_name: &RoomName,
        target_room_name: &RoomName,
        direction: Direction,
    ) {
        self.rooms.check_room_exists(target_room_name);
        let source_room = self.rooms.get_room_mut(source_room_name);
        let path_name = Direction::get_path_name(direction.clone());
        source_room.add_exit(target_room_name, &path_name);
    }

    fn get_user_location(&self, user_name: &UserName) -> RoomName {
        let user = self.users.get_user(user_name);
        self.rooms.check_room_exists(&user.room_name);
        user.room_name.clone()
    }

    // TODO: dedup this code, make path types work both directions if requested?
    //    pub fn add_path_special(
    //        &mut self,
    //        old_room_name: &RoomName,
    //        target_room_name: &RoomName,
    //        direction: Direction,
    //        path_type: PathType,
    //    ) {
    //        let old_room = self.rooms.get_mut(old_room_name).unwrap();
    //        let path_name = Direction::get_path_name(direction.clone());
    //        old_room.add_exit_special(target_room_name, &path_name, path_type);
    //
    //        if let Some(d) = Direction::get_reverse(direction.clone()) {
    //            let target_room = self.rooms.get_mut(target_room_name).unwrap();
    //            let path_name = Direction::get_path_name(d);
    //            target_room.add_exit_special(old_room_name, &path_name, path_type);
    //        }
    //    }

    pub fn create_user_in_room(
        &mut self,
        user_name: &UserName,
        room_name: &RoomName,
        user_type: UserType,
    ) {
        let user = User::new(user_name.clone(), room_name.clone(), user_type);
        self.users.users.insert(user_name.clone(), user);

        let room = self.rooms.get_room_mut(room_name);
        room.add_user(user_name.clone());
    }

    pub fn attempt_move(&mut self, user_name: &UserName, path_name: &PathName) {
        let move_succ = self.attempt_move_impl(user_name, path_name);
        match move_succ {
            Ok(succ) => {
                for m in succ.messages {
                    println!("{}", m);
                }
                self.print_room(user_name);
            }
            Err(unsucc) => {
                println!("{}", unsucc.message);
            }
        }
    }

    // TODO: Make a more general processing function of which movement is only one part, call
    // this function from there
    fn attempt_move_impl(
        &mut self,
        user_name: &UserName,
        possible_path_name: &String,
    ) -> Result<SuccessfulMove, UnsuccessfulMove> {
        let possible_path_name = Path::match_basic_aliases(possible_path_name.clone());

        if possible_path_name == "" {
            return Err(UnsuccessfulMove {
                message: "".to_string(),
            });
        }

        let mut messages = vec![];

        let room_name = self.get_user_location(user_name);
        let room = self.rooms.get_room_mut(&room_name);

        // TODO: make a pathcollection on each room, make a convenience function which does this?
        let path = match room.exits.get_mut(&possible_path_name) {
            Some(path) => Ok(path),
            None => Err(UnsuccessfulMove {
                message: format!(
                    "What? There's no direction {} from {}.",
                    possible_path_name, room_name
                ),
            }),
        }?;

        // TODO: make this a method somewhere
        if let Some(ref mut exit_lambda) = path.exit_cond {
            let user = self.users.get_user_mut(user_name);
            let exit_lambda_result = exit_lambda(user);
            match exit_lambda_result {
                Ok(message) => {
                    if let Some(m) = message {
                        messages.push(m);
                    }
                    Ok(true)
                }
                Err(message) => Err(UnsuccessfulMove { message }),
            }?;
        }

        let target_room_name = path.target_room_name.clone();
        let target_room = self.rooms.get_room_mut(&target_room_name);
        target_room.users.insert(user_name.clone());

        let user = self.users.get_user_mut(user_name);
        user.room_name = target_room_name.clone();

        let room = self.rooms.get_room_mut(&room_name);
        room.users.take(user_name);

        Ok(SuccessfulMove { messages: messages })
    }
}

pub struct Room {
    name: RoomName,
    description: String,
    exits: HashMap<PathName, Path>,
    users: HashSet<UserName>,
}

impl Room {
    fn new(name: RoomName, description: String) -> Room {
        assert!(!name.is_empty(), "Empty room names are not allowed!");
        assert!(
            !description.is_empty(),
            "Empty room descriptions are not allowed!"
        );
        Room {
            name,
            description,
            exits: HashMap::new(),
            users: HashSet::new(),
        }
    }

    pub fn add_exit(&mut self, target_room_name: &RoomName, path_name: &PathName) {
        let path = Path::new(
            target_room_name.clone(),
            path_name.clone(),
            PathType::Normal,
        );
        self.exits.insert(path_name.clone(), path);
    }

    pub fn add_exit_special(
        &mut self,
        target_room_name: &RoomName,
        path_name: &PathName,
        path_type: PathType,
    ) {
        let path = Path::new(target_room_name.clone(), path_name.clone(), path_type);
        self.exits.insert(path_name.clone(), path);
    }

    pub fn add_user(&mut self, name: UserName) {
        self.users.insert(name);
    }
}

#[derive(Debug, Clone)]
pub enum Direction {
    North,
    East,
    South,
    West,
    NorthEast,
    SouthEast,
    SouthWest,
    NorthWest,
    CustomOneWay(PathName),
    Custom(PathName, PathName),
}

impl Direction {
    fn get_path_name(dir: Direction) -> PathName {
        match dir {
            Direction::North => "north".to_string(),
            Direction::South => "south".to_string(),
            Direction::East => "east".to_string(),
            Direction::West => "west".to_string(),
            Direction::NorthEast => "northeast".to_string(),
            Direction::SouthEast => "southeast".to_string(),
            Direction::SouthWest => "southwest".to_string(),
            Direction::NorthWest => "northwest".to_string(),
            Direction::CustomOneWay(start) => start,
            Direction::Custom(start, _end) => start,
        }
    }

    fn get_reverse(dir: Direction) -> Option<Direction> {
        match dir {
            Direction::North => Some(Direction::South),
            Direction::South => Some(Direction::North),
            Direction::East => Some(Direction::West),
            Direction::West => Some(Direction::East),
            Direction::NorthEast => Some(Direction::SouthWest),
            Direction::SouthEast => Some(Direction::NorthWest),
            Direction::SouthWest => Some(Direction::NorthEast),
            Direction::NorthWest => Some(Direction::SouthEast),
            Direction::CustomOneWay(_start) => None,
            Direction::Custom(start, end) => Some(Direction::Custom(end, start)),
        }
    }
}

struct Path {
    target_room_name: RoomName,
    path_name: PathName,
    exit_cond: RoomPassFunc,
}

impl Path {}

pub enum PathType {
    Normal,
    Painful,
    Custom(RoomPassFunc),
}

impl Path {
    fn new(target_room_name: RoomName, path_name: PathName, path_type: PathType) -> Path {
        assert!(!path_name.is_empty(), "Empty path names are not allowed!");
        match path_type {
            PathType::Normal => Path {
                target_room_name,
                path_name,
                exit_cond: None,
            },
            PathType::Painful => Path::new_painful(target_room_name, path_name),
            PathType::Custom(exit_cond) => Path {
                target_room_name,
                path_name,
                exit_cond,
            },
        }
    }

    fn new_painful(target_room_name: RoomName, path_name: PathName) -> Path {
        let clos = |user: &mut User| {
            user.basic_attributes.hp -= 1;

            Ok(Some("You passed through, but it hurt you.".to_string()))
        };
        let exit_cond = mk_callback(clos);

        Path {
            target_room_name,
            path_name,
            exit_cond,
        }
    }

    fn match_basic_aliases(s: String) -> String {
        match s.as_ref() {
            "n" => "north".to_string(),
            "s" => "south".to_string(),
            "w" => "west".to_string(),
            "e" => "east".to_string(),
            "ne" => "northeast".to_string(),
            "se" => "southeast".to_string(),
            "nw" => "northwest".to_string(),
            "sw" => "southwest".to_string(),
            _ => s,
        }
    }
}

mod tests {
    #[test]
    fn move_norf() {
        use super::*;
        let mut map = GameMap::new();

        let room1name = "room1".to_string();
        let room2name = "room2".to_string();
        let room3name = "room3".to_string();
        map.create_empty_room(&room1name, "yeet".to_string());
        map.create_empty_room(&room2name, "dang".to_string());
        map.create_empty_room(&room3name, "where am i".to_string());

        map.add_path(&room1name, &room2name, Direction::North);
        map.add_path(&room2name, &room3name, Direction::North);

        let user1name = "user1".to_string();
        map.create_user_in_room(&user1name, &room1name, UserType::Civilian);

        map.attempt_move(&user1name, &"north".to_string());
        map.attempt_move(&user1name, &"n".to_string());

        let user = map.users.get_user(&user1name);
        assert_eq!(user.room_name, room3name);

        // TODO: make these use correct calls
        let room1 = map.rooms.get_room(&room1name);
        let room2 = map.rooms.get_room(&room2name);
        let room3 = map.rooms.get_room(&room3name);
        let is_user_in_room3 = room3.users.contains(&user1name);
        assert_eq!(is_user_in_room3, true);

        let is_user_in_room1 = room1.users.contains(&user1name);
        let is_user_in_room2 = room2.users.contains(&user1name);
        assert_eq!(is_user_in_room1, false);
        assert_eq!(is_user_in_room2, false);
    }

    #[test]
    fn move_up_and_back() {
        use super::*;
        let mut map = GameMap::new();

        let room1name = "room1".to_string();
        let room2name = "room2".to_string();
        map.create_empty_room(&room1name, "help".to_string());
        map.create_empty_room(&room2name, "ffffffffff".to_string());

        map.add_path(&room1name, &room2name, Direction::North);

        let user1name = "user1".to_string();
        map.create_user_in_room(&user1name, &room1name, UserType::Civilian);

        map.attempt_move(&user1name, &"n".to_string());
        map.attempt_move(&user1name, &"s".to_string());

        let user = map.users.get_user(&user1name);
        assert_eq!(user.room_name, room1name);

        let room1 = map.rooms.get_room(&room1name);
        let room2 = map.rooms.get_room(&room2name);
        let is_user_in_room1 = room1.users.contains(&user1name);
        assert_eq!(is_user_in_room1, true);

        let is_user_in_room2 = room2.users.contains(&user1name);
        assert_eq!(is_user_in_room2, false);
    }

    #[test]
    fn move_invalid_direction() {
        use super::*;
        let mut map = GameMap::new();

        let room1name = "room1".to_string();
        map.create_empty_room(&room1name, "blah".to_string());

        let user1name = "user1".to_string();
        map.create_user_in_room(&user1name, &room1name, UserType::Civilian);

        let user = map.users.get_user(&user1name);
        assert_eq!(user.room_name, room1name);

        let room1 = map.rooms.get_room(&room1name);
        let is_user_in_room1 = room1.users.contains(&user1name);
        assert_eq!(is_user_in_room1, true);

        let res = map.attempt_move_impl(&user1name, &"northhampton".to_string());
        assert_eq!(
            res.is_ok(),
            false,
            "Move somehow succeeded in moving a fake direction."
        );
    }

    #[test]
    #[should_panic(expected = "Empty room names are not allowed!")]
    fn attempt_empty_room_name_creation() {
        use super::*;
        let mut map = GameMap::new();

        map.create_empty_room(&"".to_string(), "The Land Of Dook".to_string());
    }

    #[test]
    #[should_panic(expected = "Empty room descriptions are not allowed!")]
    fn attempt_empty_room_description_creation() {
        use super::*;
        let mut map = GameMap::new();

        map.create_empty_room(&"Mang0".to_string(), "".to_string());
    }

    #[test]
    #[should_panic(expected = "Empty user names are not allowed!")]
    fn attempt_empty_user_name_creation() {
        use super::*;
        let mut map = GameMap::new();

        map.create_empty_room(
            &"Dooklandia".to_string(),
            "Big ol' dook in front of you".to_string(),
        );

        map.create_user_in_room(
            &"".to_string(),
            &"Dooklandia".to_string(),
            UserType::Civilian,
        );
    }

    #[test]
    #[should_panic(expected = "Failed to find room named FAKEFRIENDS for mutation!")]
    fn attempt_incorrect_room_user_creation() {
        use super::*;
        let mut map = GameMap::new();

        let room1name = "Dooklandia".to_string();

        map.create_empty_room(&room1name, "Big ol' dook in front of you".to_string());

        map.create_user_in_room(
            &"Freddie".to_string(),
            &"FAKEFRIENDS".to_string(),
            UserType::Civilian,
        );
    }

    #[test]
    #[should_panic(expected = "Empty path names are not allowed!")]
    fn attempt_empty_oneway_path_name_creation() {
        use super::*;
        let mut map = GameMap::new();

        let room1name = "room1".to_string();
        let room2name = "room2".to_string();

        map.create_empty_room(&room1name, "The Land Of Dook".to_string());
        map.create_empty_room(&room2name, "The Land Of Dook, 2".to_string());

        map.add_path(
            &room1name,
            &room2name,
            Direction::CustomOneWay("".to_string()),
        );
    }

    #[test]
    #[should_panic(expected = "Empty path names are not allowed!")]
    fn attempt_empty_twoway_path_name_creation() {
        use super::*;
        let mut map = GameMap::new();

        let room1name = "room1".to_string();
        let room2name = "room2".to_string();

        map.create_empty_room(&room1name, "The Land Of Dook".to_string());
        map.create_empty_room(&room2name, "The Land Of Dook, 2".to_string());

        map.add_path(
            &room1name,
            &room2name,
            Direction::Custom("mkay".to_string(), "".to_string()),
        );
    }

    #[test]
    #[should_panic(expected = "No room named FAKENEWS exists!")]
    fn attempt_path_to_invalid_room() {
        use super::*;
        let mut map = GameMap::new();

        let room1name = "room1".to_string();
        let room2name = "room2".to_string();

        map.create_empty_room(&room1name, "The Land Of Dook".to_string());
        map.create_empty_room(&room2name, "The Land Of Dook, 2".to_string());

        map.add_path(
            &room1name,
            &"FAKENEWS".to_string(),
            Direction::Custom("mkay".to_string(), "jkll".to_string()),
        );
    }

    #[test]
    #[should_panic(expected = "Failed to find room named FAKENEWS for mutation!")]
    fn attempt_path_from_invalid_room() {
        use super::*;
        let mut map = GameMap::new();

        let room1name = "room1".to_string();
        let room2name = "room2".to_string();

        map.create_empty_room(&room1name, "The Land Of Dook".to_string());
        map.create_empty_room(&room2name, "The Land Of Dook, 2".to_string());

        map.add_path(
            &"FAKENEWS".to_string(),
            &room2name,
            Direction::Custom("mkay".to_string(), "jkll".to_string()),
        );
    }

    // TODO: make sure paths are going from correct room to correct room on creation

    //    #[test]
    //    fn move_painful() {
    //        use super::*;
    //        let mut map = GameMap::new();
    //
    //        let room1name = "room1".to_string();
    //        let room2name = "room2".to_string();
    //        map.create_empty_room(&room1name);
    //        map.create_empty_room(&room2name);
    //
    //        map.add_path_special(&room1name, &room2name, Direction::North, PathType::Painful);
    //
    //        let user1name = "user1".to_string();
    //        map.create_user_in_room(&user1name, &room1name, UserType::Civilian);
    //
    //        let user1hp_before = map.users.get(&user1name).unwrap().basic_attributes.hp;
    //        map.attempt_move(&user1name, Direction::North);
    //        let user1hp_after = map.users.get(&user1name).unwrap().basic_attributes.hp;
    //
    //        assert_eq!(user1hp_before - 1, user1hp_after);
    //    }
}
