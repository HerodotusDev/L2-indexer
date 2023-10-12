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
    l2_block: i32,
    l2_output_root: String,
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

async fn connect_db() -> Result<tokio_postgres::Client> {
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

    return Ok(pg_client);
}

async fn get_output_root_from_block(
    params: &ParamsInput,
    pg_client: &tokio_postgres::Client,
) -> Result<(String, i32)> {
    let l2_block = params.l2_block;

    let rows = pg_client
        .query(
            "SELECT l2_output_root, l2_blocknumber 
            FROM optimism 
            ORDER BY ABS(l2_blocknumber - $1)
            LIMIT 1;",
            &[&l2_block],
        )
        .await?;
    if rows.len() == 0 {
        return Err(eyre::eyre!("Expected at least 1 row"));
    } else {
        // Get both output_root and l2_blocknum from the query result
        let l2_output_root: String = rows[0].get(0);
        let l2_blocknum_result: i32 = rows[0].get(1);

        println!("L2 output root: {}", l2_output_root);
        println!("L2 block number: {}", l2_blocknum_result);

        return Ok((l2_output_root, l2_blocknum_result));
    }
}

#[post("/output-root", format = "json", data = "<params>")]
async fn get_output_root(
    params: ParamsInput,
) -> Result<Json<ParamsOutput>, status::Conflict<std::string::String>> {
    let pg_client = connect_db().await.unwrap();
    match get_output_root_from_block(&params, &pg_client).await {
        Ok((l2_output_root, l2_block)) => {
            return Ok(Json(ParamsOutput {
                l2_output_root,
                l2_block,
            }));
        }
        Err(e) => Err(status::Conflict(Some(e.to_string()))),
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![get_output_root])
}
