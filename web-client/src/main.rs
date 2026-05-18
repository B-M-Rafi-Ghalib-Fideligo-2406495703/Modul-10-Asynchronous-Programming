use yew::prelude::*;
use gloo_net::websocket::{futures::WebSocket, Message};
use futures::{StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use futures::channel::mpsc;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ChatMessage {
    username: String,
    content: String,
}

#[function_component(App)]
fn app() -> Html {
    let messages = use_state(|| vec![]);
    let input_value = use_state(|| String::new());
    let tx_state = use_state(|| None);

    let messages_clone = messages.clone();
    let tx_clone = tx_state.clone();

    use_effect_with((), move |_| {
        let websocket = WebSocket::open("ws://127.0.0.1:8080").unwrap();
        let (mut write, mut read) = websocket.split();
        
        let (tx, mut rx) = mpsc::channel::<String>(10);
        tx_clone.set(Some(tx));

        wasm_bindgen_futures::spawn_local(async move {
            while let Some(msg) = rx.next().await {
                write.send(Message::Text(msg)).await.unwrap();
            }
        });

        wasm_bindgen_futures::spawn_local(async move {
            while let Some(msg) = read.next().await {
                if let Ok(Message::Text(text)) = msg {
                    let formatted_msg = if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&text) {
                        format!("{}: {}", chat_msg.username, chat_msg.content)
                    } else {
                        text
                    };

                    let mut current_messages = (*messages_clone).clone();
                    current_messages.push(formatted_msg);
                    messages_clone.set(current_messages);
                }
            }
        });

        || ()
    });

    let oninput = {
        let input_value = input_value.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            input_value.set(input.value());
        })
    };

    let onsubmit = {
        let input_value = input_value.clone();
        let tx_state = tx_state.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if let Some(mut tx) = (*tx_state).clone() {
                let msg = ChatMessage {
                    username: "Fide's WebChat".to_string(),
                    content: (*input_value).clone(),
                };
                let json = serde_json::to_string(&msg).unwrap();
                wasm_bindgen_futures::spawn_local(async move {
                    tx.send(json).await.unwrap();
                });
                input_value.set(String::new());
            }
        })
    };

    html! {
        <div class="chat-container">
            <header class="chat-header">
                <h1>{ "Fide's WebChat" }</h1>
                <div class="icon-avatar">{"💬"}</div>
            </header>
            <div class="chat-history">
                {
                    for messages.iter().map(|msg| html! {
                        <div class="chat-message">{ msg }</div>
                    })
                }
            </div>
            <form class="chat-input-form" onsubmit={onsubmit}>
                <input 
                    type="text" 
                    placeholder="Type your message here..."
                    value={(*input_value).clone()} 
                    oninput={oninput} 
                />
                <button type="submit">{ "Send" }</button>
            </form>
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
