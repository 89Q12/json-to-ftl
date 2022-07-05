use std::fs;
use std::collections::HashMap;

// A designator to specify where a variable belongs in this key/value. (e.g: for a templating engine)
static VAR_DESIGNATOR: &str = "${x}";

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
        ("x_is_live",                                              "live_upload_text"),
        ("x_ago",                                                  "upload_date"),
        ("norwegian_bokmål",                                       "norwegian"),
        ("%a_%b_%_d_%y",                                           "WTF"),
        ("x_marked_it_with_a_",                                    "like"),
        (r#"[^0_9]|^1[^0_9]|$"#,                                   "view_comments"),
        (r#"[^0_9]^1[^0_9]"#,                                      "view_comments")
    ]);
    fluent_rebinds
}

// Safely rebind all fluent keys into a more 'sane' format via HashMap, if no rebind is available: the old key is used.
fn generate_ftl_rebindings(input: &Vec<String>) -> HashMap<String, String> {
    // Fetch rebindings and prep the string map
    let fluent_rebinds = get_rebindings();
    let mut keymap = HashMap::<String, String>::new();
    // Loop every line of the FTL file
    for line in input {
        if line.is_empty() { continue; }
        // Extract the key manually (can't use split, as `=` is also used within values sometimes)
        let mut key: String = String::new();
        for c in line.chars() {
            if c == '=' { break; }
            key += &c.to_string();
        }
        keymap.insert(key.clone(), fluent_rebinds.get(&key.as_str()).unwrap_or(&key.as_str()).to_string());
    }

    keymap
}

// Parse a JSON invidious lang file into FTL format
fn ftl_parse(input: &str) -> String {
    let mut result = String::new();

    // Global replace a bunch of constant formatting
    // Blame invidious' terrible formatting for this monster of a replacer chain!
    let rinput = input.replace("  ", " ")  // Replace any double-spaces with single spaces
                      .trim()              // Trim JSON tabs
                      .replace(" `x`", format!(" {}", VAR_DESIGNATOR).as_str()) // Replace strangely-spaced `x` with a clear variable designator
                      .replace("`x`", VAR_DESIGNATOR)  // Replace `x` with a clear variable designator
                      .replace("\":", "=") // Replace : syntax with =
                      .replace("\"", "");  // Remove escaped quotes for single backlashes

    // Keep track if we're within the Key or Value
    let mut is_in_value = false;
    // Loop every char of the JSON keypair
    for c in rinput.chars() {
        if c == ' ' && !is_in_value {
            // Replace spaces with underscores
            result.push('_');
        } else {
            // Keep any non-space char
            // If we're still within the key: convert to lowercase too
            if is_in_value {
                match c {
                    '=' => result.push(':'), // Due to replace(":", "=") we need to now replace = with : again.
                    _ => result.push(c),
                }
            } else {
                match c {
                    '(' => continue,
                    ')' => continue,
                    '/' => continue,
                    '\u{005C}' => continue,
                    '.' => continue,
                    '?' => continue,
                    ',' => continue,
                    '&' => continue,
                    '!' => continue,
                    '$' => continue,
                    '{' => continue,
                    '}' => continue,
                    '|' => continue,
                    '\u{2764}' => continue,
                    '\u{0027}' => continue,
                    '-' => result.push('_'),
                    _ => result.push(c.to_lowercase().to_string().chars().next().unwrap()),
                }
            }
        }

        // Cancel any replaces after the equals sign
        if c == '=' {
            is_in_value = true;
        }
    }

    // Run a couple post-processing replaces and return!
    result.replace(":_", "")   // Trim unnecessary key endings
          .replace(".=", "=")  // Trim unnecessary periods
          .replace("_-_", "_") // Replace _-_ with a single underscore
}

fn main() {
    // Loop all files in the current directory, parsing only '.json' files
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_name = entry.file_name().into_string().unwrap();
                if entry_name.ends_with(".json") {
                    // Read JSON translations
                    let file = &fs::read(&entry_name).expect("This should never happen! Whoops.");
                    let input: &str = &String::from_utf8_lossy(file);

                    // Split into lines
                    let vec_lines = input.split("\n");

                    // Parse each individual line
                    let mut parsed_vec_lines: Vec<String> = Vec::new();
                    for line in vec_lines {
                        // Ignore lines with open/close brackets
                        if line.contains("{") || line.contains("}") { continue };
                        // Parse'n'check'n'push!
                        let parsed_line = ftl_parse(line);
                        // No empty/malformed lines!
                        if parsed_line.starts_with("=") || parsed_line.is_empty() { continue };
                        parsed_vec_lines.push(parsed_line);
                    }

                    // Compile into a single string file
                    let mut parsed_fluent_file = parsed_vec_lines.join("\n");

                    // Generate rebindings from parsed lines
                    let rebindings = generate_ftl_rebindings(&parsed_vec_lines);
                    for old_key in rebindings.keys() {
                        // Apply to the final string file
                        parsed_fluent_file = parsed_fluent_file.replace(old_key, rebindings.get(old_key).unwrap_or(old_key));
                    }

                    // Ensure the relevent directory exists
                    if fs::read_dir(entry_name.split(".").collect::<Vec<&str>>().get(0).unwrap()).is_err() {
                        fs::create_dir(entry_name.split(".").collect::<Vec<&str>>().get(0).unwrap()).unwrap();
                    }

                    // Write new version to disk
                    fs::write(format!("{}/basic.ftl", entry_name.split(".").collect::<Vec<&str>>().get(0).unwrap()), parsed_fluent_file).unwrap();
                    println!("{}: Parsed {} lines into proper .ftl format!", entry_name, &parsed_vec_lines.len());
                }
            }
        }
    }
}
