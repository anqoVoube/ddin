use std::error::Error;
use easify::unpack_split;
use sea_orm::{ActiveModelTrait, Condition, EntityTrait, Set};
use teloxide::Bot;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{CallbackQuery, ChatId, Dialogue, Message, Request, Requester, ResponseResult};
use teloxide::types::{ButtonRequest, Contact, InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup};
use crate::database::prelude::{Business, User, TelegramUser};
use crate::database::{business, telegram_user, user, weight_item};
use crate::{create_model, POSTGRES_CONNECTION, State};
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

pub fn make_keyboard(variations: Vec<[&str; 2]>, pre_callback: &str, chunk_size: usize) -> InlineKeyboardMarkup{
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];
    for variation in variations.chunks(chunk_size){
        let row = variation
            .iter()
            .map(
                |&value| InlineKeyboardButton::callback(
                    value[0],
                    format!("{}_{}", pre_callback, value[1])
                )
            )
            .collect();

        keyboard.push(row);
    }
    InlineKeyboardMarkup::new(keyboard)
}


pub async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let chat_id= msg.chat.id.0;
    println!("{}", chat_id);
    let mut condition = Condition::all();
    condition = condition.add(telegram_user::Column::TelegramId.eq(chat_id));
    match TelegramUser::find().filter(condition).one(POSTGRES_CONNECTION.get().unwrap()).await{
        Ok(Some(telegram_user)) => {
            bot.send_message(msg.chat.id, "Бот активирован.\nВы будете получать здесь сообщения с кодом для подтверждения своего аккаунта").await?;
        },

        Ok(None) => {
            bot
                .send_message(msg.chat.id, "Привет! Выбери язык")
                .reply_markup(
                    make_keyboard(
                        vec![["Ozbek", "uz"], ["Russian", "ru"], ["English", "en"]],
                        "lang",
                        1
                    ),
                )
                .await?;
            dialogue.update(State::CallbackHandler).await?;
        },
        Err(e) => println!("{:?}", e)
    }
    Ok(())
    // let keyboard = KeyboardMarkup::new([
    //     [KeyboardButton::new("Поделиться контактом").request(ButtonRequest::Contact)],
    // ]).resize_keyboard(true);
    //
    // bot.send_message(msg.chat.id, "Привет! Отправьте Ваши контакты").reply_markup(keyboard).await?;
    // dialogue.update(State::ReceiveContact).await?;
    // Ok(())
}

// pub async fn receive_full_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
//     match msg.contact() {
//         Some(contact) => {
//             println!("{}", contact.phone_number);
//             let mut condition = Condition::all();
//             let phone_number = if contact.phone_number.starts_with("+") {contact.phone_number.clone()} else {format!("+{}", &contact.phone_number)};
//             condition = condition.add(user::Column::PhoneNumber.eq(format!("{}", phone_number)));
//             match User::find().filter(condition).one(POSTGRES_CONNECTION.get().unwrap()).await{
//                 Ok(Some(user)) => {
//                     todo!();
//                 }
//                 Ok(None) => println!("Not found"),
//                 Err(e) => println!("Error, {:?}", e)
//             }
//             // bot.send_message(msg.chat.id, "How old are you?").await?;
//             dialogue.update(State::ChooseBusiness).await?;
//         }
//         None => {
//             bot.send_message(msg.chat.id, "Send me plain text.").await?;
//         }
//     }
//     Ok(())
// }


pub async fn receive_first_name(
    bot: Bot,
    dialogue: MyDialogue,
    lang: String,
    msg: Message
) -> HandlerResult{
    let chat_id= msg.chat.id.0;
    match msg.text(){
        Some(text) => {
            bot.send_message(msg.chat.id, "Введите свою фамилию").await?;
            dialogue.update(State::ReceiveLastName {lang, first_name: text.to_owned()}).await?
        },
        None => {bot.send_message(msg.chat.id, "Вы не указали своё имя").await?;}
    };

    Ok(())
}

pub async fn receive_last_name(
    bot: Bot,
    dialogue: MyDialogue,
    (lang, first_name): (String, String),
    msg: Message
) -> HandlerResult{
    let chat_id= msg.chat.id.0;
    let location_keyboard = KeyboardButton{
        text: "Отправить номер телефона".to_owned(),
        request: Some(ButtonRequest::Contact)
    };
    let markup = KeyboardMarkup::new([[location_keyboard]]);
    match msg.text(){
        Some(text) => {
            bot.send_message(msg.chat.id, "Отправьте свои контакты").reply_markup(markup).await?;
            dialogue.update(
                State::ReceiveContact {
                    lang,
                    first_name,
                    last_name: text.to_owned()
                }
            ).await?
        },
        None => {bot.send_message(msg.chat.id, "Вы не указали свою фамилию").await?;}
    };

    Ok(())
}

pub async fn receive_contacts(
    bot: Bot,
    dialogue: MyDialogue,
    (lang, first_name, last_name): (String, String, String),
    msg: Message) -> HandlerResult{
    let chat_id = msg.chat.id.0;
    match &msg.contact(){
        Some(Contact {phone_number, ..}) => {
            let user = user::ActiveModel{
                first_name: Set(first_name),
                last_name: Set(last_name),
                phone_number: Set(phone_number.to_string()),
                is_verified: Set(true),
                ..Default::default()
            };

            match user.save(POSTGRES_CONNECTION.get().unwrap()).await{
                Ok(created_user) => {
                    let creating_telegram_user = telegram_user::ActiveModel{
                        user_id: created_user.id,
                        full_name: Set(Some(format!("{} {}", msg.chat.first_name().unwrap_or(""), msg.chat.last_name().unwrap_or("")))),
                        username: Set(Some(msg.chat.username().unwrap_or("").to_string())),
                        telegram_id: Set(chat_id),
                        lang: Set(lang),
                        ..Default::default()
                    };

                    match creating_telegram_user.save(POSTGRES_CONNECTION.get().unwrap()).await{
                        Ok(_) => {},
                        Err(e) => {println!("{}", e)}
                    }
                },
                Err(e) => {println!("{}", e)}
            }
            bot.send_message(msg.chat.id, "Благодарим Вас за регистрацию").await?;
            dialogue.update(State::Start).await?;
        },
        None => { bot.send_message(msg.chat.id, "Вы неправильно отправили свои контакты, пожалуйста нажмите на кнопку").await?; }
    }

    Ok(())
}

pub async fn handle_callback_query(
    bot: Bot,
    dialogue: MyDialogue,
    query: CallbackQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("received");
    let chat_id = query.message.clone().unwrap().chat.clone().id.clone();
    println!("{}", chat_id);
    if let data = query.data.unwrap().as_str() {
        let (_, lang) = unpack_split!(data, "_", 2);
        dialogue.update(
            State::ReceiveFirstName { lang: lang.to_owned() }
        ).await?;
        bot.send_message(chat_id, "Введите имя").await?;
    } else {
        bot.send_message(query.message.clone().unwrap().chat.id, "Something went wrong").await?;
    }
    bot.answer_callback_query(&query.id).send().await?;

    Ok(())
}
