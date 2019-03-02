#[derive(Debug)]
pub struct ActionSuccess {
    pub messages: Vec<String>,
    room_move: bool,
}

impl ActionSuccess {
    pub fn new(messages: Vec<String>) -> ActionSuccess {
        ActionSuccess {
            messages,
            room_move: false,
        }
    }

    pub fn set_was_room_move(&mut self) {
        self.room_move = true;
    }

    pub fn was_room_move(&self) -> bool {
        self.room_move
    }

}

#[derive(Debug)]
pub struct ActionFailure {
    pub messages: Vec<String>,
}

pub type ActionFunc<T> = Option<Box<dyn FnMut(&mut T) -> Result<ActionSuccess, ActionFailure>>>;
pub fn mk_action_callback<F: 'static, T>(f: F) -> ActionFunc<T>
where
    F: FnMut(&mut T) -> Result<ActionSuccess, ActionFailure>,
{
    Some(Box::new(f))
}
