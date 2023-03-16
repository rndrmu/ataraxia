use super::id::UserId;


pub trait Mentionable {
    fn mention(&self) -> String;
}

impl Mentionable for UserId {
    fn mention(&self) -> String {
        format!("<@{}>", self.0)
    }
}