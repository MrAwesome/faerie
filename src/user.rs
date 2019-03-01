use crate::type_aliases::{RoomName, UserName};

#[derive(Debug)]
pub enum UserType {
    Civilian,
    Viking,
    ElfLord,
}

#[derive(Debug)]
pub struct User {
    pub name: UserName,
    pub room_name: RoomName,
    pub basic_attributes: BasicAttributes,
    pub special_attributes: SpecialAttributes,
}

impl User {
    pub fn new(name: UserName, starting_room_name: RoomName, user_type: UserType) -> User {
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
