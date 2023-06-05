use dioxus::prelude::*;

use crate::{start, UserInput};

pub fn App(cx: Scope) -> Element {
    let input: &UseState<UserInput> = use_state(cx, UserInput::default);
    cx.render(rsx! {
        div {
            p { "選擇設定檔" },
            input {
                r#type: "file",
                accept: ".json, application/json",
                multiple: false,
                onchange: |evt| {
                    to_owned![input];
                    async move {
                        if let Some(file_engine) = &evt.files {
                            input.modify(|old_input| {
                                let mut new_input = old_input.clone();
                                let default_value = String::default();
                                new_input.file_name = file_engine.files().first().unwrap_or(&default_value).clone();
                                new_input
                            });
                        }
                    }
                },
            },
            br {},
            p { "輸入測試的間隔時間" },
            input {
                name: "duration",
                r#type: "number",
                onchange: |evt| {
                    to_owned![input];
                    async move {
                        if let Ok(value) = evt.data.value.parse::<u64>() {
                            input.modify(|old_input| {
                                let mut new_input = old_input.clone();
                                new_input.duration = value;
                                new_input
                            });
                        }
                    }
                },
            },
            br {},
            button {
                onclick: |_event| {
                    to_owned![input];
                    async move {
                        start(input.get().clone()).await;
                    }
                },
                "開始推播"
            },
        }
    })
}
