use std::fs::File;
use std::io::{BufReader, BufRead, Write};

const OBF : &str = "/tmp/test.obf";
const CD : &str = "../../edax-reversi/";
const EDAXPATH : &str = "./bin/lEdax-x64-modern";
const EVFILE : &str = "data/eval.dat";

pub struct EdaxRunner {
    obfpath : String,
    curdir : String,
    path : String,
    evfile : String
}

impl EdaxRunner {
    pub fn new() -> EdaxRunner {
        EdaxRunner {
            obfpath: String::from(OBF),
            curdir: String::from(CD),
            path: String::from(EDAXPATH),
            evfile: String::from(EVFILE)
        }
    }

    pub fn from_config(path : &str) -> Result<EdaxRunner, String> {
        let mut er = EdaxRunner::new();
        if path.is_empty() {
            return Ok(er);
        }

        match er.read(path) {
            Ok(_) => Ok(er),
            Err(msg) => Err(msg),
        }
    }

    #[allow(dead_code)]
    pub fn config(&mut self, obf : &str, cd : &str, path : &str, evfile : &str)
            -> Result<(), String> {
        if !obf.is_empty() {
            self.obfpath = String::from(obf);
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
                        self.obfpath = String::from(l[4..].trim());
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
                self.obfpath, self.curdir, self.path, self.evfile)
    }

    pub fn obf2file(&self, obf : &str) {
        // println!("put board to a file...");
        let mut f = File::create(&self.obfpath).unwrap();
        f.write(obf.as_bytes()).unwrap();
        f.write("\n".as_bytes()).unwrap();
        f.flush().unwrap();
    }

    fn spawn(&self, obf : &str) -> std::io::Result<std::process::Child> {
        self.obf2file(obf);
        std::process::Command::new(&self.path)
            .arg("--solve").arg(&self.obfpath).current_dir(&self.curdir)
            .arg("--eval-file").arg(&self.evfile)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null()).spawn()
    }

    pub fn run(&self, obf : &str) -> Result<(String, String), String> {
        // launch edax
        let cmd = match self.spawn(obf) {
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

pub struct RuversiRunner {
    curdir : String,
    path : String,
    evfile : String
}

impl RuversiRunner {
    pub fn new() -> RuversiRunner {
        RuversiRunner {
            curdir : String::from("../ruversi2"),
            path : String::from("./target/release/ruversi"),
            evfile : String::from("data/evaltable.txt"),
        }
    }

    pub fn from_config(path : &str) -> Result<RuversiRunner, String> {
        let mut rr = RuversiRunner::new();
        if path.is_empty() {
            return Ok(rr);
        }

        match rr.read(path) {
            Ok(_) => Ok(rr),
            Err(msg) => Err(msg),
        }
    }

    #[allow(dead_code)]
    pub fn to_str(&self) -> String {
        format!("curdir:{}, ruversi:{}, evfile:{}",
                self.curdir, self.path, self.evfile)
    }

    /// read config from a file.
    /// 
    /// ex.
    /// curdir: ~/ruversi/
    /// path: ./bin/ruversi
    /// evfile: ./data/eval.dat
    pub fn read(&mut self, path : &str) -> Result<(), String> {
        let file = File::open(path);
        if file.is_err() {return Err(file.err().unwrap().to_string());}

        let file = file.unwrap();
        let lines = BufReader::new(file);
        for line in lines.lines() {
            match line {
                Ok(l) =>{
                    if l.starts_with("curdir:") {
                        self.curdir = String::from(l[7..].trim());
                    } else if l.starts_with("path:") {
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

    fn spawn(&self, rfen : &str) -> std::io::Result<std::process::Child> {
        let mut cmd = std::process::Command::new(&self.path);
            cmd.arg("--rfen").arg(rfen).current_dir(&self.curdir)
            .arg("--ev1").arg(&self.evfile)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null());
        // println!("cmd:{cmd:?}");
        cmd.spawn()
    }

    pub fn run(&self, rfen : &str) -> Result<(String, String), String> {
        // launch edax
        let cmd = match self.spawn(rfen) {
            Err(msg) => {
                return Err(format!("error running ruversi... [{}], config:[{}]",
                            msg, self.to_str()))
            },
            Ok(prcs) => prcs,
        };
        // read stdout and get moves
        let w = cmd.wait_with_output().unwrap();
        let txt = String::from_utf8(w.stdout).unwrap();
        // println!("txt:{txt}");
        let lines : Vec<_> = txt.split("\n").collect();
        println!("{}", lines[12]);
        let posptn = regex::Regex::new(" (@@|\\[\\])([a-h][1-8])").unwrap();
        let xtxt;
        match posptn.captures(&lines[12]) {
            Some(cap) => {xtxt = String::from(&cap[2]);},
            _ => {
                return Err(format!("invalid input from ruversi. \"{}\"", lines[12]));
            }
        };

        let scoreptn = regex::Regex::new("val:(-?\\d+\\.\\d+) ").unwrap();
        match scoreptn.captures(&lines[12]) {
            Some(cap) => {Ok((xtxt, String::from(&cap[1])))},
            _ => {Err(format!("invalid input from edax. \"{}\" pos{xtxt}", lines[2]))}
        }
    }
}
