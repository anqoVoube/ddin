use dotenvy_macro::dotenv;
use teloxide::Bot;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{Dialogue, Message, Requester};
use teloxide::types::{ButtonRequest, KeyboardButton, KeyboardMarkup};
use teloxide::update_listeners::webhooks::Options;


pub const OPTIONS: Options = Options {
    address: ([0, 0, 0, 0], 3000).into(),
    url: "https://ddin.uz/webhook".parse().unwrap(),
    certificate: None,
    max_connections: None,
    drop_pending_updates: false,
    secret_token: Some(dotenv!("TELEGRAM_SECRET_TOKEN").to_owned())
};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ReceiveFullName,
    ReceiveAge {
        full_name: String,
    },
    ReceiveLocation {
        full_name: String,
        age: u8,
    },
}


pub async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let keyboard = KeyboardMarkup::new([
        [KeyboardButton::new("Поделиться контактом").request(ButtonRequest::Contact)],
    ]).resize_keyboard(true);

    bot.send_message(msg.chat.id, "Привет! Отправьте Ваши контакты").reply_markup(keyboard).await?;
    dialogue.update(State::ReceiveFullName).await?;
    Ok(())
}

pub async fn receive_full_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.contact() {
        Some(contact) => {
            println!("{}", contact.phone_number);
            bot.send_message(msg.chat.id, "How old are you?").await?;
            dialogue.exit().await.unwrap();
        }
        None => {
            bot.send_message(msg.chat.id, "Send me plain text.").await?;
        }
    }
    Ok(())
}