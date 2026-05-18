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
        <div class="flex h-screen w-full bg-brand-lightBg font-sans text-brand-textDark shadow-2xl overflow-hidden rounded-xl border border-gray-200">
            
            // SIDEBAR
            <div class="w-20 bg-brand-purple flex flex-col items-center py-6 shrink-0">
                <div class="w-10 h-10 bg-white/20 rounded-lg flex items-center justify-center text-white mb-10 cursor-pointer hover:bg-white/30 transition">
                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path></svg>
                </div>
                <div class="flex flex-col gap-6 text-white/50">
                    <div class="w-12 h-12 flex items-center justify-center rounded-xl cursor-pointer hover:text-white hover:bg-white/10 transition"><svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"></path></svg></div>
                    <div class="w-12 h-12 flex items-center justify-center rounded-xl cursor-pointer hover:text-white hover:bg-white/10 transition"><svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 8v8m-4-5v5m-4-2v2m-2 4h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"></path></svg></div>
                    <div class="w-12 h-12 flex items-center justify-center rounded-xl cursor-pointer hover:text-white hover:bg-white/10 transition"><svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5.882V19.24a1.76 1.76 0 01-3.417.592l-2.147-6.15M18 13a3 3 0 100-6M5.436 13.683A4.001 4.001 0 017 6h1.832c4.1 0 7.625-1.234 9.168-3v14c-1.543-1.766-5.067-3-9.168-3H7a3.988 3.988 0 01-1.564-.317z"></path></svg></div>
                    <div class="w-12 h-12 flex items-center justify-center rounded-xl cursor-pointer text-white bg-white/20 relative shadow-inner"><svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"></path></svg>
                        <div class="absolute top-2 right-2 w-2 h-2 bg-brand-coral rounded-full"></div>
                    </div>
                    <div class="w-12 h-12 flex items-center justify-center rounded-xl cursor-pointer hover:text-white hover:bg-white/10 transition mt-auto"><svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"></path><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path></svg></div>
                </div>
            </div>

            // CHAT LIST COLUMN
            <div class="w-80 bg-white border-r border-gray-100 flex flex-col shrink-0">
                <div class="p-6 flex items-center justify-between">
                    <h2 class="text-2xl font-bold">{"Chat"}</h2>
                    <span class="text-sm text-brand-textMuted flex items-center gap-1 cursor-pointer"><svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z"></path></svg> {"Newest"}</span>
                </div>
                <div class="px-6 mb-4">
                    <div class="relative">
                        <svg class="w-5 h-5 absolute left-3 top-2.5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path></svg>
                        <input type="text" placeholder="Search" class="w-full bg-brand-lightBg rounded-lg py-2.5 pl-10 pr-4 text-sm outline-none border border-transparent focus:border-brand-purple/30 transition"/>
                    </div>
                </div>
                <div class="flex-1 overflow-y-auto">
                    // Active contact
                    <div class="flex items-center gap-4 p-4 mx-2 rounded-xl bg-brand-lightBg border-l-4 border-brand-coral cursor-pointer">
                        <div class="w-12 h-12 rounded-full bg-gray-200 overflow-hidden shrink-0"><img src="https://ui-avatars.com/api/?name=Public+Room&background=random" alt="avatar" class="w-full h-full object-cover"/></div>
                        <div class="flex-1 overflow-hidden">
                            <div class="flex justify-between items-center mb-1">
                                <span class="font-semibold text-sm truncate">{"Global Chatroom"}</span>
                                <span class="text-xs text-brand-textMuted shrink-0">{"Live"}</span>
                            </div>
                            <p class="text-xs text-brand-textMuted truncate">{"Join the conversation..."}</p>
                        </div>
                    </div>
                </div>
            </div>

            // MAIN CHAT AREA
            <div class="flex-1 flex flex-col bg-white">
                // HEADER
                <div class="h-24 border-b border-gray-100 flex items-center justify-between px-8 bg-white shrink-0">
                    <div class="flex items-center gap-4">
                        <div class="w-14 h-14 rounded-full bg-gray-200 overflow-hidden relative"><img src="https://ui-avatars.com/api/?name=Public+Room&background=random" alt="avatar" class="w-full h-full object-cover"/>
                            <div class="absolute bottom-0 right-0 w-3.5 h-3.5 bg-green-400 border-2 border-white rounded-full"></div>
                        </div>
                        <div>
                            <h2 class="font-bold text-lg">{"Global Chatroom"}</h2>
                            <p class="text-sm text-brand-textMuted">{"Online"}</p>
                        </div>
                    </div>
                    <div class="flex items-center gap-6">
                        <button class="bg-brand-coral text-white text-sm font-semibold py-2 px-6 rounded-md shadow-md hover:bg-orange-500 transition tracking-wide flex items-center gap-2">
                            <span>{"+"}</span> {" NEW MESSAGE"}
                        </button>
                        <div class="flex items-center gap-4 border-l border-gray-200 pl-6">
                            <div class="relative cursor-pointer text-gray-400 hover:text-gray-600">
                                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"></path></svg>
                                <div class="absolute top-0 right-0 w-2 h-2 bg-brand-coral rounded-full border border-white"></div>
                            </div>
                            <div class="flex items-center gap-2 cursor-pointer">
                                <div class="w-10 h-10 rounded-full bg-gray-200 overflow-hidden"><img src="https://ui-avatars.com/api/?name=Fide&background=434293&color=fff" alt="my-avatar" class="w-full h-full object-cover"/></div>
                                <span class="text-sm font-medium text-gray-700">{"Fide's WebChat"}</span>
                                <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path></svg>
                            </div>
                        </div>
                    </div>
                </div>

                // MESSAGES AREA
                <div class="flex-1 overflow-y-auto px-10 py-8 flex flex-col gap-6 bg-[#fbfbfe]">
                    <div class="text-center text-xs text-brand-textMuted mb-4">{"Today, 11:45 AM"}</div>
                    {
                        for messages.iter().map(|msg| {
                            let is_mine = msg.username.starts_with("Fide's WebChat");
                            
                            if is_mine {
                                html! {
                                    <div class="flex items-end justify-end gap-3 max-w-3xl self-end group">
                                        <div class="bg-brand-bubbleOut text-white rounded-2xl rounded-br-sm px-6 py-4 shadow-sm relative">
                                            <p class="text-sm leading-relaxed">{ &msg.content }</p>
                                        </div>
                                        <div class="w-8 h-8 rounded-full bg-gray-200 shrink-0 overflow-hidden"><img src="https://ui-avatars.com/api/?name=Fide&background=434293&color=fff" alt="my-avatar" class="w-full h-full object-cover"/></div>
                                    </div>
                                }
                            } else {
                                html! {
                                    <div class="flex items-end justify-start gap-3 max-w-3xl group">
                                        <div class="w-10 h-10 rounded-full bg-gray-200 shrink-0 overflow-hidden self-start mt-1"><img src={format!("https://ui-avatars.com/api/?name={}&background=random", msg.username)} alt="avatar" class="w-full h-full object-cover"/></div>
                                        <div class="flex flex-col gap-1">
                                            <span class="text-xs font-semibold text-gray-500 ml-2">{ &msg.username }</span>
                                            <div class="bg-brand-bubbleIn text-brand-textDark rounded-2xl rounded-bl-sm px-6 py-4 shadow-[0_2px_10px_-4px_rgba(0,0,0,0.1)] border border-gray-50">
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
                <div class="p-6 bg-white shrink-0 border-t border-gray-100 flex items-center gap-4">
                    <button class="text-gray-400 hover:text-gray-600 transition"><svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13"></path></svg></button>
                    <form onsubmit={onsubmit} class="flex-1 flex items-center relative">
                        <input 
                            type="text" 
                            placeholder="Write your message..." 
                            value={(*input_value).clone()} 
                            oninput={oninput} 
                            class="w-full bg-brand-lightBg rounded-full py-4 px-6 text-sm outline-none border border-transparent focus:border-brand-purple/20 transition pr-16"
                        />
                        <button type="button" class="absolute right-4 text-gray-400 hover:text-gray-600 transition"><svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.828 14.828a4 4 0 01-5.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg></button>
                    </form>
                    <button class="w-12 h-12 rounded-full bg-brand-coral flex items-center justify-center text-white shadow-lg hover:bg-orange-500 transition shrink-0 transform hover:scale-105" onclick={
                        let input_value = input_value.clone();
                        let tx_state = tx_state.clone();
                        Callback::from(move |_| {
                            if let Some(mut tx) = (*tx_state).clone() {
                                let msg_text = (*input_value).clone();
                                if msg_text.trim().is_empty() { return; }
                                let msg = ChatMessage { username: "Fide's WebChat".to_string(), content: msg_text };
                                let json = serde_json::to_string(&msg).unwrap();
                                wasm_bindgen_futures::spawn_local(async move {
                                    tx.send(json).await.unwrap();
                                });
                                input_value.set(String::new());
                            }
                        })
                    }><svg class="w-5 h-5 ml-1" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"></path></svg></button>
                </div>
            </div>
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
