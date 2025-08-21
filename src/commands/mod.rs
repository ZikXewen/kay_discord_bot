mod now_playing;
mod play;
mod queue;
mod skip;
mod stop;

pub fn all_commands() -> Vec<crate::Command> {
    vec![
        now_playing::now_playing(),
        play::play(),
        queue::queue(),
        stop::stop(),
        skip::skip(),
    ]
}
