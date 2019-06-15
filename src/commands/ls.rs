use crate::errors::ShellError;
use crate::object::{dir_entry_dict, Primitive, Value};
use crate::parser::lexer::Spanned;
use crate::prelude::*;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub fn ls(args: CommandArgs) -> Result<OutputStream, ShellError> {
    let env = args.env.lock().unwrap();
    let path = env.back().unwrap().path.to_path_buf();
    let obj = &env.back().unwrap().obj;
    let mut full_path = PathBuf::from(path);
    match &args.positional.get(0) {
        Some(Spanned {
            item: Value::Primitive(Primitive::String(s)),
            ..
        }) => full_path.push(Path::new(s)),
        _ => {}
    }

    match obj {
        Value::Filesystem => {
            let entries = std::fs::read_dir(&full_path);

            let entries = match entries {
                Err(e) => {
                    if let Some(s) = args.positional.get(0) {
                        return Err(ShellError::labeled_error(
                            e.to_string(),
                            e.to_string(),
                            s.span,
                        ));
                    } else {
                        return Err(ShellError::string(e.to_string()));
                    }
                }
                Ok(o) => o,
            };

            let mut shell_entries = VecDeque::new();

            for entry in entries {
                let value = Value::Object(dir_entry_dict(&entry?)?);
                shell_entries.push_back(ReturnValue::Value(value))
            }
            Ok(shell_entries.boxed())
        }
        _ => {
            let mut entries = VecDeque::new();
            let mut viewed = obj;
            let sep_string = std::path::MAIN_SEPARATOR.to_string();
            let sep = OsStr::new(&sep_string);
            for p in full_path.iter() {
                //let p = p.to_string_lossy();
                match p {
                    x if x == sep => {}
                    step => {
                        let tmp = step.to_string_lossy();
                        let split_tmp = tmp.split('[');
                        let mut first = true;

                        for s in split_tmp {
                            if !first {
                                match s.find(']') {
                                    Some(finish) => {
                                        let idx = s[0..finish].parse::<usize>();
                                        match idx {
                                            Ok(idx) => match viewed.get_data_by_index(idx) {
                                                Some(v) => {
                                                    viewed = v;
                                                }
                                                _ => println!("Idx not found"),
                                            },
                                            _ => println!("idx not a number"),
                                        }
                                    }
                                    _ => println!("obj not some"),
                                    /*
                                    _ => match viewed.get_data_by_key(s) {
                                        Some(v) => {
                                            viewed = v;
                                        }
                                        _ => println!("Obj not Some"),
                                    },
                                    */
                                }
                            } else {
                                match viewed.get_data_by_key(s) {
                                    Some(v) => {
                                        viewed = v;
                                    }
                                    _ => println!("Obj not Some"),
                                }
                                first = false;
                            }
                        }
                    }
                }
            }
            match viewed {
                Value::List(l) => {
                    for item in l {
                        entries.push_back(ReturnValue::Value(item.copy()));
                    }
                }
                x => {
                    entries.push_back(ReturnValue::Value(x.clone()));
                }
            }
            Ok(entries.boxed())
        }
    }
}
