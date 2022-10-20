use std::collections::HashMap;
use std::fs;

use serde_json::Value;

// A reusable function to fetch the fluent rebinding hashmap
fn get_rebindings() -> HashMap<&'static str, &'static str> {
    let fluent_rebinds: HashMap<&'static str, &'static str> = HashMap::from([
        // Old,  New
        ("hi_looks_like_you_have_javascript_turned_off_click_here_to_view_comments_keep_in_mind_they_may_take_a_bit_longer_to_load", "no_js_comments"),
        ("an_alternative_front_end_to_youtube",                    "alternative_youtube_front-end"),
        ("time_h:mm:ss:",                                          "captcha_time_format"),
        ("quota_exceeded,_try_again_in_a_few_hours",               "quota_exceeded"),
        ("unable_to_log_in_make_sure_two_factor_authentication_authenticator_or_sms_is_turned_on",       "unable_to_login"),
        ("login_failed_this_may_be_because_two_factor_authentication_is_not_turned_on_for_your_account", "login_failed"),
        ("please_sign_in_using_log_in_with_google",                "sign_in_using_google"),
        ("password_cannot_be_longer_than_55_characters",           "password_too_long"),
        ("token_is_expired_please_try_again",                      "token_expired"),
        ("family_friendly_",                                       "family_friendly"),
        ("x_uploaded_a_video",                                     "upload_text"),
        ("x_is_live",                                              "channel_is_live"),
        ("x_ago",                                                  "upload_date"),
        ("norwegian_bokmål",                                       "norwegian"),
        ("%a_%b_%_d_%y",                                           "WTF"),
        ("x_marked_it_with_a_",                                    "liked"),
        (r#"[^0_9]|^1[^0_9]|$"#,                                   "view_comments"),
        (r#"[^0_9]^1[^0_9]"#,                                      "view_comments"),
        (r#"password_cannot_be_longer_than_55_characters"#,        "password_too_long"),
        (r#"adminprefs_modified_source_code_url_label"#,           "modified_source_code_url_label"),
        (r#"edited"#,                                              "comment_edited"),
        (r#"alternative_youtube_front-end"#,                       "about_project"),
        (r#"export_subscriptions_as_opml_for_newpipe__freetube"#,  "export_subscriptions_as_opml_for_other_projects"),
        (r#"hidden_field_"challenge"_is_a_required_field"#,        "challenge_is_required"),
        (r#"hidden_field_"token"_is_a_required_field"#,            "csrf_token_is_required"),
    ]);
    fluent_rebinds
}

// Parse a JSON invidious lang file into FTL format
fn ftl_parse(input: &str) -> String {
    let mut result = String::new();

    // Global replace a bunch of constant formatting
    // Blame invidious' terrible formatting for this monster of a replacer chain!
    let rinput = input
        .replace("  ", " ") // Replace any double-spaces with single spaces
        .trim() // Trim JSON tabs
        .replace(" `x`", "_x") // Replace strangely-spaced `x` with a clear variable designator
        .replace("`x`", "x") // Replace `x` with a clear variable designator
        .replace("\":", "")
        .replace(" - ", " ")
        .replace("/", " ")
        .replace("-", "_");
    // Loop every char of the JSON keypair
    for c in rinput.chars() {
        // Keep any non-space char
        match c {
            '(' => continue,
            ')' => continue,
            '/' => continue,
            '\u{005C}' => continue,
            '.' => continue,
            '?' => result.push_str("_question"),
            ',' => continue,
            '&' => continue,
            '!' => continue,
            '$' => continue,
            '{' => continue,
            '}' => continue,
            '|' => continue,
            ':' => continue,
            '\u{2764}' => continue,
            '\u{0027}' => continue,
            '-' => result.push('_'),
            ' ' => result.push('_'),
            _ => result.push(c.to_lowercase().to_string().chars().next().unwrap()),
        }
    }

    // Run a couple post-processing replaces and return!
    result
}

fn main() {
    let mut total_lines: i32 = 0;
    // Loop all files in the current directory, parsing only '.json' files
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_name = entry.file_name().into_string().unwrap();
                if entry_name.ends_with(".json") {
                    let fluent_rebinds = get_rebindings();
                    // Read JSON translations
                    let file = &fs::read(&entry_name).expect("This should never happen! Whoops.");
                    let input: &str = &String::from_utf8_lossy(file);
                    let mut keys = HashMap::<String,u8>::new();
                    // Split into lines
                    let value: Value = serde_json::from_str(input).unwrap();
                    // Parse each individual line
                    let mut parsed_vec_lines: Vec<String> = Vec::new();
                    for line in value.as_object().unwrap().keys() {
                        if keys.contains_key(line){
                            continue;
                        }
                        keys.insert(line.to_string(), 1);
                        let mut mutated_line = ftl_parse(line);
                        if fluent_rebinds.contains_key(&mutated_line.as_str()) {
                            mutated_line = fluent_rebinds
                                .get(&mutated_line.as_str())
                                .unwrap()
                                .to_string();
                        }
                        println!("{}", mutated_line);
                        println!(
                            "{}",
                            value[line]
                                .to_string()
                                .replace("`x`", "{ $x }")
                                .replace(" `x`", " { $x }")
                                .replace("{{count}}", "{ $x }")
                        );
                        if value[line].is_object() {
                            parsed_vec_lines.push(
                                mutated_line
                                    + "="
                                    + &value[line].as_object().unwrap()[""]
                                        .as_str()
                                        .unwrap()
                                        .replace("`x`", "{ $x }")
                                        .replace(" `x`", " { $x }"),
                            );
                            continue;
                        }
                        parsed_vec_lines.push(
                            mutated_line
                                + "="
                                + &value[line]
                                    .as_str()
                                    .unwrap()
                                    .replace("`x`", "{ $x }")
                                    .replace(" `x`", " { $x }")
                                    .replace("{{count}}", "{ $x }"),
                        );
                    }
                    // Compile into a single string file
                    let parsed_fluent_file = parsed_vec_lines.join("\n");

                    // Ensure the relevant directory exists
                    if fs::read_dir(entry_name.split(".").collect::<Vec<&str>>().get(0).unwrap())
                        .is_err()
                    {
                        fs::create_dir(
                            entry_name.split(".").collect::<Vec<&str>>().get(0).unwrap(),
                        )
                        .unwrap();
                    }

                    // Write new version to disk
                    fs::write(
                        format!(
                            "{}/basic.ftl",
                            entry_name.split(".").collect::<Vec<&str>>().get(0).unwrap()
                        ),
                        parsed_fluent_file,
                    )
                    .unwrap();
                    println!(
                        "{}: Parsed {} lines into proper .ftl format!",
                        entry_name,
                        &parsed_vec_lines.len() -1
                    );
                    total_lines += (parsed_vec_lines.len() -1) as i32;
                }
            }
        }
    }
    println!(
        "Parsed a total {} lines into proper .ftl format!", total_lines
    );
}
