
use std::thread;
use std::time::Duration;
use std::fs::{File, OpenOptions};
use std::process::Command;
use std::io::{Read, Write};

use conf::Config;
use time::get_time;
use item::ItemKind;

pub fn start(mut conf: Config) {
    // We would deamonize here if necessary

    loop {
        loop {
            if let Some(c) = conf.items.peek() {
                if c.next_time > get_time().sec {
                    break;
                }
            } else {
                break;
            }


            let mut item = conf.items.pop().unwrap();
            let clone = item.clone();
            item.next_time = get_time().sec + item.interval;
            conf.items.push(item);

            let mut shell = String::new();

            let mut output_folder = conf.output.clone();

            if let ItemKind::Shell(_) = clone.kind {
                shell = conf.general.shell.clone();
            }

            thread::spawn(move || {
                let mut result = String::new();
                match clone.kind {
                    ItemKind::File(ref path) => {
                        let mut f = File::open(path).unwrap();
                        f.read_to_string(&mut result).unwrap();
                    }
                    ItemKind::Command(ref path, ref args) => {
                        let mut output = Command::new(path);
                        output.args(args);
                        for (k,v) in clone.env {
                            output.env(k, v);
                        }
                        let output = output.output().unwrap();
                        result = String::from_utf8(output.stdout).unwrap();
                    }
                    ItemKind::Shell(ref command) => {
                        let mut output = Command::new(shell);
                        output.arg("-c");
                        output.arg(command);
                        for (k,v) in clone.env {
                            output.env(k, v);
                        }
                        let output = output.output().unwrap();
                        result = String::from_utf8(output.stdout).unwrap();
                    }
                }
                debug!("{}={}", clone.key, result);
                output_folder.push(clone.key);
                match OpenOptions::new().append(true).open(output_folder)
                    .and_then(|mut file| file.write(&result.as_bytes()[..]))
                    {
                        Ok(_) => (),
                        Err(e) => {
                            debug!("{}", e)
                        }
                    }
            });
        }
        if let Some(c) = conf.items.peek() {
            thread::sleep(Duration::from_secs((c.next_time - get_time().sec) as u64));
        }
    }
}

