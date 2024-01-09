use http::StatusCode;
use sea_orm::{Condition, EntityTrait};
use teloxide::Bot;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{Dialogue, Message, Requester};
use teloxide::types::{ButtonRequest, InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup};
use crate::database::prelude::{Business, User};
use crate::database::{business, user};
use crate::{POSTGRES_CONNECTION, State};
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use serde::Serialize;

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Serialize, Debug)]
pub struct BusinessSchema {
    id: i32,
    title: String,
    // location: Vec<Decimal>,
    // works_from: NaiveTime,
    // works_until: NaiveTime,
    // #[serde(default="default_as_false")]
    // is_closed: bool,
    // owner_id: i32
}

impl From<business::Model> for BusinessSchema {
    fn from(business: business::Model) -> Self {
        BusinessSchema {
            id: business.id,
            title: business.title,
            // location: business.location,
            // works_from: business.works_from,
            // works_until: business.works_until,
            // is_closed: business.is_closed,
            // owner_id: business.owner_id
        }
    }
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
            match User::find().filter(condition).one(POSTGRES_CONNECTION.get().unwrap()).await{
                Ok(Some(user)) => {
                    let mut condition = Condition::all();
                    condition = condition.add(business::Column::OwnerId.eq(user.id));
                    let businesses = Business::find()
                        .filter(condition)
                        .all(POSTGRES_CONNECTION.get().unwrap())
                        .await?;
                    let buttons: Vec<[InlineKeyboardButton; 1]> = businesses
                        .into_iter()
                        .map(|x| [InlineKeyboardButton::callback(&x.title, format!("business_{}", &x.id))])
                        .collect();
                    let markup = InlineKeyboardMarkup::new(buttons);
                    bot.send_message(msg.chat.id, "Home").reply_markup(markup).await?;
                },
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