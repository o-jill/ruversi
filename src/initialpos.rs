use std::{fs::File, io::{BufReader, BufRead}};
use std::collections::HashSet;
// use regex::Regex;

pub const INITIALPOSFILE : &str = "data/initialpos.txt";
pub const EQUALFILE : &str = "data/initialpos.eq5.txt";

pub struct RfenSet {
  tag: String,
  pub rfens: Vec<String>,
}

impl RfenSet {
  pub fn new(tag : String, rfen : String) -> RfenSet {
      RfenSet {
          tag : tag,
          rfens : vec![rfen]
      }
  }

  pub fn add(&mut self, rfen : &str) {
      self.rfens.push(rfen.to_string());
  }

  pub fn len(&self) -> usize {
      self.rfens.len()
  }

  pub fn dump(&self) -> String {
      let mut ret : String = format!("# {} {}\n", self.tag, self.len());
      ret += &self.rfens.join("\n");
      ret
  }
}

pub struct InitialPos {
  list : Vec<RfenSet>,
}

impl InitialPos {
  pub fn new() -> InitialPos {
      InitialPos { list: Vec::new() }
  }

  pub fn read(path : &str) -> Result<InitialPos, String> {
      let mut tag = String::new();
      let mut ret = InitialPos::new();

      let file = File::open(path);
      if file.is_err() {return Err(file.err().unwrap().to_string());}

      let file = file.unwrap();
      let lines = BufReader::new(file);
      for line in lines.lines() {
            match line {
                Ok(line) => {
                    // println!("line:{}", line);
                    let tagptn = regex::Regex::new("#\\s+(\\S+) *").unwrap();
                    match tagptn.captures(&line) {
                        Some(cap) => {
                            tag = String::from(&cap[1]);
                            // println!("tag:{}", tag);
                        },
                        _ => {}
                    }
                    let rfenptn =
                        regex::Regex::new("^([1-8A-Ha-h/]+ [bw])").unwrap();
                    match rfenptn.captures(&line) {
                        Some(cap) => {
                            let rfen = &cap[1];
                            ret.add(&tag, rfen);
                            // println!("rfen:{}({})", rfen, tag);
                        },
                        _ => {continue;}
                    }
                },
                Err(e) => {
                    return Err(e.to_string());
                },
            }
        }
        Ok(ret)
    }

    fn add(&mut self, tag : &str, rfen : &str) {
        for rs in self.list.iter_mut() {
            if rs.tag == tag {
                rs.add(rfen);
                return;
            }
        }
        self.list.push(RfenSet::new(tag.to_string(), rfen.to_string()));
    }

    pub fn tags(&self) -> Vec<&str> {
        self.list.iter().map(|a| {a.tag.as_str()}).collect::<Vec<&str>>()
    }

    pub fn at(&self, tag : &str) -> Option<&RfenSet> {
        self.list.iter().find(|a| a.tag == tag)
    }

    pub fn rfens(&self, tags : &[&str]) -> Vec<String> {
        let mut ret = Vec::<String>::new();
        for a in self.list.iter() {
            for &b in tags.iter() {
                if a.tag == b {
                    ret.extend_from_slice(&a.rfens);
                    break;
                }
            }
        }
        ret
    }

    pub fn rfens_uniq(&self, tags : &[&str]) -> Vec<String> {
        let mut ret = Vec::<String>::new();
        for a in self.list.iter() {
            for &b in tags.iter() {
                if a.tag == b {
                    let t = a.rfens.clone();
                    let hm = t.into_iter().collect::<HashSet<String>>();
                    let mut rfentbl = Vec::from_iter(hm);
                    rfentbl.sort();
                    ret.extend_from_slice(&rfentbl);
                    break;
                }
            }
        }
        ret
    }

    pub fn rfens_all(&self) -> Vec<String> {
        let mut ret = Vec::<String>::new();
        for a in self.list.iter() {
            ret.extend_from_slice(&a.rfens);
        }
        ret
    }
}

#[test]
fn testinitpos() {
    let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), INITIALPOSFILE);
    let ip = InitialPos::read(&path);
    assert!(ip.is_ok());
    let ipos = ip.unwrap();
    let tags = ipos.tags();
    assert_eq!(tags, ["ZERO", "ONE", "TWO", "THREE", "FOUR", "FIVE", "SIX"]);
    let zero = ipos.at("ZERO");
    assert!(zero.is_some());
    let zero = zero.unwrap();
    assert_eq!(zero.tag, "ZERO");
    assert_eq!(zero.rfens, ["8/8/8/3Aa3/3aA3/8/8/8 b".to_string()]);
    let one = ipos.at("ONE");
    assert!(one.is_some());
    let one = one.unwrap();
    assert_eq!(one.tag, "ONE");
    assert_eq!(one.rfens, [
        "8/8/8/3Aa3/2C3/8/8/8 w".to_string(), "8/8/8/3Aa3/3B3/3A4/8/8 w".to_string(),
        "8/8/4A3/3B3/3aA3/8/8/8 w".to_string(), "8/8/8/3C2/3aA3/8/8/8 w".to_string()]);
    let zeroone = ipos.rfens(&["ZERO", "ONE"]);
    assert_eq!(zeroone, ["8/8/8/3Aa3/3aA3/8/8/8 b".to_string(),
        "8/8/8/3Aa3/2C3/8/8/8 w".to_string(), "8/8/8/3Aa3/3B3/3A4/8/8 w".to_string(),
        "8/8/4A3/3B3/3aA3/8/8/8 w".to_string(), "8/8/8/3C2/3aA3/8/8/8 w".to_string()]);
}
