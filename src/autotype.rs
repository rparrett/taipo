use bevy::prelude::*;

use crate::typing::{TypingState, TypingSubmitEvent};
use crate::{GameState, TaipoStage, TaipoState};

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
    game_state: Res<GameState>,
) {
    if !game_state.ready {
        return;
    }

    timer.0.tick(time.delta_seconds());
    if timer.0.finished() {
        info!("autotype timer!");
        if let Some(word) = state.words.get(state.index) {
            match state.state {
                0 => {
                    info!("typing {}", word);
                    typing_state.buf = word.clone();
                    state.state += 1;
                }
                1 => {
                    // chill for a frame
                    info!("chilling");
                    state.state += 1;
                }
                _ => {
                    info!("submitting");
                    typing_submit_events.send(TypingSubmitEvent { text: word.clone() });
                    typing_state.buf = "".to_string();

                    state.state = 0;
                    state.index += 1;
                }
            }
        }
    }
}

impl Plugin for AutoTypePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(AutoTypeState {
            index: 0,
            state: 0,
            words: vec![
                "hiragana".to_string(),
                "toukyou".to_string(),
                "karaoke".to_string(),
                "sanndoicchi".to_string(),
                "takushi-".to_string(),
                "kare-raisu".to_string(),
                "hyakupa-sennto".to_string(),
                "furannsu".to_string(),
                "mainichi".to_string(),
                "kannji".to_string(),
                "mirukuko-hi-".to_string(),
                "katakana".to_string(),
                "akaibo-ru".to_string(),
                "kokonatsu".to_string(),
                "gozaimasu".to_string(),
                "ashikubi".to_string(),
                "kutsushita".to_string(),
                "wainn".to_string(),
                "kamera".to_string(),
                "amerika".to_string(),
                "hoteru".to_string(),
                "esukare-ta-".to_string(),
                "erebe-ta-".to_string(),
                "robotto".to_string(),
                "kayakku".to_string(),
                "yuni-ku".to_string(),
                "nyu-su".to_string(),
                "mayone-zu".to_string(),
                "aisukuri-mu".to_string(),
                "remonn".to_string(),
                "haikinngu".to_string(),
                "gorufu".to_string(),
                "herikoputa-".to_string(),
                "meronnso-da".to_string(),
                "mamonaku".to_string(),
                "arigatou".to_string(),
            ],
        });
        app.insert_resource(AutoTypeTimer(Timer::from_seconds(0.2, true)));
        app.add_stage_before(CoreStage::Update, "autotype", SystemStage::parallel());
        app.add_system_to_stage("autotype", update.system());
    }
}
