use std::fs::File;
use std::io::Write;

const OBF : &str = "/tmp/test.obf";
const CD : &str = "../../edax-reversi/";
const EDAXPATH : &str = "./bin/lEdax-x64-modern";
const EVFILE : &str = "data/eval.dat";

pub fn obf2file(obf : &str) {
    // println!("put board to a file...");
    let mut f = File::create(OBF).unwrap();
    f.write(obf.as_bytes()).unwrap();
    f.write("\n".as_bytes()).unwrap();
    f.flush().unwrap();
}

pub fn run() -> Result<(String, String), String> {
    // launch edax
    let cmd = match std::process::Command::new(EDAXPATH)
        .arg("--solve").arg(OBF).current_dir(CD)
        .arg("--eval-file").arg(EVFILE)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null()).spawn() {
        Err(msg) => return Err(format!("error running edax... [{}]", msg)),
        Ok(prcs) => prcs,
    };
    // read stdout and get moves
    let w = cmd.wait_with_output().unwrap();
    let txt = String::from_utf8(w.stdout).unwrap();
    // println!("{txt}");
    let lines : Vec<_> = txt.split("\n").collect();
    // println!("{}", lines[2]);
    let pos = lines[2].chars().position(|c| c.is_alphabetic());
    if pos.is_none() {return Err(format!("EDAX:\"{}\"", lines[2]));}
    let i = pos.unwrap();
    let mut xtxt = lines[2].chars().nth(i).unwrap().to_ascii_lowercase().to_string();
    xtxt.push(lines[2].chars().nth(i + 1).unwrap());

    let scoreptn = regex::Regex::new("\\s+([+-]\\d\\d)").unwrap();
    match scoreptn.captures(&lines[2]) {
        Some(cap) => {Ok((xtxt, String::from(&cap[1])))},
        _ => {Err(format!("invalid input from edax. \"{}\" pos{xtxt}", lines[2]))}
    }
}