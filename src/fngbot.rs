/*
 * Copyright (c) 2023-2024 Marcus Butler
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use actix_web::{error, http::header::ContentType, post, web, Error, HttpRequest, HttpResponse};
use hmac::{Hmac, Mac};
use lettre::message::header::ContentType as LettreContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use tracing::{info,debug,trace};

#[derive(Serialize, Deserialize, Debug)]
struct FngbotRequest {
    trigger_id: Option<String>,
    payload: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FngbotRequestPayload {
    payload: Option<String>,
    trigger_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct FngbotResponse {
    code: u16,
    status: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct FngbotSlackJSON {
    #[serde(rename = "type")]
    payload_type: Option<String>,
    trigger_id: Option<String>,
    view: Option<FngbotSlackViewJSON>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FngbotSlackViewJSON {
    blocks: Option<Vec<FngbotSlackBlockJSON>>,
    state: Option<FngbotSlackStateJSON>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FngbotSlackBlockJSON {
    block_id: Option<String>,
    label: Option<FngbotSlackBlockLabelJSON>,
    element: Option<FngbotSlackBlockElementJSON>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FngbotSlackBlockLabelJSON {
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct FngbotSlackBlockElementJSON {
    action_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct FngbotSlackStateJSON {
    values: HashMap<String, HashMap<String, FngbotSlackStateValueJSON>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FngbotSlackStateValueJSON {
    value: String,
}

enum MailChimpStatus {
    Success,
    Failure,
}

enum SlackInviteStatus {
    Success,
    Failure,
}

enum WelcomeStatus {
    Success,
    Failure,
}

type HmacSha256 = Hmac<Sha256>;

#[post("/fngbot/")]
async fn fngbot_service(
    body: web::Bytes,
    state: web::Data<crate::AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    trace!("Data structures for fngbot_service: {body:?} {req:?} {:?}", &state.config);

    let data: HashMap<String, String> = serde_urlencoded::from_bytes(&body).expect("Decoding error...");

    let body_str = std::str::from_utf8(&body).expect("Decoding error");

    let slack_signature = if let Some(signature) = req.headers().get("X-Slack-Signature") {
        debug!("X-Slack-Signature: {signature:?}");
        signature.to_str().expect("")
    } else {
        return Err(error::ErrorBadRequest("X-Slack-Signature header missing; cannot validate message"));
    };

    let slack_timestamp = if let Some(timestamp) = req.headers().get("X-Slack-Request-Timestamp") {
        debug!("X-Slack-Request-Timestamp: {timestamp:?}");
        timestamp.to_str().expect("")
    } else {
        return Err(error::ErrorBadRequest("X-Slack-Timestamp header missing; cannot validate message"));
    };

    // Validate the Slack message.
    if !validate_signature(&state, body_str, slack_signature, slack_timestamp) {
        info!("Unable to validate Slack signature. Dropping request.");
        return Err(error::ErrorBadRequest("Invalid HMAC-SHA256 Signature."));
    }

    // We need to distinguish between three incoming requests, all of which will come to the
    // same URI:
    //
    // 1 - A request made via a slash action for the modal dialog will contain a trigger_id
    // field in the POST data, but no payload field.
    //
    // 2 - A request made via a button on the app home page to launch the modal dialog will
    // contain a payload field; the type will be set to block_actions and the trigger_id will
    // be in the JSON payload.
    //
    // 3 - A request with user-submitted form data will have a payload field, which will
    // contain the form data.
    if let Some(trigger_id) = data.get("trigger_id") {
        debug!("Send view modal for slash command invocation {trigger_id}");
        return fngbot_display(&state, trigger_id.to_string()).await;
    } else if let Some(payload) = data.get("payload") {
        if let Ok(json) = serde_json::from_str::<FngbotSlackJSON>(payload) {
            debug!("Parsed json... {json:?}");
            if json.payload_type == Some(String::from("block_actions")) {
                if let Some(trigger) = json.trigger_id {
                    return fngbot_display(&state, trigger.to_string()).await;
                }
            } else if json.payload_type == Some(String::from("view_submission")) {
                return fngbot_view_submission(state, json).await;
            }
        }
    }
    Err(error::ErrorBadRequest("Unknown Request"))
}

async fn fngbot_display(
    state: &web::Data<crate::AppState>,
    trigger_id: String,
) -> Result<HttpResponse, Error> {
    let client = reqwest::Client::new();

    let payload = format!(
        r#"{{
             "trigger_id": "{}",
             "view": {{
               "title": {{
                 "type": "plain_text",
                 "text": "FNG Bot",
                 "emoji": true
               }},
               "submit": {{
                 "type": "plain_text",
                 "text": "Invite!",
                 "emoji": true
               }},
               "type": "modal",
               "blocks": [
                 {{
                   "type": "input",
                   "element": {{
                     "type": "plain_text_input"
                   }},
                   "label": {{
                     "type": "plain_text",
                             "text": "F3 Name",
                             "emoji": false
                   }}
                 }},
                 {{
                   "type": "input",
                   "element": {{
                     "type": "plain_text_input"
                   }},
                   "label": {{
                     "type": "plain_text",
                     "text": "Hospital Name",
                     "emoji": false
                   }}
                 }},
                 {{
                   "type": "input",
                   "element": {{
                     "type": "email_text_input"
                   }},
                   "label": {{
                     "type": "plain_text",
                     "text": "Email Address",
                     "emoji": false
                   }}
                 }},
                 {{
                   "type": "input",
                   "element": {{
                     "type": "number_input",
                     "is_decimal_allowed": false,
                     "action_id": "number_input-action"
                   }},
                   "label": {{
                     "type": "plain_text",
                     "text": "Cell Phone",
                     "emoji": false
                   }}
                 }}
               ]
             }}
           }}"#, trigger_id);

    trace!("Formatted payload for display call: {payload:?}");

    let res = client
        .post("https://slack.com/api/views.open")
        .body(payload)
        .header("Content-type", "application/json")
        .header(
            "Authorization",
            format!("Bearer {}", state.config.slack_api_key),
        )
        .header("Pragma", "No-Cache")
        .header(
            "Cache-Control",
            "private, no-cache, no-store, must-revalidate",
        )
        .send()
        .await;

    debug!("Response from call to https://slack.com/api/views.open: {res:?}");

    if let Ok(_res) = res {
        Ok(HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body(""))
    } else {
        Err(error::ErrorBadRequest(format!(
            "Error sending view open request: {res:?}"
        )))
    }
}

async fn fngbot_view_submission(
    state: web::Data<crate::AppState>,
    json: FngbotSlackJSON,
) -> Result<HttpResponse, Error> {
    let f3_name: String = if let Some(f3name) = get_slack_field(&json, "F3 Name") {
        f3name
    } else {
        return Err(error::ErrorBadRequest(
            "F3 Name not specified or not in JSON document",
        ));
    };

    let hospital_name: String = if let Some(hospital_name) = get_slack_field(&json, "Hospital Name")
    {
        hospital_name
    } else {
        return Err(error::ErrorBadRequest(
            "Hospital name not specifid or not in JSON document",
        ));
    };

    let email: String = if let Some(email) = get_slack_field(&json, "Email Address") {
        email
    } else {
        return Err(error::ErrorBadRequest("Email Address not specified"));
    };

    let cell: String = if let Some(cell) = get_slack_field(&json, "Cell Phone") {
        cell
    } else {
        return Err(error::ErrorBadRequest("Cell number not provided"));
    };

    debug!("User input field values: {f3_name:?} {hospital_name:?} {email:?} {cell:?}");

    // Add the user to Mailchimp
    let mailchimp_status = match mailchimp(&state, &f3_name, &hospital_name, &email, &cell).await {
        MailChimpStatus::Success => "✅ Success adding user to the mailing list!",
        MailChimpStatus::Failure => "🛑 Could not add user to the mailing list.",
    };

    // Send the user a Slack invite
    let slack_invite_status = match slack_invite(&state, &f3_name, &hospital_name, &email).await {
        SlackInviteStatus::Success => "✅ Success inviting user to Slack!",
        SlackInviteStatus::Failure => "🛑 I could not invite the user to Slack.",
    };

    // Post a message to the welcome crew
    let welcome_status = match welcome_message(
        &state,
        &f3_name,
        &hospital_name,
        &email,
        &cell,
        slack_invite_status,
        mailchimp_status,
    )
    .await
    {
        WelcomeStatus::Success => "✅ Success notifying the welcome team!",
        WelcomeStatus::Failure => "🛑 Could not notify the welcome team.",
    };

    info!("{mailchimp_status}\n{slack_invite_status}\n{welcome_status}");

    // Send the modal response
    let modal_update = format!(
        r#"
{{
  "response_action": "update",
  "view": {{
    "type": "modal",
    "title": {{
      "type": "plain_text",
      "text": "Status"
    }},

    "blocks": [
               {{
                 "type": "section",
                 "text": {{
                           "type": "mrkdwn",
                           "text": "Thanks for using FNG Bot!\n*Status Report*:\n\n✅ I am a robot 🤖!\n\n {mailchimp_status}\n\n{slack_invite_status}\n\n{welcome_status}\n\n"
                 }}
               }}
    ]
  }}
}}"#
    );

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(modal_update))
}

/// Walk the Slack JSON document and extract data fields.
///
/// This *should* be as simple as finding the element for "field" and reading "value" or something
/// similiar, but, oh, no. The first thing you need to do is walk the block array in the view key of
/// the document to find the block and element ids of the data field you are looking for:
///
/// {"type":"view_submission",
///  ...
///  "view": {
///  ...
///     "blocks": [
///         {
/// ...
///             "block_id": "rT6QZ",
///             "label": {
/// ...
///                 "text": "F3 Name", <-- This is the field name you are looking for.
/// ...
///             "element": {
/// ...
///                 "action_id": "XOxu+"
///             }
///         },
///
/// Then, you need to find the block_id and action_id keys in the view -> state dictionary:
///
///      "state": {
///         "values": {
///             "rT6QZ": { <-- Block id from the blocks tables
///                 "XOxu+": { <- Action id from the blocks table
///                     "type": "plain_text_input",
///                     "value": "f3name" <-- This is the data value for the field you are looking for.
///                 }
///             },
///
/// The people who designed Slack's JSON schema are bad, and they should feel bad.
fn get_slack_field(json: &FngbotSlackJSON, field: &str) -> Option<String> {
    if let Some(view) = &json.view {
        if let Some(blocks) = &view.blocks {
            for block in blocks {
                if let Some(ref label) = block.label {
                    if label.text == *field {
                        let block_id = if let Some(block_id) = &block.block_id {
                            block_id
                        } else {
                            return None;
                        };

                        let action_id = if let Some(element) = &block.element {
                            &element.action_id
                        } else {
                            return None;
                        };

                        if let Some(state) = &view.state {
                            if let Some(block_id) = state.values.get(&block_id[..]) {
                                if let Some(action_id) = block_id.get(&action_id[..]) {
                                    return Some(action_id.value.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Every Slack message contains an HMAC-SHA256 signature. This function validates that signature
fn validate_signature(state: &web::Data<crate::AppState>, body: &str, signature: &str, timestamp: &str) -> bool {
    let payload = format!("v0:{timestamp}:{body}");

    trace!("Formatted signature payload: {payload}");

    let mut mac = HmacSha256::new_from_slice(state.config.slack_signing_secret.as_bytes()).expect("?!?");
    mac.update(payload.as_bytes());
    let res = mac.finalize();

    trace!("Formatted hash: {:?}", &res.clone().into_bytes());

    let computed_mac = format!("v0={}", hex::encode(res.into_bytes()));

    debug!("Signature: {signature} Calculated MAC: {computed_mac} result: {}", signature == computed_mac);

    signature == computed_mac
}

async fn mailchimp(
    state: &web::Data<crate::AppState>,
    f3_name: &String,
    hospital_name: &String,
    email: &String,
    cell: &String,
) -> MailChimpStatus {
    let client = reqwest::Client::new();

    let res = client
        .post(format!(
            "https://{}/3.0/lists/{}?skip_merge_validation=true",
            state.config.mailchimp_api_endpoint, state.config.mailchimp_list_id,
        ))
        .body(format!(
            r#"{{"members": [{{"email_address": "{}",
                                 "status": "subscribed",
                                 "email_type": "html",
                                 "merge_fields": {{"FULLNAME": "{}",
                                                  "PHONE": "{}",
                                                  "F3NAME": "{}"
                                                 }}
                               }}],
                    "sync_tags": false,
                    "update_existing": false
                   }}"#,
            email, hospital_name, cell, f3_name
        ))
        .basic_auth("anystring", Some(state.config.mailchimp_api_key.clone()))
        .send()
        .await;

    if let Ok(res) = res {
        debug!("Mailchimp result: {res:?}");
        MailChimpStatus::Success
    } else {
        debug!("Mailchimp error: {res:?}");
        MailChimpStatus::Failure
    }
}

async fn slack_invite(
    state: &web::Data<crate::AppState>,
    f3_name: &String,
    hospital_name: &String,
    email: &String,
) -> SlackInviteStatus {
    let msg = Message::builder()
        .from(
            format!(
                "{} <{}>",
                state.config.f3_region, state.config.email_sender_address
            )
            .parse()
            .unwrap(),
        )
        .reply_to(
            format!(
                "{} <{}>",
                state.config.f3_region, state.config.email_reply_to_address
            )
            .parse()
            .unwrap(),
        )
        .to(format!("{hospital_name} <{email}>").parse().unwrap())
        .subject(format!("Join {} on Slack", state.config.f3_region))
        .header(LettreContentType::TEXT_HTML)
        .body(format!(
            r#"Hi {f3_name}! Please join {} on Slack by clicking this <a href="{}">link</a>."#,
            state.config.f3_region, state.config.slack_invite_link,
        ))
        .unwrap();

    let mailer = if let Some(user) = &state.config.email_smtp_user {
        let bind_pass: String;

        let pass = if let Some(pass) = &state.config.email_smtp_pass {
            pass
        } else {
            bind_pass = String::from("");
            &bind_pass
        };

        SmtpTransport::relay(&state.config.email_smtp_host)
            .unwrap()
            .credentials(Credentials::new(user.to_owned(), pass.to_owned()))
            .build()
    } else {
        SmtpTransport::relay(&state.config.email_smtp_host)
            .unwrap()
            .build()
    };

    // Send the email
    match mailer.send(&msg) {
        Ok(res) => {
            debug!("Success sending email to {email}: {res:?}");
            SlackInviteStatus::Success
        },
        Err(e) => {
            debug!("Error sending email to {email}: {e:?}");
            SlackInviteStatus::Failure
        }
    }
}

async fn welcome_message(
    state: &web::Data<crate::AppState>,
    f3_name: &String,
    hospital_name: &String,
    email: &String,
    cell: &String,
    slack_invite_status: &str,
    mailchimp_status: &str,
) -> WelcomeStatus {
    let client = reqwest::Client::new();

    let welcome_msg = format!(
        r#"Hi welcome team! A new FNG just posted.  Their contact info is:
F3 Name: {f3_name}
Hospital Name: {hospital_name}
Email Address: {email}
Cell Phone: {cell}

Here are the results of inviting them to slack and adding them to Mailchimp:
{slack_invite_status}
{mailchimp_status}"#
    );

    let res = client
        .post("https://slack.com/api/chat.postMessage")
        .body(format!(
            r#"{{
                "channel": "{}",
                "text": "{}"
               }}"#,
            state.config.slack_channel_id, welcome_msg
        ))
        .header("Content-type", "application/json")
        .header(
            "Authorization",
            format!("Bearer {}", state.config.slack_api_key),
        )
        .send()
        .await;

    debug!("Welcome Message result: {res:?}");

    match res {
        Ok(_) => WelcomeStatus::Success,
        Err(e) => {
            debug!("Error posting welcome message: {e:?}");
            WelcomeStatus::Failure
        }
    }
}
