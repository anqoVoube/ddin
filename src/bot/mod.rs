use std::error::Error;
use sea_orm::{ActiveModelTrait, Condition, EntityTrait, Set};
use teloxide::Bot;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{CallbackQuery, Dialogue, Message, Request, Requester, ResponseResult};
use teloxide::types::{ButtonRequest, InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup};
use crate::database::prelude::{Business, User};
use crate::database::{business, user, weight_item};
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
}

impl From<business::Model> for BusinessSchema {
    fn from(business: business::Model) -> Self {
        BusinessSchema {
            id: business.id,
            title: business.title,
        }
    }
}

pub async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let keyboard = KeyboardMarkup::new([
        [KeyboardButton::new("Поделиться контактом").request(ButtonRequest::Contact)],
    ]).resize_keyboard(true);

    bot.send_message(msg.chat.id, "Привет! Отправьте Ваши контакты").reply_markup(keyboard).await?;
    dialogue.update(State::ReceiveContact).await?;
    Ok(())
}

pub async fn receive_full_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.contact() {
        Some(contact) => {
            println!("{}", contact.phone_number);
            let mut condition = Condition::all();
            let phone_number = if contact.phone_number.starts_with("+") {contact.phone_number.clone()} else {format!("+{}", &contact.phone_number)};
            condition = condition.add(user::Column::PhoneNumber.eq(format!("{}", phone_number)));
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
            // bot.send_message(msg.chat.id, "How old are you?").await?;
            dialogue.update(State::ChooseBusiness).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Send me plain text.").await?;
        }
    }
    Ok(())
}


pub async fn handle_callback_query(
    bot: Bot,
    dialogue: MyDialogue,
    query: CallbackQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chat_id = query.message.clone().unwrap().chat.clone().id.clone();
    if let data = query.data.unwrap().as_str() {
        let gotten_data = data.split("_").collect::<Vec<&str>>();
        let clear_data = gotten_data.get(0).unwrap();
        let business_id = gotten_data.get(1).unwrap().parse::<i32>().unwrap();
        let database = POSTGRES_CONNECTION.get().unwrap();
        // TODO: refactor

        if data.starts_with("hide") {
            match Business::find_by_id(business_id).one(database).await{
                Ok(Some(business)) => {
                    let mut business: business::ActiveModel = business.into();
                    business.has_full_access = Set(false);
                    if let Err(err) = business.update(database).await {
                        println!("{:?}", err);
                    }
                },
                Ok(None) => todo!(),
                Err(err) => {
                    println!("{:?}", err);
                }
            }
            let markup = InlineKeyboardMarkup::new([
                [InlineKeyboardButton::callback("Открыть статистику", format!("open_{}", business_id))]
            ]);
            bot.send_message(chat_id, "Статус Статистики: Закрытый 📕").reply_markup(markup).await?;
        } else if data.starts_with("open"){
            match Business::find_by_id(business_id).one(database).await{
                Ok(Some(business)) => {
                    let mut business: business::ActiveModel = business.into();
                    business.has_full_access = Set(true);
                    if let Err(err) = business.update(database).await {
                        println!("{:?}", err);
                    }
                },
                Ok(None) => todo!(),
                Err(err) => {
                    println!("{:?}", err);
                }
            }
            let markup = InlineKeyboardMarkup::new([
                [InlineKeyboardButton::callback("Закрыть статистику", format!("hide_{}", business_id))]
            ]);
            bot.send_message(chat_id, "Статус Статистики: Открытый 📖").reply_markup(markup).await?;
        }
    } else {
        bot.send_message(query.message.clone().unwrap().chat.id, "Something went wrong").await?;
    }
    bot.answer_callback_query(&query.id).send().await?;

    Ok(())
}
