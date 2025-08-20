mod play;
mod queue;
mod skip;
mod stop;

pub fn all_commands() -> Vec<crate::Command> {
    vec![play::play(), queue::queue(), stop::stop(), skip::skip()]
}
