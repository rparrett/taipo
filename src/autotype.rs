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
                "shatsu".to_string(),
                "butaniku".to_string(),
                "ofuro".to_string(),
                "byouki".to_string(),
                "banngohann".to_string(),
                "hirugohann".to_string(),
                "asagohann".to_string(),
                "nomimono".to_string(),
                "tabemono".to_string(),
                "douzo".to_string(),
                "yoroshiku".to_string(),
                "dennsha".to_string(),
                "chotto".to_string(),
                "chiisai".to_string(),
                "tannjoubi".to_string(),
                "daijoubu".to_string(),
                "zennbu".to_string(),
                "jitennsha".to_string(),
                "sakana".to_string(),
                "gyuuniku".to_string(),
                "tamago".to_string(),
                "daunnro-do".to_string(),
                "pe-ji".to_string(),
                "isogashii".to_string(),
                "tsuduku".to_string(),
                "eigo".to_string(),
                "keisatsu".to_string(),
                "kodomo".to_string(),
                "nuigurumi".to_string(),
                "ninngenn".to_string(),
                "ninnjinn".to_string(),
                "sennsei".to_string(),
                "mukashimukashi".to_string(),
                "honntou".to_string(),
                "oboeru".to_string(),
                "rippa".to_string(),
                "ennpitsu".to_string(),
                "kippu".to_string(),
                "kannpeki".to_string(),
                "shippo".to_string(),
                "taihenn".to_string(),
                "amegafuru".to_string(),
                "kasa".to_string(),
                "soudesune".to_string(),
                "tanuki".to_string(),
                "yukigafuru".to_string(),
                "gasorinn".to_string(),
                "ba-ga-".to_string(),
                "joginngu".to_string(),
                "ge-mu".to_string(),
                "deza-to".to_string(),
                "purezennto".to_string(),
                "rizo-to".to_string(),
                "nu-doru".to_string(),
                "basu".to_string(),
                "bitaminn".to_string(),
                "te-buru".to_string(),
                "konnpyu-ta".to_string(),
                "ochawonomu".to_string(),
                "kapibara".to_string(),
                "kitsune".to_string(),
                "kuroneko".to_string(),
                "shibainu".to_string(),
                "suro-rorisu".to_string(),
                "kotoba".to_string(),
                "meronnpann".to_string(),
                "kirei".to_string(),
                "kawaii".to_string(),
                "koronauirusu".to_string(),
                "kokonoka".to_string(),
                "tsuitachi".to_string(),
                "shichigatsunanoka".to_string(),
                "hachigatsuhatsuka".to_string(),
                "kugatsukokonoka".to_string(),
                "juuichigatsutsuitachi".to_string(),
                "nihonngo".to_string(),
                "kudasai".to_string(),
                "hitotsu".to_string(),
                "futatsu".to_string(),
                "mittsu".to_string(),
                "yottsu".to_string(),
                "itsutsu".to_string(),
                "muttsu".to_string(),
                "nanatsu".to_string(),
                "yattsu".to_string(),
                "kokonotsu".to_string(),
                "sennenn".to_string(),
                "ichimannenn".to_string(),
                "nichiyoubi".to_string(),
                "getsuyoubi".to_string(),
                "kayoubi".to_string(),
                "suiyoubi".to_string(),
                "mokuyoubi".to_string(),
                "kinnyoubi".to_string(),
                "doyoubi".to_string(),
                "sannzennenn".to_string(),
                "ichigatsu".to_string(),
                "nigatsu".to_string(),
                "sanngatsu".to_string(),
                "shigatsu".to_string(),
                "gogatsu".to_string(),
                "rokugatsu".to_string(),
                "shichigatsu".to_string(),
                "hachigatsu".to_string(),
                "kugatsu".to_string(),
                "juugatsu".to_string(),
                "juuichigatsu".to_string(),
                "juunigatsu".to_string(),
                "ookii".to_string(),
                "daigakusei".to_string(),
                "hidarite".to_string(),
                "migite".to_string(),
                "daijoubu".to_string(),
                "nishi".to_string(),
                "higashi".to_string(),
                "kita".to_string(),
                "minami".to_string(),
                "konngetsu".to_string(),
                "jouzu".to_string(),
                "juuichiji".to_string(),
                "gannbatte".to_string(),
                "atarashii".to_string(),
                "yasai".to_string(),
                "jouzu".to_string(),
                "poketto".to_string(),
                "dennki".to_string(),
                "sofutowea".to_string(),
                "mejiro".to_string(),
                "hatsuka".to_string(),
                "kyou".to_string(),
                "yoyogi".to_string(),
                "shinnjuku".to_string(),
                "shinnbashi".to_string(),
                "tabata".to_string(),
                "kannda".to_string(),
                "tamachi".to_string(),
                "gotannda".to_string(),
                "yuurakuchou".to_string(),
                "ueno".to_string(),
                "shinagawa".to_string(),
                "yamanouchimachi".to_string(),
            ],
        });
        app.insert_resource(AutoTypeTimer(Timer::from_seconds(0.2, true)));
        app.add_stage_before(CoreStage::Update, "autotype", SystemStage::parallel());
        app.add_system_to_stage("autotype", update.system());
    }
}
