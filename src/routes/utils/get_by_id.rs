use sea_orm::{DatabaseConnection, EntityTrait};

pub async fn get_object_by_id<T: EntityTrait>(model: T, database: &DatabaseConnection, id: i32) -> Option<T>{
    let result = model.find_by_id(id).one(database).await.unwrap().await;
    if let Ok(instance) = result {
        instance
    } else{
        None
    }
}
