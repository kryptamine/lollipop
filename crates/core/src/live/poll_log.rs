use super::payload::AllGameData;

/// Logs each change in what League's answers say, rather than every poll.
#[derive(Default)]
pub(super) struct PollLog {
    last: Option<String>,
}

impl PollLog {
    pub(super) fn answer(&mut self, answer: &Result<AllGameData, String>) {
        let (state, game_time) = match answer {
            Err(why) => (format!("no answer, {why}"), None),
            Ok(data) => (state_of(data), Some(data.game_time())),
        };
        if self.last.as_ref() == Some(&state) {
            return;
        }
        match game_time {
            Some(at) => log::info!("Live Client: {state}, gameTime {at:.1}"),
            None => log::info!("Live Client: {state}"),
        }
        self.last = Some(state);
    }
}

fn state_of(data: &AllGameData) -> String {
    let stage = if data.at_loading_screen() {
        "loading screen"
    } else if data.game_ended() {
        "game ended"
    } else {
        "game running"
    };
    let listed = data.active_player().is_some();
    format!(
        "{stage}, {} players, active player listed: {listed}",
        data.players().len()
    )
}
