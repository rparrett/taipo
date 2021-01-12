use bevy::prelude::*;

use crate::typing::{TypingState, TypingSubmitEvent};
use crate::{AppState, STAGE};

pub struct AutoTypePlugin;

struct AutoTypeTimer(Timer);

struct AutoTypeState {
    index: usize,
    state: u32,
    words: Vec<String>,
}
// not about to dive into why dashes aren't working here, but the game happily accepts underscores instead

fn update(
    time: Res<Time>,
    mut timer: ResMut<AutoTypeTimer>,
    mut state: ResMut<AutoTypeState>,
    mut typing_submit_events: ResMut<Events<TypingSubmitEvent>>,
    mut typing_state: ResMut<TypingState>,
) {
    timer.0.tick(time.delta_seconds());
    if timer.0.finished() {
        println!("autotype timer!");
        if let Some(word) = state.words.get(state.index) {
            match state.state {
                0 => {
                    println!("typing {}", word);
                    typing_state.buf = word.clone();
                    state.state += 1;
                }
                1 => {
                    // chill for a frame
                    println!("chilling");
                    state.state += 1;
                }
                _ => {
                    println!("submitting");
                    typing_submit_events.send(TypingSubmitEvent { text: word.clone() });

                    state.state = 0;
                    state.index += 1;
                }
            }
        }
    }
}

impl Plugin for AutoTypePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(AutoTypeState {
            index: 0,
            state: 0,
            words: vec![
                "akaibo-ru".to_string(),
                "tamago".to_string(),
                "kasa".to_string(),
                "toukyou".to_string(),
                "karaoke".to_string(),
                "sanndoicchi".to_string(),
                "takushi-".to_string(),
                "kare-raisu".to_string(),
                "hyakupa-sennto".to_string(),
                "furannsu".to_string(),
                "hiragana".to_string(),
                "mirukuko-hi-".to_string(),
                "meronnpann".to_string(),
                "hitotsu".to_string(),
                "itsutsu".to_string(),
                "muttsu".to_string(),
                "nanatsu".to_string(),
                "yattsu".to_string(),
                "kokonotsu".to_string(),
                "sennenn".to_string(),
                "mainichi".to_string(),
                "kannji".to_string(),
                "kokonatsu".to_string(),
                "gannbatte".to_string(),
                "mamonaku".to_string(),
                "arigatou".to_string(),
                "gozaimasu".to_string(),
                "nichiyoubi".to_string(),
                "getsuyoubi".to_string(),
                "kayoubi".to_string(),
                "suiyoubi".to_string(),
                "mokuyoubi".to_string(),
                "kinnyoubi".to_string(),
                "katakana".to_string(),
                "mittsu".to_string(),
            ],
        });
        app.add_resource(AutoTypeTimer(Timer::from_seconds(0.7, true)));
        //app.add_system(update.system());
        //app.on_state_update(STAGE, AppState::Ready, update.system());
        app.add_stage_before(stage::UPDATE, "autotype", SystemStage::parallel());
        app.add_system_to_stage("autotype", update.system());
    }
}
