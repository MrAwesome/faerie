use std::collections::HashMap;
use std::collections::HashSet;
use crate::type_aliases::{PathName, RoomName, UserName};
use crate::user::User;

pub type RoomPassFunc = Option<Box<dyn FnMut(&mut User) -> Result<Option<String>, String>>>;
fn mk_callback<F: 'static>(f: F) -> RoomPassFunc
where
    F: FnMut(&mut User) -> Result<Option<String>, String>,
{
    Some(Box::new(f))
}

pub struct Room {
    pub name: RoomName,
    pub description: String,
    pub exits: HashMap<PathName, Path>,
    pub users: HashSet<UserName>,
}

impl Room {
    pub fn new(name: RoomName, description: String) -> Room {
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
}

pub struct Path {
    pub target_room_name: RoomName,
    pub path_name: PathName,
    pub exit_cond: RoomPassFunc,
}

pub enum PathType {
    Normal,
    Painful,
    Custom(RoomPassFunc),
}

impl Path {
    pub fn new(target_room_name: RoomName, path_name: PathName, path_type: PathType) -> Path {
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

    pub fn new_painful(target_room_name: RoomName, path_name: PathName) -> Path {
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

    pub fn match_basic_aliases(s: String) -> String {
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

