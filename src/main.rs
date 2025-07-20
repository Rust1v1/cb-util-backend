#[macro_use]
extern crate rocket;
use futures::StreamExt;
use futures::stream::TryStreamExt;
use mongodb::bson::doc;
use rocket::http::Status;
use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket_db_pools::{Connection, Database, mongodb};

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Streamer {
    profile_url: String,
    profile_name: String,
    profile_status: StreamerState,
    download_size_mb: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
enum StreamerState {
    #[serde(rename = "added")]
    Added,
    #[serde(rename = "waiting")]
    Waiting,
    #[serde(rename = "downloading")]
    Downloading,
    #[serde(rename = "stopped")]
    Stopped,
    #[serde(rename = "stopping")]
    Stopping,
    #[serde(rename = "error")]
    Error(u32),
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct StreamerUpdateMessage {
    profile_status: StreamerState,
    download_size_mb: u64,
}

#[derive(Database)]
#[database("mongo")]
struct StreamersDB(mongodb::Client);

#[get("/users")]
async fn retrieve_users(
    db: Connection<StreamersDB>,
) -> Result<Json<Vec<Streamer>>, rocket::response::status::Custom<String>> {
    let streamer_cursor = (&*db)
        .database("cbutil")
        .collection::<Streamer>("streamers")
        .find(None, None)
        .await;
    match streamer_cursor {
        Ok(cursor) => {
            if let Ok(streamers) = cursor.try_collect().await {
                return Ok(Json(streamers));
            } else {
                return Err(rocket::response::status::Custom(
                    Status::InternalServerError,
                    String::from("Could Not Group Streamers From Database"),
                ));
            }
        }
        Err(e) => {
            return Err(rocket::response::status::Custom(
                Status::InternalServerError,
                e.to_string(),
            ));
        }
    };
}

#[get("/users/state/<state>")]
async fn retrieve_stateful_users(
    db: Connection<StreamersDB>,
    state: &str,
) -> Result<Json<Vec<Streamer>>, rocket::response::status::Custom<String>> {
    match state {
        "waiting" => (),
        "downloading" => (),
        "stopped" => (),
        "stopping" => (),
        "error" => (),
        _ => {
            println!("Invalid State Passed: {}", state);
            let error_msg = format!("{{\"invalid_state\": \"{}\"}}", state);
            return Err(rocket::response::status::Custom(
                Status::BadRequest,
                error_msg,
            ));
        }
    };

    let waiting_user_filter: bson::Document = doc! {
        "profile_status": state
    };
    let streamer_cursor = (&*db)
        .database("cbutil")
        .collection::<Streamer>("streamers")
        .find(waiting_user_filter, None)
        .await;
    match streamer_cursor {
        Ok(cursor) => {
            if let Ok(streamers) = cursor.try_collect().await {
                return Ok(Json(streamers));
            }
        }
        Err(e) => {
            return Err(rocket::response::status::Custom(
                Status::InternalServerError,
                e.to_string(),
            ));
        }
    };
    return Err(rocket::response::status::Custom(
        Status::InternalServerError,
        "Woah".to_string(),
    ));
}

#[get("/users/<user>")]
async fn retrieve_user(
    db: Connection<StreamersDB>,
    user: &str,
) -> Result<Json<Streamer>, rocket::response::status::Custom<String>> {
    let user_option = (&*db)
        .database("cbutil")
        .collection::<Streamer>("streamers")
        .find_one(
            doc! {
                "profile_name": user
            },
            None,
        )
        .await;
    match user_option {
        Ok(opt) => {
            if let Some(streamer) = opt {
                println!("Found User {}", user);
                return Ok(Json(streamer));
            } else {
                let err_string = format!("User: {} is not being tracked", user);
                return Err(rocket::response::status::Custom(
                    Status::NotFound,
                    err_string,
                ));
            }
        }
        Err(e) => {
            return Err(rocket::response::status::Custom(
                Status::InternalServerError,
                e.to_string(),
            ));
        }
    };
}

#[put("/users/<user>")]
async fn add_user(
    db: Connection<StreamersDB>,
    user: &str,
) -> Result<Json<StreamerState>, rocket::response::status::Custom<String>> {
    if let Ok(user_count) = (&*db)
        .database("cbutil")
        .collection::<Streamer>("streamers")
        .count_documents(
            doc! {
                "profile_name": user
            },
            None,
        )
        .await
    {
        if user_count > 0 {
            let err_string = format!("User: {} already exists!", user);
            return Err(rocket::response::status::Custom(
                Status::Conflict,
                err_string,
            ));
        } else {
            println!("Adding New User: {}", user);
        }
    } else {
        return Err(rocket::response::status::Custom(
            Status::InternalServerError,
            String::from("Could Not Retrieve Streamer Info"),
        ));
    }

    let new_user = Streamer {
        profile_url: format!("https://chaturbate.com/{}", user),
        profile_name: String::from(user),
        profile_status: StreamerState::Waiting,
        download_size_mb: 0,
    };
    if let Ok(_res) = (&*db)
        .database("cbutil")
        .collection::<Streamer>("streamers")
        .insert_one(&new_user, None)
        .await
    {
        Ok(Json(StreamerState::Waiting))
    } else {
        let err_string = format!("Could Not Add User {}", user);
        Err(rocket::response::status::Custom(
            Status::InternalServerError,
            err_string,
        ))
    }
}

#[delete("/users/<user>")]
async fn delete_user(
    db: Connection<StreamersDB>,
    user: &str,
) -> Result<Json<String>, rocket::response::status::Custom<String>> {
    match (&*db)
        .database("cbutil")
        .collection::<Streamer>("streamers")
        .count_documents(
            doc! {
                "profile_name": user
            },
            None,
        )
        .await
    {
        Ok(count) => {
            if count != 1 {
                return Err(rocket::response::status::Custom(
                    Status::NotFound,
                    format!("User {} does not exist to be deleted.", user),
                ));
            }
            if let Err(delete_err) = (&*db)
                .database("cbutil")
                .collection::<Streamer>("streamers")
                .delete_one(doc! {"profile_name": user}, None)
                .await
            {
                return Err(rocket::response::status::Custom(
                    Status::InternalServerError,
                    delete_err.to_string(),
                ));
            } else {
                println!("Deleting User: {}", user);
                return Ok(Json(format!("Deleted {}", user)));
            }
        }
        Err(e) => {
            return Err(rocket::response::status::Custom(
                Status::InternalServerError,
                e.to_string(),
            ));
        }
    };
}

#[post("/users/<user>", data = "<msg>")]
async fn mutate_user(
    db: Connection<StreamersDB>,
    user: &str,
    msg: Json<StreamerUpdateMessage>,
) -> Result<Json<String>, rocket::response::status::Custom<String>> {
    let update = msg.into_inner();
    let updated_user = Streamer {
        profile_url: format!("https://chaturbate.com/{}", user),
        profile_name: String::from(user),
        profile_status: update.profile_status.clone(),
        download_size_mb: update.download_size_mb,
    };
    match (&*db)
        .database("cbutil")
        .collection::<Streamer>("streamers")
        .count_documents(
            doc! {
                "profile_name": user
            },
            None,
        )
        .await
    {
        Ok(count) => {
            if count != 1 {
                return Err(rocket::response::status::Custom(
                    Status::NotFound,
                    format!("User {} does not exist to be updated.", user),
                ));
            }
            if let Err(update_err) = (&*db)
                .database("cbutil")
                .collection::<Streamer>("streamers")
                .replace_one(doc! {"profile_name": user}, updated_user, None)
                .await
            {
                return Err(rocket::response::status::Custom(
                    Status::InternalServerError,
                    update_err.to_string(),
                ));
            } else {
                println!("Updating User: {} to {:?}", user, update.profile_status);
                return Ok(Json(format!(
                    "Updated {} to {:?}",
                    user, update.profile_status
                )));
            }
        }
        Err(e) => {
            return Err(rocket::response::status::Custom(
                Status::InternalServerError,
                e.to_string(),
            ));
        }
    };
}

#[post("/users", data = "<msg>")]
async fn mutate_all_users(
    db: Connection<StreamersDB>,
    msg: Json<StreamerUpdateMessage>,
) -> Result<Json<String>, rocket::response::status::Custom<String>> {
    let update_state = msg.into_inner();
    let streamer_cursor = (&*db)
        .database("cbutil")
        .collection::<Streamer>("streamers")
        .find(None, None)
        .await;
    match streamer_cursor {
        Ok(mut cursor) => {
            while let Some(streamer_doc) = cursor.next().await {
                let streamer_profile = streamer_doc.expect("invalid user doc retrieved from mongo");
                let updated_user = Streamer {
                    profile_url: streamer_profile.profile_url.clone(),
                    profile_name: streamer_profile.profile_name.clone(),
                    profile_status: update_state.profile_status.clone(),
                    download_size_mb: 0,
                };
                if let Err(update_err) = (&*db)
                    .database("cbutil")
                    .collection::<Streamer>("streamers")
                    .replace_one(
                        doc! {"profile_name": streamer_profile.profile_name},
                        updated_user,
                        None,
                    )
                    .await
                {
                    return Err(rocket::response::status::Custom(
                        Status::InternalServerError,
                        update_err.to_string(),
                    ));
                }
            }
            println!(
                "updated all user state to {:?}",
                update_state.profile_status
            );
            return Ok(Json(String::from("successfully updated all users' state")));
        }
        Err(e) => {
            return Err(rocket::response::status::Custom(
                Status::InternalServerError,
                e.to_string(),
            ));
        }
    };
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .attach(StreamersDB::init())
        .mount(
            "/",
            routes![
                retrieve_users,
                retrieve_stateful_users,
                retrieve_user,
                add_user,
                delete_user,
                mutate_user,
                mutate_all_users,
            ],
        )
        .launch()
        .await?;
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
        assert_eq!(
            r#"{"profile_url":"https://chaturbate.com/ehotlovea","profile_name":"ehotlovea","profile_status":"stopped","download_size_mb":0}"#,
            stopped_json
        );
        assert_eq!(
            r#"{"profile_url":"https://chaturbate.com/killpretty","profile_name":"killpretty","profile_status":{"error":100},"download_size_mb":0}"#,
            error_json
        );
    }
}
