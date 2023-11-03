



#[derive(Serialize, Deserialize)]
pub struct Body {
    products: Vec<HashMap<String, String>>,
    grand_total: i32,
    paid_amount: i32
}

pub async fn create(
    Extension(AppConnections{redis, database}): Extension<AppConnections>,
    Json(body): Json<Body>
) -> Response{
    ().into_response()
}