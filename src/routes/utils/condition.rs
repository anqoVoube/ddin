use sea_orm::Condition;
use sea_orm::sea_query::{ConditionExpression, Expr, Func, IntoColumnRef, SimpleExpr};

pub fn starts_with<T>(text: &str, column: T, is_case_sensitive: bool) -> Condition
    where T: IntoColumnRef + sea_orm::ColumnTrait{
        let like_text = if !is_case_sensitive {
            format!("{}%", text.to_lowercase())
        } else {
            text.to_string()
        };

        let condition = if !is_case_sensitive {
            Condition::all().add(
                Expr::expr(Func::lower(Expr::col(column))).like(like_text)
            )
        } else {
            Condition::all().add(
                column.starts_with(like_text)
            )
        };

        condition
}
