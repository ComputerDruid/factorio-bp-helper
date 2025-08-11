use serde_json::json;

const QUALITIES: [&str; 6] = [
    "normal",
    "uncommon",
    "rare",
    "epic",
    "legendary",
    "legendary",
];

fn upgrade_quality(thing: &mut serde_json::Value) {
    if let Some(quality) = thing.get_mut("quality") {
        let s = quality.as_str().unwrap();
        let new_quality = QUALITIES
            .windows(2)
            .find(|pair| pair[0] == s)
            .unwrap_or_else(|| panic!("can't find quality {s}"))[1];
        *quality = json!(new_quality);
    } else {
        // TODO handle normal
    }
}

fn upgrade_circuit_condition(circuit_condition: &mut serde_json::Value) {
    if let Some(first_signal) = circuit_condition.get_mut("first_signal") {
        upgrade_quality(first_signal);
    }
    // TODO what about second signal?
}

pub(crate) fn upgrade(mut json: serde_json::Value) -> serde_json::Value {
    let bp = json
        .get_mut("blueprint")
        .expect("blueprint books not supported yet");
    let entities = bp.get_mut("entities").unwrap().as_array_mut().unwrap();
    for entity in entities {
        if let Some(control_behavior) = entity.get_mut("control_behavior") {
            if let Some(circuit_condition) = control_behavior.get_mut("circuit_condition") {
                upgrade_circuit_condition(circuit_condition);
            }
            if let Some(logistic_condition) = control_behavior.get_mut("logistic_condition") {
                upgrade_circuit_condition(logistic_condition);
            }
        }
        if let Some(filter) = entity.get_mut("filter") {
            upgrade_quality(filter);
        }
        if let Some(filters) = entity.get_mut("filters") {
            for filter in filters.as_array_mut().unwrap() {
                upgrade_quality(filter);
            }
        }
        if let Some(quality) = entity.get_mut("recipe_quality") {
            let s = quality.as_str().unwrap();
            let new_quality = QUALITIES.windows(2).find(|pair| pair[0] == s).unwrap()[1];
            *quality = json!(new_quality);
        } else {
            // TODO figure out what things are supposed to have a recipe quality when the recipe is normal
        }
        if let Some(request_filters) = entity.get_mut("request_filters") {
            if let Some(sections) = request_filters.get_mut("sections") {
                for section in sections.as_array_mut().unwrap() {
                    if let Some(filters) = section.get_mut("filters") {
                        for filter in filters.as_array_mut().unwrap() {
                            upgrade_quality(filter);
                        }
                    }
                }
            }
        }
    }
    json
}
