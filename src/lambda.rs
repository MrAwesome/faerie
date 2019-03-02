pub struct ActionSuccess {
    pub messages: Vec<String>,
}

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
