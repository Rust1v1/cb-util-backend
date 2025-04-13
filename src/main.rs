#[macro_use] extern crate rocket;
use rocket::tokio::time::{sleep, Duration};
use rocket::serde::{Deserialize, Serialize, json::Json};
use mongodb::{ bson::doc, options::{ ClientOptions, ServerApi, ServerApiVersion }, Client };

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Streamer {
    profile_url: String,
    profile_name: String,
    profile_status: StreamerState,
    download_size_mb: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
enum StreamerState {
    #[serde(rename = "added")]
    Added,
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "downloading")]
    Downloading,
    #[serde(rename = "stopped")]
    Stopped,
    #[serde(rename = "error")]
    Error(u32),
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct StreamerUpdateMessage {
    profile_status: String
}

#[get("/users")]
async fn retrieve_users() -> Result<Json<Vec<Streamer>>, rocket::response::status::Custom<String>> {
    sleep(Duration::from_secs(1)).await;
    let streamer1 = Streamer {
        profile_url: String::from("https://chaturbate.com/ehotlovea"),
        profile_name: String::from("ehotlovea"),
        profile_status: StreamerState::Added,
        download_size_mb: 0,
    };
    let streamer2 = Streamer {
        profile_url: String::from("https://chaturbate.com/brooklyn_white"),
        profile_name: String::from("brooklyn_white"),
        profile_status: StreamerState::Added,
        download_size_mb: 0,
    };
    let streamers = vec![streamer1, streamer2];
    Ok(Json(streamers))
}

#[get("/users/<user>")]
async fn retrieve_user(user: &str) -> Result<Json<Streamer>, rocket::response::status::Custom<String>> {
    sleep(Duration::from_secs(1)).await;
    Ok(Json(Streamer {
        profile_url: format!("https://chaturbate.com/{}", user),
        profile_name: String::from(user),
        profile_status: StreamerState::Stopped,
        download_size_mb: 0,
    }))
}

#[put("/users/<user>")]
async fn add_user(user: &str) -> Result<Json<String>, rocket::response::status::BadRequest<String>> {
    println!("Adding New User: {}", user);
    Ok(Json(String::from("SUCCESS")))
}

#[delete("/users/<user>")]
async fn delete_user(user: &str) -> Result<Json<String>, rocket::response::status::BadRequest<String>> {
    println!("Deleting User: {}", user);
    Ok(Json(String::from("SUCCESS")))
}

#[post("/users/<user>", data = "<msg>")]
async fn mutate_user(user: &str, msg: Json<StreamerUpdateMessage> ) -> Result<Json<String>, rocket::response::status::BadRequest<String>> {
    let update = msg.into_inner();
    println!("Updating User: {} to {}", user, update.profile_status);

    Ok(Json(String::from("SUCCESS")))
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {

    let uri = "mongodb://127.0.0.1:39239";
    let mut client_options = ClientOptions::parse(uri).await.unwrap();
    // Set the server_api field of the client_options object to Stable API version 1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);
    // Create a new client and connect to the server
    let client = Client::with_options(client_options).unwrap();
    // Send a ping to confirm a successful connection
    client.database("admin").run_command(doc! { "ping": 1 }).await.unwrap();
    println!("Pinged your deployment. You successfully connected to MongoDB!");

    let _rocket = rocket::build()
        .mount("/", routes![retrieve_users, retrieve_user, add_user, delete_user, mutate_user])
        .launch().await?;
    Ok(())
}

#[cfg(test)]
mod api_tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let streamer_stopped = Streamer {
            profile_url: String::from("https://chaturbate.com/ehotlovea"),
            profile_name: String::from("ehotlovea"),
            profile_status: StreamerState::Stopped,
            download_size_mb: 0,
        };
        let streamer_error = Streamer {
            profile_url: String::from("https://chaturbate.com/killpretty"),
            profile_name: String::from("killpretty"),
            profile_status: StreamerState::Error(100),
            download_size_mb: 0,
        };
        let stopped_json: String = serde_json::to_string(&streamer_stopped).unwrap();
        let error_json: String = serde_json::to_string(&streamer_error).unwrap();
        assert_eq!(r#"{"profile_url":"https://chaturbate.com/ehotlovea","profile_name":"ehotlovea","profile_status":"stopped","download_size_mb":0}"#, stopped_json);
        assert_eq!(r#"{"profile_url":"https://chaturbate.com/killpretty","profile_name":"killpretty","profile_status":{"error":100},"download_size_mb":0}"#, error_json);
    }
}
