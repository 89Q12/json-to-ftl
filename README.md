# json-to-ftl
A little program to convert json lang files to FTL format used in Fluent for translation, thanks @JSKitty.

This was disgned with language json files from invidious in mind but it should work with most json files.
It also renames the keys to lowercase and replace spaces with `_` if there are any.
It also replaces \`x\` with `VARHERE` to indicate that theres are variable
# How to
- Rename your json file to `lang.json` and put it in the same folder as the script.
- Use rustc `json_ftl_converter.rs` to compile.
- To excute use ./json_ftl_converter and wait for it to finish.
