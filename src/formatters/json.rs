use serde_json::Value;

pub fn print_formatted_json(json_output: &String){
    // TODO: remonter le from_str
    match serde_json::from_str::<Value>(json_output){
        Ok(converted_json_output) => {
            match serde_json::to_string_pretty(&converted_json_output){
                Ok(pretty_json) => {
                    print!("{}", pretty_json);
                }
                Err(..) => {
                    // if formatting fails we print the original version
                    println!("{}", json_output);
                }
            }
        }
        Err(e) => {
            eprintln!("Error for serializing JSON: {}", e);
        }
    }
}