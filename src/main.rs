extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate warp;

use std::env;
use std::error::Error;

use warp::Filter;

fn main() -> Result<(), Box<Error>> {
    let port = env::var("PORT")?.parse::<u16>()?;

    let routes = warp::any()
        .and(warp::body::json())
        .and_then(handle_callback);

    warp::serve(routes).run(([127, 0, 0, 1], port));

    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Callback {
    UrlVerification { token: String, challenge: String },
    EventCallback { token: String, event: Event },
}

#[derive(Debug, Deserialize)]
struct Event {
    channel: String,
    ts: String,
    #[serde(default)]
    thread_ts: Option<String>,
    #[serde(default)]
    subtype: Option<String>,
}

#[derive(Serialize)]
struct Message {
    channel: String,
    text: String,
    thread_ts: String,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Response {
    Ok,
    Challenge { challenge: String },
}

fn handle_callback(callback: Callback) -> Result<impl warp::Reply, warp::Rejection> {
    let verification_token =
        env::var("VERIFICATION_TOKEN").map_err(|_| warp::reject::server_error())?;

    let response = match callback {
        Callback::UrlVerification {
            ref token,
            ref challenge,
        }
            if token == &verification_token =>
        {
            Ok(Response::Challenge {
                challenge: challenge.to_string(),
            })
        }
        Callback::UrlVerification { .. } => Err(warp::reject::bad_request()),
        Callback::EventCallback { ref event, .. }
            if event.channel == "C4C959YHF"
                && event.thread_ts.is_none()
                && event.subtype.is_none() =>
        {
            let client = reqwest::Client::new();
            let text = "You might want to post this to <#C07BYTE1W|sea-labs>. This channel isn't widely used by the office.".into();
            let message = Message {
                channel: event.channel.clone(),
                text,
                thread_ts: event.ts.clone(),
            };
            client
                .post("https://slack.com/api/chat.postMessage")
                .json(&message)
                .send()
                .map(|_| Response::Ok)
                .map_err(|_| warp::reject::server_error())
        }
        Callback::EventCallback { .. } => Ok(Response::Ok),
    };

    response.map(|r| warp::reply::json(&r))
}
