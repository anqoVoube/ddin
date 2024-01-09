use dotenvy_macro::dotenv;
use sea_orm::{Condition, EntityTrait};
use teloxide::Bot;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{Dialogue, Message, Requester};
use teloxide::types::{ButtonRequest, KeyboardButton, KeyboardMarkup};
use teloxide::update_listeners::webhooks::Options;
use crate::database::prelude::User;
use crate::database::user;
use crate::POSTGRES_CONNECTION;


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
            let mut condition = Condition::all();
            condition = condition.add(user::Column::PhoneNumber.eq(contact.phone_number.clone()));
            match User::find().filter(condition).one(&POSTGRES_CONNECTION.get().unwrap()).await{
                Ok(Some(_)) => println!("Found"),
                Ok(None) => println!("Not found"),
                Err(e) => println!("Error, {:?}", e)
            }
            bot.send_message(msg.chat.id, "How old are you?").await?;
            dialogue.exit().await.unwrap();
        }
        None => {
            bot.send_message(msg.chat.id, "Send me plain text.").await?;
        }
    }
    Ok(())
}