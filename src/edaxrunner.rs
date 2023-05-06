use std::fs::File;
use std::io::{BufReader, BufRead, Write};

const OBF : &str = "/tmp/test.obf";
const CD : &str = "../../edax-reversi/";
const EDAXPATH : &str = "./bin/lEdax-x64-modern";
const EVFILE : &str = "data/eval.dat";

pub struct EdaxRunner {
    obf : String,
    curdir : String,
    path : String,
    evfile : String
}

impl EdaxRunner {
    pub fn new() -> EdaxRunner {
        EdaxRunner {
            obf: String::from(OBF),
            curdir: String::from(CD),
            path: String::from(EDAXPATH),
            evfile: String::from(EVFILE)
        }
    }

    pub fn config(&mut self, obf : &str, cd : &str, path : &str, evfile : &str)
            -> Result<(), String> {
        if !obf.is_empty() {
            self.obf = String::from(obf);
        }
        if !cd.is_empty() {
            self.curdir = String::from(cd);
        }
        if !path.is_empty() {
            self.path = String::from(path);
        }
        if !evfile.is_empty() {
            self.evfile = String::from(evfile);
        }
        Ok(())
    }

    /// read config from a file.
    /// 
    /// ex.
    /// obf: /tmp/test.obf
    /// curdir: ~/edax/
    /// edax: ./bin/edax
    /// evfile: ./data/eval.dat
    pub fn read(&mut self, path : &str) -> Result<(), String> {
        let file = File::open(path);
        if file.is_err() {return Err(file.err().unwrap().to_string());}

        let file = file.unwrap();
        let lines = BufReader::new(file);
        for line in lines.lines() {
            match line {
                Ok(l) =>{
                    if l.starts_with("obf:") {
                        self.obf = String::from(l[4..].trim());
                    } else if l.starts_with("curdir:") {
                        self.curdir = String::from(l[7..].trim());
                    } else if l.starts_with("edax:") {
                        self.path = String::from(l[5..].trim());
                    } else if l.starts_with("evfile:") {
                        self.evfile = String::from(l[7..].trim());
                    }
                },
                Err(err) => {return Err(err.to_string())}
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn to_str(&self) -> String {
        format!("obf:{}, curdir:{}, edax:{}, evfile:{}",
                self.obf, self.curdir, self.path, self.evfile)
    }

    pub fn obf2file(&self, obf : &str) {
        // println!("put board to a file...");
        let mut f = File::create(&self.obf).unwrap();
        f.write(obf.as_bytes()).unwrap();
        f.write("\n".as_bytes()).unwrap();
        f.flush().unwrap();
    }

    fn spawn(&self) -> std::io::Result<std::process::Child> {
        std::process::Command::new(&self.path)
            .arg("--solve").arg(&self.obf).current_dir(&self.curdir)
            .arg("--eval-file").arg(&self.evfile)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null()).spawn()
    }

    pub fn run(&self) -> Result<(String, String), String> {
        // launch edax
        let cmd = match self.spawn() {
            Err(msg) => {
                return Err(format!("error running edax... [{}], config:[{}]",
                            msg, self.to_str()))
            },
            Ok(prcs) => prcs,
        };
        // read stdout and get moves
        let w = cmd.wait_with_output().unwrap();
        let txt = String::from_utf8(w.stdout).unwrap();
        // println!("{txt}");
        let lines : Vec<_> = txt.split("\n").collect();
        // println!("{}", lines[2]);
        let pos = lines[2].chars().position(|c| c.is_alphabetic());
        if pos.is_none() {return Err(format!("EDAX:\"{}\"", txt));}
        let i = pos.unwrap();
        let mut xtxt = lines[2].chars().nth(i).unwrap().to_ascii_lowercase().to_string();
        xtxt.push(lines[2].chars().nth(i + 1).unwrap());

        let scoreptn = regex::Regex::new("\\s+([+-]\\d\\d)").unwrap();
        match scoreptn.captures(&lines[2]) {
            Some(cap) => {Ok((xtxt, String::from(&cap[1])))},
            _ => {Err(format!("invalid input from edax. \"{}\" pos{xtxt}", lines[2]))}
        }
    }
}