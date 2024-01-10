use std::error::Error;
use sea_orm::{Condition, EntityTrait};
use teloxide::Bot;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{CallbackQuery, Dialogue, Message, Request, Requester, ResponseResult};
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
            condition = condition.add(user::Column::PhoneNumber.eq(format!("+{}", contact.phone_number.clone())));
            match User::find().filter(condition).one(POSTGRES_CONNECTION.get().unwrap()).await{
                Ok(Some(user)) => {
                    let mut condition = Condition::all();
                    condition = condition.add(business::Column::OwnerId.eq(user.id));
                    let businesses = Business::find()
                        .filter(condition)
                        .all(POSTGRES_CONNECTION.get().unwrap())
                        .await?;
                    if businesses.len() == 1{
                        let working_business = businesses.first().unwrap();
                        if working_business.has_full_access{
                            let markup = InlineKeyboardMarkup::new([
                                [InlineKeyboardButton::callback("Закрыть статистику", format!("hide_{}", working_business.id))]
                            ]);
                            bot.send_message(msg.chat.id, "Статус Статистики: Открытый 📖").reply_markup(markup).await?;
                        } else {
                            let markup = InlineKeyboardMarkup::new([
                                [InlineKeyboardButton::callback("Открыть статистику", format!("open_{}", working_business.id))]
                            ]);
                            bot.send_message(msg.chat.id, "Статус Статистики: Закрытый 📕").reply_markup(markup).await?;
                        }
                    } else {
                        let buttons: Vec<[InlineKeyboardButton; 1]> = businesses
                            .into_iter()
                            .map(|x| [InlineKeyboardButton::callback(&x.title, format!("business_{}", &x.id))])
                            .collect();

                        let markup = InlineKeyboardMarkup::new(buttons);
                        bot.send_message(msg.chat.id, "Home").reply_markup(markup).await?;
                    }

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


pub async fn handle_callback_query(
    bot: Bot,
    dialogue: Dialogue<State, InMemStorage<State>>,
    query: CallbackQuery,
    // Other dependencies if required
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Check if the callback query has data
    if let Some(data) = query.data {
        println!("{}", data);
        match data.as_str() {
            // Handle different callback data
            "some_callback_data" => {
                // Perform actions based on the callback data
                // For example, send a message or update dialogue state
                bot.send_message(query.message.unwrap().chat.id, "You selected an option!").await?;
            },
            // Handle other callback data cases
            _ => {
                // Handle unknown or unexpected callback data
                bot.send_message(query.message.unwrap().chat.id, "Unknown option selected.").await?;
            }
        }
    } else {
        // No callback data present, send an error message or handle accordingly
        bot.send_message(query.message.unwrap().chat.id, "No data received from button.").await?;
    }

    // Optionally, you can also answer the callback query to stop the loading animation on the button
    bot.answer_callback_query(&query.id).send().await?;

    Ok(())
}
