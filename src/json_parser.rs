extern crate json;

use std::error::Error;
use std::fs::File;
use std::io;
use std::io::Error as IoError;
use std::io::ErrorKind::Other as ReadError;
use std::io::prelude::*;
use std::path::Path;

use string_gen::keywords::*;
use raw_code::{RawCode, CodeData, generate_raw};

pub fn process_json_file(filename: &str) -> io::Result<RawCode> {
    let path = Path::new(&filename);
    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(why) => {
            return Err(IoError::new(ReadError, format!("Failed to open {} for reading: {}",
                                                       path.display(), why.description())));
        }
    };

    let mut json_data = String::new();
    try!(file.read_to_string(&mut json_data));

    process_json_str(json_data.as_str())
}

pub fn process_json_str(json_str: &str) -> io::Result<RawCode> {
    let mut code_data   = Vec::<CodeData>::new();
    let mut config_tags = Vec::<CodeData>::new();
    let mut config_data = Vec::<CodeData>::new();
    
    let parsed_json = json::parse(json_str).unwrap();
    for i in parsed_json.entries() {
        if i.0 == CONFIG {
            process(&i, &mut config_tags, 0);
        }
        else {
            process(&i, &mut code_data, 0);
        }
    }

    // process configs, if found
    let mut json_cfg = String::new();
    for c in config_tags.iter() {
        for a in c.1.iter() {
            if a.0 == FILE {
                let path = Path::new(&a.1);
                let mut file = match File::open(&path) {
                    Ok(file) => file,
                    Err(why) => {
                        return Err(IoError::new(ReadError,format!("Failed to open {} for reading: {}",
                                                                  path.display(), why.description())));
                    }
                };
                json_cfg.clear();
                try!(file.read_to_string(&mut json_cfg));

                let parsed_json = json::parse(json_cfg.as_str()).unwrap();
                for i in parsed_json.entries() {
                    process(&i, &mut config_data, 0);
                }
            }
        }
    }

    Ok(generate_raw(&code_data, &config_data))
}

fn process(json: &(&String, &json::JsonValue), data: &mut Vec<CodeData>, depth: u8) {
    data.push((json.0.clone(), Vec::<(String, String)>::new(), depth));
    let mut attribs = Vec::<(String, String)>::new();
    let idx = data.len() - 1;
    
    for i in json.1.entries() {
        if i.1.entries().count() == 0 && i.1.members().count() == 0 {
            attribs.push((i.0.clone(), String::from(i.1.as_str().unwrap())));
        }
        for j in i.1.members() {
            process(&(&i.0, &j), data, depth+1);
        }
        if i.1.entries().count() > 0 {
            process(&i, data, depth+1);
        }
    }

    for j in json.1.members() {
        process(&(&json.0, &j), data, depth);
    }

    // assign collected attributes to element
    data[idx].1 = attribs;

    // if element is only a list of members, remove it from the list
    if json.1.entries().count() == 0 && json.1.members().count() > 0 {
        data.remove(idx);
    }
}
