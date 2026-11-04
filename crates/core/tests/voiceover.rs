//! Voiceover following the Advice Widget, with League and OpenAI faked over HTTP.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_core::voiceover::{Reader, Speech};
use serde_json::json;

use support::fake_openai::{FakeOpenAi, Reply};
use support::item_game::{item_table, magic_heavy, settle, ItemGame, SETTLING_POLLS};
use support::loading_game::Updates;

const KILLER: &str = "Foe2#EUW";
const FIRST: &str = "Zed ловит тебя у стены: держи E на его прыжок.";
const SECOND: &str = "Zed снова в лесу: отходи к башне до его ульты.";

fn answer(text: &str) -> Reply {
    Reply::advising(&json!({ "advice": text, "next_item": null, "reason": null }).to_string())
}

/// What the voice was told over polls until OpenAI has answered, with the overlay
/// shown or not.
async fn heard(
    reader: &mut Reader,
    updates: &mut Updates,
    openai: &FakeOpenAi,
    shown: bool,
) -> Vec<Speech> {
    let mut speech = vec![];
    let mut follow = |game: Option<lollipop_core::live::LiveGame>| {
        let advice = game.and_then(|game| game.advice).filter(|_| shown);
        speech.extend(reader.follow(advice.as_ref(), None));
    };
    for _ in 0..SETTLING_POLLS {
        follow(updates.recv().await.unwrap());
    }
    while openai.answered() < openai.requests().len() {
        follow(updates.recv().await.unwrap());
    }
    for _ in 0..3 {
        follow(updates.recv().await.unwrap());
    }
    speech
}

async fn game_with(replies: Vec<Reply>) -> (ItemGame, Updates, FakeOpenAi) {
    let openai = FakeOpenAi::replying(replies).await;
    let game = ItemGame::running_at(300.0, &item_table(&magic_heavy())).await;
    let mut updates = game.poll(&openai.url);
    settle(&mut updates, &openai).await;
    (game, updates, openai)
}

// One reading per Advice: the Widget redraws it every poll, the voice must not.
#[tokio::test]
async fn an_advice_is_read_once_while_it_shows() {
    let (game, mut updates, openai) = game_with(vec![answer(FIRST)]).await;
    let mut reader = Reader::default();
    game.die_to(KILLER, &[], 30.0);

    let speech = heard(&mut reader, &mut updates, &openai, true).await;
    assert_eq!(speech, vec![Speech::Read(FIRST.to_string())]);
}

#[tokio::test]
async fn an_advice_replacing_another_is_read_in_its_place() {
    let (game, mut updates, openai) = game_with(vec![answer(FIRST), answer(SECOND)]).await;
    let mut reader = Reader::default();
    game.die_to(KILLER, &[], 30.0);
    heard(&mut reader, &mut updates, &openai, true).await;
    game.respawn();
    game.set_time(305.0);
    heard(&mut reader, &mut updates, &openai, true).await;
    game.die_to(KILLER, &[], 30.0);

    let speech = heard(&mut reader, &mut updates, &openai, true).await;
    assert_eq!(speech, vec![Speech::Read(SECOND.to_string())]);
}

// The Advice leaving the Widget ends its reading, with nothing queued behind it.
#[tokio::test]
async fn an_advice_dropped_stops_the_reading() {
    let (game, mut updates, openai) = game_with(vec![answer(FIRST)]).await;
    let mut reader = Reader::default();
    game.die_to(KILLER, &[], 30.0);
    heard(&mut reader, &mut updates, &openai, true).await;
    game.set_time(330.0);

    let speech = heard(&mut reader, &mut updates, &openai, true).await;
    assert_eq!(speech, vec![Speech::Stop]);
}

// Alt-tabbing away silences the voice, and coming back does not repeat the Advice.
#[tokio::test]
async fn hiding_the_overlay_stops_the_reading_for_good() {
    let (game, mut updates, openai) = game_with(vec![answer(FIRST)]).await;
    let mut reader = Reader::default();
    game.die_to(KILLER, &[], 30.0);
    heard(&mut reader, &mut updates, &openai, true).await;

    let hidden = heard(&mut reader, &mut updates, &openai, false).await;
    assert_eq!(hidden, vec![Speech::Stop]);
    let shown_again = heard(&mut reader, &mut updates, &openai, true).await;
    assert_eq!(shown_again, vec![]);
}

// An Advice that arrived behind a hidden overlay is still new to the player.
#[tokio::test]
async fn an_advice_that_came_while_hidden_is_read_once_shown() {
    let (game, mut updates, openai) = game_with(vec![answer(FIRST)]).await;
    let mut reader = Reader::default();
    game.die_to(KILLER, &[], 30.0);

    let hidden = heard(&mut reader, &mut updates, &openai, false).await;
    assert_eq!(hidden, vec![]);
    let shown = heard(&mut reader, &mut updates, &openai, true).await;
    assert_eq!(shown, vec![Speech::Read(FIRST.to_string())]);
}
