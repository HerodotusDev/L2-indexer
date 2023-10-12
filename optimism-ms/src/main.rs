#[macro_use]
extern crate rocket;
use eyre::Result;

use rocket::data::{FromData, Outcome, ToByteUnit};
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;

use dotenv::dotenv;
use rocket::tokio::io::AsyncReadExt;
use rocket::{Data, Request};
use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;

#[derive(Deserialize, Debug)]
pub struct ParamsInput {
    l2_block: i32,
}

#[derive(Serialize, Debug)]
pub struct ParamsOutput {
    pub l2_output_root: String,
}

#[rocket::async_trait]
impl<'r> FromData<'r> for ParamsInput {
    type Error = std::io::Error;

    async fn from_data(_req: &'r Request<'_>, data: Data<'r>) -> rocket::data::Outcome<'r, Self> {
        let mut string = String::new();
        if let Err(e) = data.open(512.kilobytes()).read_to_string(&mut string).await {
            return Outcome::Failure((Status::InternalServerError, e));
        }

        let params: ParamsInput = match serde_json::from_str(&string) {
            Ok(params) => params,
            Err(e) => {
                return Outcome::Failure((
                    Status::UnprocessableEntity,
                    std::io::Error::new(std::io::ErrorKind::InvalidData, e),
                ))
            }
        };

        Outcome::Success(params)
    }
}

async fn get_output_root_from_block(params: &ParamsInput) -> Result<String> {
    dotenv().ok();
    let db_url: &str = &std::env::var("DB_URL").expect("DB_URL must be set.");

    // Establish a PostgreSQL connection
    let (pg_client, connection) = tokio_postgres::connect(db_url, NoTls)
        .await
        .expect("Failed to connect to PostgreSQL");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("PostgreSQL connection error: {}", e);
        }
    });

    let l2_block = params.l2_block;

    let rows = pg_client
        .query(
            "SELECT l2_output_root FROM optimism WHERE l2_blocknumber = $1",
            &[&l2_block],
        )
        .await?;

    // And then check that we got back the same string we sent over.
    let l2_output_root: String = rows[0].get(0);
    println!("L2 output root: {}", l2_output_root);
    return Ok(l2_output_root);
}

#[post("/output-root", format = "json", data = "<params>")]
async fn get_output_root(
    params: ParamsInput,
) -> Result<Json<ParamsOutput>, status::Conflict<std::string::String>> {
    match get_output_root_from_block(&params).await {
        Ok(l2_output_root) => {
            return Ok(Json(ParamsOutput { l2_output_root }));
        }
        Err(e) => Err(status::Conflict(Some(e.to_string()))),
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![get_output_root])
}
