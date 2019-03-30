use crate::github::api::*;
use crate::github::push_event::PushEvent;
use crate::AIChannResult;

pub fn exec(json: serde_json::Value) -> AIChannResult {
    let push_event: PushEvent = serde_json::from_value(json)?;
    let repository = push_event.repository;

    let pull_request_numbers = fetch_all_pulls_numbers(&repository)?;

    for number in pull_request_numbers {
        let pull = fetch_pull_request(number, &repository)?;

        if let Some(false) = pull.mergeable {
            add_label(number, &repository, vec!["unmergeable"])?;
        }
    }

    Ok(())
}
