use std::fs;

fn ftl_parse(input: &str) -> String {
    let mut result = String::new();
    let mut replaced = false;

    // Global replace a bunch of constant formatting
    // Blame invidious' terrible formatting for this monster of a replacer chain!
    let rinput = input.replace("  ", " ")  // Replace any double-spaces with single spaces
                      .trim()              // Trim JSON tabs
                      .replace(" `x`", " VARHERE") // Replace strangely-spaced `x` with *nothin!*
                      .replace("`x`", "VARHERE")  // Replace `x` with *nothin!*
                      .replace(":", "=")   // Remove escaped quotes for single backlashes
                      .replace("\"", "");  // Replace : syntax with =

    // Loop every char until 'replaced' is true
    for c in rinput.chars() {
        if c == ' ' && !replaced {
            // Replace spaces with underscores
            result.push('_');
        } else {
            // Keep any non-space char
            // If we're still within the key: convert to lowercase too
            if replaced {
                result.push(c);
            } else {
                result.push(c.to_lowercase().to_string().chars().next().unwrap());
            }
        }

        // Cancel any replaces after the equals sign
        if c == '=' {
            replaced = true;
        }
    }

    result
}

fn main() {
    // Read JSON translations
    let file = &fs::read("lang.json").expect("Slap your JSON locale file into `lang.json` and try again!");
    let input: &str = &String::from_utf8_lossy(file);

    // Split into lines
    let vec_lines = input.split("\n");

    // Parse each individual line
    let mut parsed_vec_lines: Vec<String> = Vec::new();
    for line in vec_lines {
        // Ignore lines with open/close brackets
        if line.contains("{") || line.contains("}") { continue };
        // Parse'n'push!
        parsed_vec_lines.push(ftl_parse(line));
    }

    // Write new version to disk
    fs::write("parsed_lang.ftl", &parsed_vec_lines.join("\n")).unwrap();
    println!("Parsed {} lines into proper .ftl format!", &parsed_vec_lines.len());
}
