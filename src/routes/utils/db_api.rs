use sea_orm::{ActiveModelBehavior, DatabaseConnection, DbErr};

#[macro_export]
macro_rules! define_model {
    ($m:ident, $($e:ident),+) => {
        $m{
                $($e: Set($e),)+
                ..Default::default()
        }
    };
}

#[macro_export]
macro_rules! create_model {
    ($m:ident, $db:expr, $($e:ident),+) => {
        {
            let creating_instance = crate::define_model!($m, $($e),+);
            match creating_instance.save($db).await{
                Ok(user) => Ok(user),
                Err(e) => Err(e)
            }
        }
    };
}

// pub async fn create<T: ActiveModelBehavior>(database: &DatabaseConnection, instance: T) -> Result<(), DbErr>{
//     match instance.save(database).await{
//         Ok(_) => Ok(()),
//         Err(e) => Err(e)
//     }
// }