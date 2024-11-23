use type_lib::EnvState;
use execution_lib::interpret_code;
use std::fs;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut current_dir =
        env::current_dir().expect("something went wrong reading the current directory");

    //this will be the entry point of the cli
    match args.get(1).unwrap().as_str() {
        "run" => {
            //just run files ending with .thor

            if let Some(filename) = args.get(2) {
                let mut filename = filename.clone().to_owned();

                //tries to append the .thor filetype to allow for only putting in the filename
                if filename.contains(".thor") {
                } else {
                    filename += ".thor";
                }

                current_dir.push(filename);
                let file_dir = current_dir.clone();
                //remove
                current_dir.pop();

                //when i introduce methods like "get_file" or something i need to have access to
                //the environment of
                //1. the executable
                //2. the file thats run
                let env = EnvState { path: current_dir };

                let file_text = fs::read_to_string(file_dir).expect("no such file found");

                interpret_code(file_text, env);
            }
        }
        _ => {}
    }
}


