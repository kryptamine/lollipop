use std::sync::{Arc, Mutex};

use super::marker_game::{body, Moment, MASTER_YI_MID};
use super::server::serve;

/// What League answers with for as long as a test leaves it there.
#[derive(Clone, Copy)]
pub enum Stage {
    Running { game_time: f64 },
    Ended,
    Unanswered,
}

/// Live Client Data whose answer a test moves by hand, so a test can wait inside a game.
pub struct StagedGame {
    stage: Arc<Mutex<Stage>>,
    pub url: String,
}

impl StagedGame {
    pub async fn at(stage: Stage) -> Self {
        let stage = Arc::new(Mutex::new(stage));
        let served = Arc::clone(&stage);
        let url = serve(Box::new(move || {
            let stage = *served.lock().unwrap();
            match stage {
                Stage::Running { game_time } => {
                    let moment = Moment::at(game_time);
                    Some(body(MASTER_YI_MID, moment, &["GameStart"]))
                }
                Stage::Ended => {
                    let moment = Moment::at(0.0);
                    Some(body(MASTER_YI_MID, moment, &["GameStart", "GameEnd"]))
                }
                Stage::Unanswered => None,
            }
        }))
        .await;
        Self { stage, url }
    }

    pub fn move_to(&self, stage: Stage) {
        *self.stage.lock().unwrap() = stage;
    }
}
