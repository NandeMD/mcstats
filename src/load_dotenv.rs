use std::fs::File;
use std::io::Read;

use std::collections::HashMap;

use std::env;

fn read_file(env_path: Option<String>) -> Result<String, Box<dyn std::error::Error>> {
    let mut p = String::from("./.env");

    match env_path {
        None => {},
        Some(s) => {p = s}
    }

    let mut string_buf = String::new();

    let mut f = File::open(p)?;
    f.read_to_string(&mut string_buf)?;

    Ok(string_buf)
}

fn parse_env_file(env_file: String) -> HashMap<String, String> {
    let mut vars: HashMap<String, String> = HashMap::new();

    for l in env_file.lines() {
        let mut separated = l.split(" = ");

        if separated.clone().count() == 2 {
            let key = separated.next().unwrap().to_string();
            let val = separated.next().unwrap().to_string();

            vars.insert(key, val);
        }
    }

    vars
}

pub fn load_dotenv(env_path: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let env_file = read_file(env_path)?;
    let env_vars = parse_env_file(env_file);

    for (k, v) in env_vars {
        env::set_var(k, v);
    }

    Ok(())
}

#[cfg(test)]
mod load_env_tests {
    use crate::load_dotenv::load_dotenv;
    use std::env;

    #[test]
    fn check_load_env() {
        load_dotenv(Some(String::from(".dummy_env"))).unwrap();
        assert_eq!(
            env::var("TEST_VAR").unwrap(),
            "NUMNAM"
        )
    }
}