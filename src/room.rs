use crate::type_aliases::{PathName, RoomName, UserName};
use crate::lambda::{mk_action_callback, ActionFunc, ActionSuccess};
use crate::user::User;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct Room {
    pub name: RoomName,
    pub description: String,
    pub paths: HashMap<PathName, Path>,
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
            paths: HashMap::new(),
            users: HashSet::new(),
        }
    }

    pub fn add_path(&mut self, target_room_name: &RoomName, path_name: &PathName) {
        self.check_duplicate_path(&path_name);
        let path = Path::new(
            target_room_name.clone(),
            path_name.clone(),
            PathType::Normal,
        );
        self.paths.insert(path_name.clone(), path);
    }

    pub fn add_path_special(
        &mut self,
        target_room_name: &RoomName,
        path_name: &PathName,
        path_type: PathType,
    ) {
        let path = Path::new(target_room_name.clone(), path_name.clone(), path_type);
        self.paths.insert(path_name.clone(), path);
    }

    pub fn check_duplicate_path(&self, path_name: &PathName) {
        assert!(
            !self.paths.contains_key(path_name),
            format!("Path '{}' from {} already exists!", &path_name, &self.name) 
        );
    }
}

pub struct Path {
    pub target_room_name: RoomName,
    pub path_name: PathName,
    pub exit_cond: ActionFunc<User>,
}

pub enum PathType {
    Normal,
    Painful,
    Custom(ActionFunc<User>),
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

            Ok(ActionSuccess::new(vec!["You passed through, but it hurt you.".to_string()]))
        };
        let exit_cond = mk_action_callback(clos);

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
    pub fn get_path_name(dir: Direction) -> PathName {
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

    pub fn get_reverse(dir: Direction) -> Option<Direction> {
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
