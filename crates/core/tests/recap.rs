//! The Recap after a game, with League, the League client and OpenAI faked over HTTP.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_core::advice::LessonsOff;
use lollipop_core::game_inputs::Role;
use lollipop_core::live::{Lessons, Outcome, RecapMinute, Scores};

use support::fake_openai::{FakeOpenAi, Reply};
use support::recap_game::{farming, lessons_reply, Beat, RecapBench};

const LESSON: &str = "К 10:00 ты отстал на 20 CS: добивай волну перед каждым походом в лес.";

/// Ten minutes at 6 CS a minute against the Cell's 8, then `GameEnd` at 10:30.
fn ten_minute_win() -> Vec<Beat> {
    let mut beats = farming(10, 6);
    beats.push(Beat::Ended {
        at: 630.0,
        cs: 63,
        result: "Win",
    });
    beats
}

#[tokio::test]
async fn a_game_seen_ending_gets_a_recap_against_its_cell() {
    let played = RecapBench::new().await.play(ten_minute_win()).await;

    assert_eq!(played.recaps.len(), 1);
    let recap = played.last();
    assert_eq!(recap.champion, "Ahri");
    assert_eq!(recap.role, Some(Role::Mid));
    assert_eq!(recap.outcome, Some(Outcome::Win));
    assert!((recap.length - 630.0).abs() < f64::EPSILON);
    assert_eq!(
        recap.scores,
        Some(Scores {
            kills: 1,
            deaths: 2,
            assists: 3
        })
    );
    let expected: Vec<RecapMinute> = (0..=10)
        .map(|minute| RecapMinute {
            minute,
            cs: 6 * minute,
            gold_earned: Some(500 + 300 * minute),
            cs_benchmark: Some(8.0 * f64::from(minute)),
            gold_benchmark: Some(400.0f64.mul_add(f64::from(minute), 500.0)),
        })
        .collect();
    assert_eq!(recap.minutes, expected);
}

// Live Client Data drops about 1 s after `GameEnd`, so most polls never see it.
#[tokio::test]
async fn a_game_whose_end_no_poll_saw_gets_a_recap_once_the_client_says_it_is_over() {
    let mut beats = farming(10, 6);
    beats.push(Beat::Gone("WaitingForStats"));
    let played = RecapBench::new().await.play(beats).await;

    assert_eq!(played.recaps.len(), 1);
    let recap = played.last();
    assert_eq!(recap.outcome, None);
    assert!((recap.length - 600.0).abs() < f64::EPSILON);
    assert_eq!(recap.minutes.len(), 11);
}

// A dropped poll mid-game, as on a lag spike, is no end of the game.
#[tokio::test]
async fn failed_polls_mid_game_make_no_recap() {
    let mut beats = farming(10, 6);
    beats.extend([Beat::Gone("InProgress"); 3]);
    beats.push(Beat::Running { at: 603.0, cs: 60 });
    let played = RecapBench::new().await.play(beats).await;

    assert!(played.recaps.is_empty());
}

#[tokio::test]
async fn a_remake_gets_no_recap() {
    let mut beats = farming(3, 6);
    beats.push(Beat::Ended {
        at: 210.0,
        cs: 21,
        result: "Lose",
    });
    let played = RecapBench::new().await.play(beats).await;

    assert!(played.recaps.is_empty());
}

// The victory screen still holds the game's window; the Recap waits for the client,
// and a client that cannot be read is no post-game screen.
#[tokio::test]
async fn the_window_opens_once_on_the_post_game_screens() {
    let mut beats = ten_minute_win();
    beats.extend([
        Beat::Gone("InProgress"),
        Beat::Gone("WaitingForStats"),
        Beat::Unanswered,
    ]);
    beats.push(Beat::Gone("PreEndOfGame"));
    let opened_on = beats.len() - 1;
    let played = RecapBench::new().await.play(beats).await;

    assert_eq!(played.opened_at, [opened_on]);
}

#[tokio::test]
async fn the_recap_marks_each_death() {
    let mut bench = RecapBench::new().await;
    bench.deaths = vec![150.0, 400.0];
    let played = bench.play(ten_minute_win()).await;

    assert_eq!(played.last().deaths, [150.0, 400.0]);
}

