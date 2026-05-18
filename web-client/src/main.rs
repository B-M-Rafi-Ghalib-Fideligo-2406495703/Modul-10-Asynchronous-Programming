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
                        chat_msg
                    } else {
                        // Jika bukan JSON, jadikan objek sementara agar layout konsisten
                        ChatMessage {
                            username: "System / Terminal".to_string(),
                            content: text,
                        }
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
                let msg_text = (*input_value).clone();
                if msg_text.trim().is_empty() { return; }
                
                let msg = ChatMessage {
                    username: "Fide's WebChat".to_string(),
                    content: msg_text,
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
        <div class="flex h-screen w-full bg-brand-lightBg font-sans text-brand-textDark shadow-2xl overflow-hidden rounded-xl border border-gray-200 justify-center items-center p-4">
            
            // MAIN CHAT AREA
            <div class="w-full max-w-4xl h-[90vh] flex flex-col bg-white rounded-2xl shadow-xl overflow-hidden">
                // HEADER
                <div class="h-20 border-b border-gray-100 flex items-center px-8 bg-white shrink-0">
                    <div class="flex items-center gap-4">
                        <div class="w-12 h-12 rounded-full bg-brand-purple flex items-center justify-center text-white text-lg font-bold shadow-md">
                            {"FC"}
                        </div>
                        <div>
                            <h2 class="font-bold text-xl text-gray-800">{"Fide's Chatroom"}</h2>
                            <p class="text-xs text-green-500 font-medium flex items-center gap-1">
                                <span class="w-2 h-2 rounded-full bg-green-500 inline-block"></span>
                                {"Online"}
                            </p>
                        </div>
                    </div>
                </div>

                // MESSAGES AREA
                <div class="flex-1 overflow-y-auto px-10 py-8 flex flex-col gap-6 bg-[#fbfbfe]">
                    <div class="text-center text-xs text-brand-textMuted mb-4 font-medium uppercase tracking-wider">{"Welcome to the chat"}</div>
                    {
                        for messages.iter().map(|msg| {
                            let is_mine = msg.username.starts_with("Fide's WebChat");
                            
                            if is_mine {
                                html! {
                                    <div class="flex items-end justify-end gap-3 max-w-3xl self-end group">
                                        <div class="flex flex-col gap-1 items-end">
                                            <div class="bg-brand-bubbleOut text-white rounded-2xl rounded-br-sm px-5 py-3 shadow-md relative">
                                                <p class="text-sm leading-relaxed">{ &msg.content }</p>
                                            </div>
                                        </div>
                                    </div>
                                }
                            } else {
                                html! {
                                    <div class="flex items-end justify-start gap-3 max-w-3xl group">
                                        <div class="w-8 h-8 rounded-full bg-gray-200 shrink-0 overflow-hidden self-start mt-1"><img src={format!("https://ui-avatars.com/api/?name={}&background=random", msg.username)} alt="avatar" class="w-full h-full object-cover"/></div>
                                        <div class="flex flex-col gap-1">
                                            <span class="text-xs font-semibold text-gray-500 ml-2">{ &msg.username }</span>
                                            <div class="bg-brand-bubbleIn text-brand-textDark rounded-2xl rounded-bl-sm px-5 py-3 shadow-[0_2px_15px_-4px_rgba(0,0,0,0.1)] border border-gray-100">
                                                <p class="text-sm leading-relaxed">{ &msg.content }</p>
                                            </div>
                                        </div>
                                    </div>
                                }
                            }
                        })
                    }
                </div>

                // INPUT AREA
                <div class="p-5 bg-white shrink-0 border-t border-gray-100 flex items-center gap-3">
                    <form onsubmit={onsubmit} class="flex-1 flex items-center gap-3">
                        <input 
                            type="text" 
                            placeholder="Write your message..." 
                            value={(*input_value).clone()} 
                            oninput={oninput} 
                            class="flex-1 bg-gray-50 rounded-full py-3 px-6 text-sm outline-none border border-gray-200 focus:border-brand-purple/40 focus:bg-white transition shadow-inner"
                        />
                        <button type="submit" class="w-11 h-11 rounded-full bg-brand-coral flex items-center justify-center text-white shadow-lg hover:bg-orange-500 transition shrink-0 transform hover:scale-105"><svg class="w-5 h-5 ml-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"></path></svg></button>
                    </form>
                </div>
            </div>
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
