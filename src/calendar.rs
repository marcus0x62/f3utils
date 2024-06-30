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

use actix_web::{post, web, HttpRequest};
use mysql::prelude::*;
use mysql::*;
use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;
use tracing::info;

#[derive(Serialize, Deserialize)]
struct CalendarRequest {
    team_id: String,
    start_date: String,
    end_date: String,
}

#[derive(Serialize, Deserialize)]
struct CalendarResponse {
    code: u16,
    status: String,
    responses: Vec<CalendarEntry>,
}

#[derive(Serialize, Deserialize)]
struct CalendarEntry {
    ao: String,
    date: String,
    special: String,
    q: Option<String>,
}

#[post("/calendar/")]
async fn calendar_service(
    data: web::Form<CalendarRequest>,
    state: web::Data<crate::AppState>,
    req: HttpRequest,
) -> web::Json<CalendarResponse> {
    let team_id = ammonia::clean(&data.team_id[..]);
    let start_date = ammonia::clean(&data.start_date[..]);
    let end_date = ammonia::clean(&data.end_date[..]);

    info!("Handling POST request for /calendar/ for client {} (team id: {team_id}, start: {start_date}, end: {end_date})", crate::get_client_ip(&req));

    let pool = Pool::new(&state.db_url[..]).unwrap();
    let mut db_conn = pool.get_conn().unwrap();

    let query = r#"SELECT ao.ao_display_name, qm.event_date, qm.event_special, qm.q_pax_name
                     FROM f3stcharles.qsignups_aos ao
                     LEFT JOIN f3stcharles.qsignups_master qm ON (ao.ao_channel_id = qm.ao_channel_id)
                     WHERE qm.team_id = :team_id AND qm.event_date >= :start_date AND qm.event_date <= :end_date
                     ORDER BY qm.event_date ASC;"#;

    let mut response = CalendarResponse {
        code: 200,
        status: String::from("OK"),
        responses: vec![],
    };

    let statement = db_conn.prep(query).unwrap();
    response.responses = db_conn
        .exec_map(
            statement,
            params! {
                "team_id" => team_id,
                "start_date" => start_date,
                "end_date" => end_date
            },
            |(ao_display_name, event_date, event_special, q_pax_name)| {
                let ao: String;
                if let Ok(row_ao) = from_value_opt::<String>(ao_display_name) {
                    ao = row_ao;
                } else {
                    ao = String::from("ERROR");
                }

                let date: String;
                if let Ok(row_date) = from_value_opt::<PrimitiveDateTime>(event_date) {
                    date = format!("{}", row_date.date());
                } else {
                    date = String::from("ERROR");
                }

                let special: String;
                if let Ok(row_special) = from_value_opt::<String>(event_special) {
                    special = row_special;
                } else {
                    special = String::from("");
                }

                let q: Option<String>;
                if let Ok(row_q) = from_value_opt::<String>(q_pax_name) {
                    q = Some(row_q);
                } else {
                    q = None;
                }

                CalendarEntry {
                    ao,
                    date,
                    special,
                    q,
                }
            },
        )
        .unwrap();
    web::Json(response)
}