// A lead banked early hides a player who stopped farming; the stretch shows where.
#[tokio::test]
async fn the_recap_marks_where_farm_slowed() {
    let cs_at = |at: f64| -> u32 {
        let farmed = if at <= 300.0 {
            at
        } else if at <= 540.0 {
            300.0
        } else {
            at - 240.0
        };
        #[allow(
            clippy::as_conversions,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a few hundred CS, never negative"
        )]
        let cs = (farmed / 60.0 * 8.0) as u32;
        cs
    };
    let mut beats: Vec<Beat> = (0..=40)
        .map(|half| {
            let at = f64::from(half) * 30.0;
            Beat::Running { at, cs: cs_at(at) }
        })
        .collect();
    beats.push(Beat::Ended {
        at: 1230.0,
        cs: cs_at(1230.0),
        result: "Lose",
    });
    let played = RecapBench::new().await.play(beats).await;

    let slow = &played.last().slow_farm;
    assert_eq!(slow.len(), 1, "{slow:?}");
    assert!(slow[0].from > 300.0 && slow[0].from <= 540.0, "{slow:?}");
    assert!(slow[0].to > 540.0 && slow[0].to < 1230.0, "{slow:?}");
}

#[tokio::test]
async fn lessons_arrive_after_the_recap() {
    let openai = FakeOpenAi::advising(&lessons_reply(&[LESSON, "Второй урок."])).await;
    let played = RecapBench::new()
        .await
        .play_asking(ten_minute_win(), &openai.url, Ok(()))
        .await;

    assert_eq!(played.recaps.first().unwrap().lessons, Lessons::Asking);
    assert_eq!(
        played.last().lessons,
        Lessons::Given(vec![LESSON.to_string(), "Второй урок.".to_string()])
    );
}

#[tokio::test]
async fn lessons_are_asked_with_the_game_against_its_cell_and_no_names() {
    let openai = FakeOpenAi::advising(&lessons_reply(&[LESSON])).await;
    let mut bench = RecapBench::new().await;
    bench.deaths = vec![150.0];
    bench
        .play_asking(ten_minute_win(), &openai.url, Ok(()))
        .await;

    let bodies = openai.bodies();
    assert_eq!(bodies.len(), 1);
    assert_eq!(bodies[0]["text"]["format"]["name"], "lessons");
    let input = bodies[0]["input"].as_str().unwrap();
    assert!(input.contains("Language: Russian\n"));
    assert!(input.contains("Game: Ahri, Mid, Gold, length 10:30, Victory\n"));
    assert!(input.contains("Minute 5: CS 30 against 40, gold earned 2000 against 2500\n"));
    assert!(input.contains("Minute 10: CS 60 against 80, gold earned 3500 against 4500\n"));
    assert!(input.contains("Death at 2:30: to Zed\n"));
    assert_eq!(input.matches("Enemy: ").count(), 5);
    assert_eq!(input.matches("Ally: ").count(), 4);
    for name in ["#EUW", "Foe", "Friend", "Me,"] {
        assert!(!input.contains(name), "{name} is in the prompt");
    }
}

// The window says why there are no Lessons, so the reason travels with the Recap.
#[tokio::test]
async fn lessons_not_asked_for_say_why() {
    let openai = FakeOpenAi::advising(&lessons_reply(&[LESSON])).await;
    let played = RecapBench::new()
        .await
        .play_asking(ten_minute_win(), &openai.url, Err(LessonsOff::SwitchedOff))
        .await;

    assert_eq!(played.last().lessons, Lessons::Off(LessonsOff::SwitchedOff));
    assert!(openai.requests().is_empty());
}

// A model with nothing to say and a call that failed read differently in the window.
#[tokio::test]
async fn no_lessons_and_failed_lessons_differ() {
    let empty = FakeOpenAi::advising(&lessons_reply(&[])).await;
    let failing = FakeOpenAi::replying(vec![Reply::Hangup]).await;
    let bench = RecapBench::new().await;

    let given_none = bench
        .play_asking(ten_minute_win(), &empty.url, Ok(()))
        .await;
    let failed = bench
        .play_asking(ten_minute_win(), &failing.url, Ok(()))
        .await;

    assert_eq!(given_none.last().lessons, Lessons::Empty);
    assert_eq!(failed.last().lessons, Lessons::Failed);
}

// Lessons are never cut to fit: an answer past the cap is dropped whole.
#[tokio::test]
async fn more_than_three_lessons_are_dropped_whole() {
    let openai = FakeOpenAi::advising(&lessons_reply(&["Один.", "Два.", "Три.", "Четыре."])).await;
    let played = RecapBench::new()
        .await
        .play_asking(ten_minute_win(), &openai.url, Ok(()))
        .await;

    assert_eq!(played.last().lessons, Lessons::Failed);
}
