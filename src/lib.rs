use std::collections::HashMap;

mod user;
use user::{User, UserType};

mod type_aliases;
use type_aliases::{PathName, RoomName, UserName};

mod lambda;
use lambda::{ActionFailure, ActionSuccess};

pub mod room;
use room::{Direction, Path, Room};

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

struct RoomStore {
    rooms: HashMap<RoomName, Room>,
}

impl RoomStore {
    fn new() -> RoomStore {
        RoomStore {
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

struct UserStore {
    users: HashMap<UserName, User>,
}

impl UserStore {
    fn new() -> UserStore {
        UserStore {
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

pub struct GameState {
    rooms: RoomStore,
    users: UserStore,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            rooms: RoomStore::new(),
            users: UserStore::new(),
        }
    }

    pub fn print_debug_map(&self) {
        println!("Rooms:");
        for (_roomname, room) in &self.rooms.rooms {
            println!("  {}: ", room.name);
            println!("    paths:");
            for (_pathname, path) in &room.paths {
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
        println!("paths: ");
        for (_path_name, exit) in &room.paths {
            println!("* {}", exit.path_name);
        }
    }

    pub fn create_room(&mut self, name: &RoomName, desc: String) {
        let room = Room::new(name.clone(), desc);
        // TODO: make this an action on the roomcollection directly?
        self.rooms.rooms.insert(name.clone(), room);
    }

    pub fn create_room_from(
        &mut self,
        this_name: &RoomName,
        this_desc: String,
        other_room_name: &RoomName,
        direction: Direction,
    ) {
        self.create_room(this_name, this_desc);
        self.add_path(other_room_name, this_name, direction);
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
        source_room.add_path(target_room_name, &path_name);
    }

    fn get_user_location(&self, user_name: &UserName) -> RoomName {
        let user = self.users.get_user(user_name);
        self.rooms.check_room_exists(&user.room_name);
        user.room_name.clone()
    }

    pub fn create_user_in_room(
        &mut self,
        user_name: &UserName,
        room_name: &RoomName,
        user_type: UserType,
    ) {
        let user = User::new(user_name.clone(), room_name.clone(), user_type);
        self.users.users.insert(user_name.clone(), user);

        let room = self.rooms.get_room_mut(room_name);
        room.users.insert(user_name.clone());
    }

    pub fn create_basic_user_in_room(&mut self, user_name: &UserName, room_name: &RoomName) {
        let user_type = UserType::Civilian;
        let user = User::new(user_name.clone(), room_name.clone(), user_type);
        self.users.users.insert(user_name.clone(), user);

        let room = self.rooms.get_room_mut(room_name);
        room.users.insert(user_name.clone());
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
                // TODO: fix
                for m in unsucc.messages {
                    println!("{}", m);
                }
            }
        }
    }

    // TODO: Make a more general processing function of which movement is only one part, call
    // this function from there
    fn attempt_move_impl(
        &mut self,
        user_name: &UserName,
        possible_path_name: &String,
    ) -> Result<ActionSuccess, ActionFailure> {
        let possible_path_name = Path::match_basic_aliases(possible_path_name.clone());

        if possible_path_name == "" {
            return Err(ActionFailure { messages: vec![] });
        }

        let mut messages = vec![];

        let room_name = self.get_user_location(user_name);
        let room = self.rooms.get_room_mut(&room_name);

        // TODO: make a pathcollection on each room, make a convenience function which does this?
        let path = match room.paths.get_mut(&possible_path_name) {
            Some(p) => Ok(p),
            None => Err(ActionFailure {
                messages: vec![format!(
                    // TODO: better message
                    "What? There's no direction {} from {}.",
                    possible_path_name, room_name
                )],
            }),
        }?;

        // TODO: make this a method somewhere
        // TODO: pass actionsuccess/failure messages through as a single thing?
        if let Some(ref mut exit_lambda) = path.exit_cond {
            let user = self.users.get_user_mut(user_name);
            let exit_lambda_result = exit_lambda(user);
            match exit_lambda_result {
                Ok(mut action_succ) => {
                    messages.append(&mut action_succ.messages);
                    Ok(true)
                }
                Err(action_fail) => Err(action_fail),
            }?;
        }

        let target_room_name = path.target_room_name.clone();
        let target_room = self.rooms.get_room_mut(&target_room_name);
        target_room.users.insert(user_name.clone());

        let user = self.users.get_user_mut(user_name);
        user.room_name = target_room_name.clone();

        let room = self.rooms.get_room_mut(&room_name);
        room.users.take(user_name);

        Ok(ActionSuccess { messages: messages })
    }
}

mod tests {
    #[test]
    fn move_norf() {
        use super::*;
        let mut game_state = GameState::new();

        let room1name = "room1".to_string();
        let room2name = "room2".to_string();
        let room3name = "room3".to_string();
        game_state.create_room(&room1name, "yeet".to_string());
        game_state.create_room(&room2name, "dang".to_string());
        game_state.create_room(&room3name, "where am i".to_string());

        game_state.add_path(&room1name, &room2name, Direction::North);
        game_state.add_path(&room2name, &room3name, Direction::North);

        let user1name = "user1".to_string();
        game_state.create_user_in_room(&user1name, &room1name, UserType::Civilian);

        game_state.attempt_move(&user1name, &"north".to_string());
        game_state.attempt_move(&user1name, &"n".to_string());

        let user = game_state.users.get_user(&user1name);
        assert_eq!(user.room_name, room3name);

        let room1 = game_state.rooms.get_room(&room1name);
        let room2 = game_state.rooms.get_room(&room2name);
        let room3 = game_state.rooms.get_room(&room3name);
        let is_user_in_room3 = room3.users.contains(&user1name);
        assert_eq!(is_user_in_room3, true);

        let user = game_state.users.get_user(&user1name);
        assert_eq!(user.room_name, room3name);

        let is_user_in_room1 = room1.users.contains(&user1name);
        let is_user_in_room2 = room2.users.contains(&user1name);
        assert_eq!(is_user_in_room1, false);
        assert_eq!(is_user_in_room2, false);
    }

    #[test]
    fn move_up_and_back() {
        use super::*;
        let mut game_state = GameState::new();

        let room1name = "room1".to_string();
        let room2name = "room2".to_string();
        game_state.create_room(&room1name, "help".to_string());
        game_state.create_room(&room2name, "ffffffffff".to_string());

        game_state.add_path(&room1name, &room2name, Direction::North);

        let user1name = "user1".to_string();
        game_state.create_user_in_room(&user1name, &room1name, UserType::Civilian);

        game_state.attempt_move(&user1name, &"n".to_string());
        game_state.attempt_move(&user1name, &"s".to_string());

        let user = game_state.users.get_user(&user1name);
        assert_eq!(user.room_name, room1name);

        let room1 = game_state.rooms.get_room(&room1name);
        let room2 = game_state.rooms.get_room(&room2name);
        let is_user_in_room1 = room1.users.contains(&user1name);
        assert_eq!(is_user_in_room1, true);

        let is_user_in_room2 = room2.users.contains(&user1name);
        assert_eq!(is_user_in_room2, false);
    }

    #[test]
    fn move_invalid_direction() {
        use super::*;
        let mut game_state = GameState::new();

        let room1name = "room1".to_string();
        game_state.create_room(&room1name, "blah".to_string());

        let user1name = "user1".to_string();
        game_state.create_user_in_room(&user1name, &room1name, UserType::Civilian);

        let user = game_state.users.get_user(&user1name);
        assert_eq!(user.room_name, room1name);

        let room1 = game_state.rooms.get_room(&room1name);
        let is_user_in_room1 = room1.users.contains(&user1name);
        assert_eq!(is_user_in_room1, true);

        let res = game_state.attempt_move_impl(&user1name, &"northhampton".to_string());
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
        let mut game_state = GameState::new();

        game_state.create_room(&"".to_string(), "The Land Of Dook".to_string());
    }

    #[test]
    #[should_panic(expected = "Empty room descriptions are not allowed!")]
    fn attempt_empty_room_description_creation() {
        use super::*;
        let mut game_state = GameState::new();

        game_state.create_room(&"Mang0".to_string(), "".to_string());
    }

    #[test]
    #[should_panic(expected = "Empty user names are not allowed!")]
    fn attempt_empty_user_name_creation() {
        use super::*;
        let mut game_state = GameState::new();

        game_state.create_room(
            &"Dooklandia".to_string(),
            "Big ol' dook in front of you".to_string(),
        );

        game_state.create_user_in_room(
            &"".to_string(),
            &"Dooklandia".to_string(),
            UserType::Civilian,
        );
    }

    #[test]
    #[should_panic(expected = "Failed to find room named FAKEFRIENDS for mutation!")]
    fn attempt_incorrect_room_user_creation() {
        use super::*;
        let mut game_state = GameState::new();

        let room1name = "Dooklandia".to_string();

        game_state.create_room(&room1name, "Big ol' dook in front of you".to_string());

        game_state.create_user_in_room(
            &"Freddie".to_string(),
            &"FAKEFRIENDS".to_string(),
            UserType::Civilian,
        );
    }

    #[test]
    #[should_panic(expected = "Empty path names are not allowed!")]
    fn attempt_empty_oneway_path_name_creation() {
        use super::*;
        let mut game_state = GameState::new();

        let room1name = "room1".to_string();
        let room2name = "room2".to_string();

        game_state.create_room(&room1name, "The Land Of Dook".to_string());
        game_state.create_room(&room2name, "The Land Of Dook, 2".to_string());

        game_state.add_path(
            &room1name,
            &room2name,
            Direction::CustomOneWay("".to_string()),
        );
    }

    #[test]
    #[should_panic(expected = "Empty path names are not allowed!")]
    fn attempt_empty_twoway_path_name_creation() {
        use super::*;
        let mut game_state = GameState::new();

        let room1name = "room1".to_string();
        let room2name = "room2".to_string();

        game_state.create_room(&room1name, "The Land Of Dook".to_string());
        game_state.create_room(&room2name, "The Land Of Dook, 2".to_string());

        game_state.add_path(
            &room1name,
            &room2name,
            Direction::Custom("mkay".to_string(), "".to_string()),
        );
    }

    #[test]
    #[should_panic(expected = "No room named FAKENEWS exists!")]
    fn attempt_path_to_invalid_room() {
        use super::*;
        let mut game_state = GameState::new();

        let room1name = "room1".to_string();
        let room2name = "room2".to_string();

        game_state.create_room(&room1name, "The Land Of Dook".to_string());
        game_state.create_room(&room2name, "The Land Of Dook, 2".to_string());

        game_state.add_path(
            &room1name,
            &"FAKENEWS".to_string(),
            Direction::Custom("mkay".to_string(), "jkll".to_string()),
        );
    }

    #[test]
    #[should_panic(expected = "Failed to find room named FAKENEWS for mutation!")]
    fn attempt_path_from_invalid_room() {
        use super::*;
        let mut game_state = GameState::new();

        let room1name = "room1".to_string();
        let room2name = "room2".to_string();

        game_state.create_room(&room1name, "The Land Of Dook".to_string());
        game_state.create_room(&room2name, "The Land Of Dook, 2".to_string());

        game_state.add_path(
            &"FAKENEWS".to_string(),
            &room2name,
            Direction::Custom("mkay".to_string(), "jkll".to_string()),
        );
    }

    #[test]
    fn create_room_from_other_room() {
        use super::*;
        let mut game_state = GameState::new();

        let room1name = "room1".to_string();
        let room2name = "room2".to_string();

        game_state.create_room(&room1name, "description".to_string());
        game_state.create_room_from(
            &room2name,
            "description2".to_string(),
            &room1name,
            Direction::North,
        );

        let user1name = "user1".to_string();
        game_state.create_user_in_room(&user1name, &room1name, UserType::Civilian);

        game_state.attempt_move(&user1name, &"north".to_string());
        game_state.attempt_move(&user1name, &"south".to_string());

        let user = game_state.users.get_user(&user1name);
        assert_eq!(user.room_name, room1name);

        let room1 = game_state.rooms.get_room(&room1name);
        let room2 = game_state.rooms.get_room(&room2name);
        let is_user_in_room1 = room1.users.contains(&user1name);
        let is_user_in_room2 = room2.users.contains(&user1name);
        assert_eq!(is_user_in_room1, true);
        assert_eq!(is_user_in_room2, false);
    }

    #[test]
    #[should_panic(expected = "Path 'north' from room1 already exists!")]
    fn test_duplicate_path_creation_panics() {
        use super::*;
        let mut game_state = GameState::new();

        let room1name = "room1".to_string();
        let room2name = "room2".to_string();

        game_state.create_room(&room1name, "description".to_string());
        game_state.create_room_from(
            &room2name,
            "description2".to_string(),
            &room1name,
            Direction::North,
        );

        game_state.add_path(&room1name, &room2name, Direction::North);
    }

    #[test]
    fn test_attempt_global_action() {
        use super::*;
        let mut game_state = GameState::new();

        let room1name = "room1".to_string();
        game_state.create_room(&room1name, "help".to_string());

        let user1name = "user1".to_string();
        game_state.create_user_in_room(&user1name, &room1name, UserType::Civilian);

        //        let action_name = "list_users".to_string();
        //        let global_action = GlobalAction::new(action_name, mk_action_callback(
        //                                              |state: &mut GameState, user_name: &UserName| {
        //                                                  state.print_debug_map();
        //                                                  Ok(ActionSuccess { messages: vec!["Passed brah.".to_string()] })
        //                                              }));
        //        game_state.add_global_action(global_action);
        //
        //        let invalid_action = game_state.attempt_global_action(&user1name, &"fuckem".to_string());
        //        let valid_action = game_state.attempt_global_action(&user1name, action_name);
        //
        //        assert!(invalid_action.is_err()); // make more robust
        //        assert!(valid_action.is_ok()); // make more robust
    }
}
