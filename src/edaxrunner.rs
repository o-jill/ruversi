use core::fmt;
use std::fs::File;
use std::io::{BufReader, BufRead, Write};
use std::fmt::{Display, Formatter};
use std::process::{Child, Command, Stdio};

/**
 * configファイルのargsタグを処理する。
 * # Argumemts  
 * - txt args:を含まない"arg:"の後ろに書いてある文字列。
 * # Returns  
 * - Ok(Vec<String>)  
 *   引数に渡したい文字列の配列
 * - Err(String>)  
 *   処理エラーの内容
 */
fn parse_args_tag(txt : &str) -> Result<Vec<String>, String> {
    let args = txt.trim().split(",")
        .map(|s| s.trim().to_string()).collect::<Vec<_>>();
    if args.len() > 1 || !args[0].is_empty() {
        for (i, a) in args.iter().enumerate() {
            if a.is_empty() {
                return Err(
                    format!("\"{txt}\" contains empty part @{i}! {args:?}"));
            }
        }
        Ok(args)
    } else  {
        Ok(Vec::new())
    }
}

const OBF : &str = "test.obf";
const CD : &str = "../../edax-reversi/";
const EDAXPATH : &str = "./bin/lEdax-x64-modern";
const EVFILE : &str = "data/eval.dat";

pub struct EdaxRunner {
    obfpath : String,
    curdir : String,
    path : String,
    evfile : String,
    args : Vec<String>,
}

impl std::fmt::Display for EdaxRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "obf:{}, curdir:{}, edax:{}, evfile:{}, args:{:?}",
               self.obfpath, self.curdir, self.path, self.evfile, self.args)
    }
}

impl EdaxRunner {
    pub fn new() -> EdaxRunner {
        EdaxRunner {
            obfpath: {
                let id = std::process::id();
                let tmp = std::env::temp_dir().join(
                    &format!("ruversi_{id:x}_{OBF}"));
                tmp.to_str().unwrap().to_string()
            },
            curdir: String::from(CD),
            path: String::from(EDAXPATH),
            evfile: String::from(EVFILE),
            args: Vec::new(),
        }
    }

    pub fn from_config(path : &std::path::PathBuf) -> Result<EdaxRunner, String> {
        let mut er = EdaxRunner::new();
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
    /// args: --some,arguments,if,needed
    /// 
    /// note:
    /// COMMAs between `"`s are not be ignored yet...
    /// "a,b" will be `"a` and `b"`...
    pub fn read(&mut self, path : &std::path::PathBuf) -> Result<(), String> {
        let file = File::open(path);
        if file.is_err() {return Err(file.err().unwrap().to_string());}

        let file = file.unwrap();
        let lines = BufReader::new(file);
        for line in lines.lines() {
            match line {
                Ok(l) => {
                    if let Some(obf) = l.strip_prefix("obf:") {
                        self.obfpath = String::from(obf.trim());
                    } else if let Some(cd) = l.strip_prefix("curdir:") {
                        self.curdir = String::from(cd.trim());
                    } else if let Some(ed) = l.strip_prefix("edax:") {
                        self.path = String::from(ed.trim());
                    } else if let Some(evf) = l.strip_prefix("evfile:") {
                        self.evfile = String::from(evf.trim());
                    } else if let Some(args_txt) = l.strip_prefix("args:") {
                        self.args = parse_args_tag(&args_txt)?;
                    }
                },
                Err(err) => {return Err(err.to_string())}
            }
        }
        Ok(())
    }

    pub fn obf2file(&self, obf : &str) {
        // println!("put board to a file...");
        match File::create(&self.obfpath) {
            Ok(mut f) => {
                f.write_all(obf.as_bytes()).unwrap();
                f.write_all("\n".as_bytes()).unwrap();
                f.flush().unwrap();
            },
            Err(e) => {
                panic!("{e:?}, {}", &self.obfpath);
            }
        }
    }

    fn spawn(&self, obf : &str) -> std::io::Result<Child> {
         self.obf2file(obf);
         std::env::set_current_dir(&self.curdir).unwrap();
        let mut cmd = Command::new(&self.path);
        cmd.arg("--solve").arg(&self.obfpath)
            .arg("--eval-file").arg(&self.evfile).args(&self.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        // eprintln!("cmd:{cmd:?}");
        cmd.spawn()
    }

    pub fn run(&self, obf : &str) -> Result<(String, String), String> {
        // launch edax
        let curdir = std::env::current_dir().unwrap();
        let cmd = match self.spawn(obf) {
            Err(msg) => {
                std::env::set_current_dir(curdir).unwrap();
                return Err(
                    format!("error running edax... [{msg}], config:[{self}]"))
            },
            Ok(prcs) => prcs,
        };
        // read stdout and get moves
        let w = cmd.wait_with_output().unwrap();

        std::env::set_current_dir(curdir).unwrap();
        let txt = String::from_utf8(w.stdout).unwrap();
        // println!("{txt}");
        let lines : Vec<_> = txt.split("\n").collect();
        // println!("{}", lines[2]);
        if lines.len() < 3 {
            panic!("invalid input {lines:?}");
        }
        let pos = lines[2].chars().position(|c| c.is_alphabetic());
        if pos.is_none() {return Err(format!("EDAX:\"{txt}\""));}
        let i = pos.unwrap();
        let mut xtxt = lines[2].chars().nth(i).unwrap().to_ascii_lowercase().to_string();
        xtxt.push(lines[2].chars().nth(i + 1).unwrap());

        let scoreptn = regex::Regex::new("\\s+([+-]\\d\\d)").unwrap();
        match scoreptn.captures(lines[2]) {
            Some(cap) => {Ok((xtxt, String::from(&cap[1])))},
            _ => {Err(format!("invalid input from edax. \"{}\" pos{xtxt}", lines[2]))}
        }
    }
}

pub struct RuversiRunner {
    curdir : String,
    path : String,
    evfile : String,
    verbose : bool,
    args : Vec<String>,
}

impl std::fmt::Display for RuversiRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "curdir:{}, ruversi:{}, evfile:{}, args:{:?}",
               self.curdir, self.path, self.evfile, self.args)
    }
}

impl RuversiRunner {
    pub fn new() -> RuversiRunner {
        RuversiRunner {
            curdir : String::from("../ruversi2"),
            path : String::from("./target/release/ruversi"),
            evfile : String::from("data/evaltable.txt"),
            verbose : true,
            args : Vec::new(),
        }
    }

    pub fn set_verbose(&mut self, verbose : bool) {
        self.verbose = verbose;
    }

    pub fn from_config(path : &std::path::PathBuf)
            -> Result<RuversiRunner, String> {
        let mut rr = RuversiRunner::new();
        if path.as_os_str().is_empty() {
            return Ok(rr);
        }

        match rr.read(path) {
            Ok(_) => Ok(rr),
            Err(msg) => Err(msg),
        }
    }

    /// read config from a file.
    /// 
    /// ex.
    /// curdir: ~/ruversi/
    /// path: ./bin/ruversi
    /// evfile: ./data/eval.dat
    /// args: --depth,7,--silent
    /// 
    /// note:
    /// COMMAs between `"`s are not be ignored yet...
    /// "a,b" will be `"a` and `b"`...
    pub fn read(&mut self, path : &std::path::PathBuf) -> Result<(), String> {
        let file = File::open(path);
        if file.is_err() {return Err(file.err().unwrap().to_string());}

        let file = file.unwrap();
        let lines = BufReader::new(file);
        for line in lines.lines() {
            match line {
                Ok(l) => {
                    if let Some(cd) = l.strip_prefix("curdir:") {
                        self.curdir = String::from(cd.trim());
                    } else if let Some(ed) = l.strip_prefix("path:") {
                        self.path = String::from(ed.trim());
                    } else if let Some(evf) = l.strip_prefix("evfile:") {
                        self.evfile = String::from(evf.trim());
                    } else if let Some(args_txt) = l.strip_prefix("args:") {
                        self.args = parse_args_tag(&args_txt)?;
                    }
                },
                Err(err) => {return Err(err.to_string())}
            }
        }
        Ok(())
    }

    fn spawn(&self, rfen : &str) -> std::io::Result<Child> {
        std::env::set_current_dir(&self.curdir).unwrap();
        let mut cmd = Command::new(&self.path);
        cmd.arg("--rfen").arg(rfen).arg("--ev1").arg(&self.evfile)
            .args(&self.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        if self.verbose {eprintln!("cmd:{cmd:?}");}
        cmd.spawn()
    }

    pub fn run(&self, rfen : &str) -> Result<(String, String), String> {
        // launch another
        let curdir = std::env::current_dir().unwrap();
        let cmd = match self.spawn(rfen) {
            Err(msg) => {
                std::env::set_current_dir(curdir).unwrap();
                return Err(format!(
                    "error running ruversi... [{msg}], config:[{self}]"))
            },
            Ok(prcs) => prcs,
        };
        // read stdout and get moves
        let w = cmd.wait_with_output().unwrap();
        std::env::set_current_dir(curdir).unwrap();
        let txt = String::from_utf8(w.stdout).unwrap();
        // println!("txt:{txt}");
        let lines : Vec<_> = txt.split("\n").collect();
        if lines.len() < 13 {
            return Err(format!("invalid input {lines:?}"));
        }

        let res = lines[12].to_ascii_lowercase();
        if self.verbose {println!("opp:{}", &res);}
        let posptn = regex::Regex::new("nodes\\. ([A-Ha-h][1-8])").unwrap();
        let xtxt = match posptn.captures(&res) {
            Some(cap) => {
                String::from(&cap[1].to_lowercase())
            },
            _ => {
                return Err(
                    format!("invalid input from ruversi. \"{}\"", &res));
            }
        };

        let scoreptn = regex::Regex::new("val:(-?\\d+\\.\\d+) ").unwrap();
        match scoreptn.captures(&res) {
            Some(cap) => {
                Ok((xtxt, String::from(&cap[1])))
            },
            _ => {
                Err(format!("invalid input from ruversi. \"{}\" pos{xtxt}",
                    lines[2]))
            }
        }
    }
}

pub struct CassioRunner {
    curdir : String,
    path : String,
    evfile : String,
    cas : String,
    args : Vec<String>,
    // verbose : bool,
}

impl Display for CassioRunner {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        write!(f, "curdir:{}, path:{}, evfile:{}, cassio:{}, args:{:?}",
                self.curdir, self.path, self.evfile, self.cas, self.args)
    }
}

impl CassioRunner {
    pub fn new() -> Self {
        Self {
            curdir: String::from(CD),
            path: String::from(EDAXPATH),
            evfile: String::from(EVFILE),
            cas: String::from("-cassio"),
            args: Vec::new(),
        }
    }

    // pub fn set_verbose(&mut self, verbose : bool) {
    //     self.verbose = verbose;
    // }

    pub fn from_config(path : &std::path::PathBuf)
            -> Result<CassioRunner, String> {
        let mut rr = CassioRunner::new();
        if path.as_os_str().is_empty() {
            return Ok(rr);
        }

        match rr.read(path) {
            Ok(_) => Ok(rr),
            Err(msg) => Err(msg),
        }
    }

    /// read config from a file.
    /// 
    /// ex.
    /// curdir: ~/ruversi/
    /// path: ./bin/ruversi
    /// evfile: ./data/eval.dat
    /// args: --some,arguments,if,needed
    /// 
    /// note:
    /// COMMAs between `"`s are not be ignored yet...
    /// "a,b" will be `"a` and `b"`...
    pub fn read(&mut self, path : &std::path::PathBuf) -> Result<(), String> {
        let file = File::open(path);
        if file.is_err() {return Err(file.err().unwrap().to_string());}

        let file = file.unwrap();
        let lines = BufReader::new(file);
        for line in lines.lines() {
            match line {
                Ok(l) => {
                    if let Some(cd) = l.strip_prefix("curdir:") {
                        // println!("{l}");
                        self.curdir = String::from(cd.trim());
                    } else if let Some(ed) = l.strip_prefix("path:") {
                        // println!("{l}");
                        self.path = String::from(ed.trim());
                    } else if let Some(ed) = l.strip_prefix("edax:") {
                        // println!("{l}");
                        self.path = String::from(ed.trim());
                    } else if let Some(evf) = l.strip_prefix("evfile:") {
                        // println!("{l}");
                        self.evfile = String::from(evf.trim());
                    } else if let Some(cas) = l.strip_prefix("cas:") {
                        // println!("{l}");
                        self.cas = String::from(cas.trim());
                    } else if let Some(args_txt) = l.strip_prefix("args:") {
                        self.args = parse_args_tag(&args_txt)?;
                    }
                },
                Err(err) => {return Err(err.to_string())}
            }
        }
        Ok(())
    }

    fn spawn(&self) -> std::io::Result<Child> {
        std::env::set_current_dir(&self.curdir).unwrap();
        let mut cmd = Command::new(&self.path);
        cmd.arg(&self.cas)
            .arg("-eval-file").arg(&self.evfile).args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        // println!("cmd:{cmd:?}");
        cmd.spawn()
    }

    pub fn run(&self) -> Result<Child, String> {
        // launch cassio
        let curdir = std::env::current_dir().unwrap();
        match self.spawn() {
            Err(msg) => {
                std::env::set_current_dir(curdir).unwrap();
                Err(format!("error running cassio... [{msg}], {self}"))
            },
            Ok(prcs) => {
                std::env::set_current_dir(curdir).unwrap();
                Ok(prcs)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::fs;

    #[test]
    fn test_edaxrunner_default_values() {
        // EdaxRunner::new() で各フィールドがデフォルト値になっていることを確認
        let er = EdaxRunner::new();
        let id = std::process::id();
        let file_name = format!("ruversi_{id:x}_{OBF}");
        let tmp = std::env::temp_dir().join(&file_name);
        assert_eq!(er.obfpath, tmp.to_str().unwrap());
        assert_eq!(er.curdir, "../../edax-reversi/");
        assert_eq!(er.path, "./bin/lEdax-x64-modern");
        assert_eq!(er.evfile, "data/eval.dat");
    }

    #[test]
    fn test_edaxrunner_config() {
        // config メソッドで値が上書きされることを確認
        let mut er = EdaxRunner::new();
        let result = er.config("foo.obf", "./bar", "./baz", "./eval.dat");
        assert!(result.is_ok());
        assert_eq!(er.obfpath, "foo.obf");
        assert_eq!(er.curdir, "./bar");
        assert_eq!(er.path, "./baz");
        assert_eq!(er.evfile, "./eval.dat");
    }

    #[test]
    fn test_edaxrunner_config_partial() {
        // 一部だけ上書きした場合、それ以外はデフォルト値のまま
        let mut er = EdaxRunner::new();
        let result = er.config("", "updated_dir", "", "");
        assert!(result.is_ok());
        let id = std::process::id();
        let file_name = format!("ruversi_{id:x}_{OBF}");
        let tmp = std::env::temp_dir().join(&file_name);
        assert_eq!(er.obfpath, tmp.to_str().unwrap());
        assert_eq!(er.curdir, "updated_dir");
        assert_eq!(er.path, "./bin/lEdax-x64-modern");
        assert_eq!(er.evfile, "data/eval.dat");
    }

    #[test]
    fn test_edaxrunner_to_str() {
        // to_str の内容がフィールドに基づくことを確認
        let er = EdaxRunner::new();
        let s = er.to_string();
        let id = std::process::id();
        let file_name = format!("ruversi_{id:x}_{OBF}");
        let tmp = std::env::temp_dir().join(&file_name);
        assert!(s.contains(&format!("obf:{}", tmp.to_str().unwrap())));
        assert!(s.contains("curdir:../../edax-reversi/"));
        assert!(s.contains("edax:./bin/lEdax-x64-modern"));
        assert!(s.contains("evfile:data/eval.dat"));
    }

    #[test]
    fn test_edaxrunner_read_config_file() {
        // 一時ファイルに設定を書き込み、read で値が読み込まれることを確認
        let tmp = std::env::temp_dir();
        let config_path =
                tmp.join("test_edaxrunner_config.txt");
        let contents = "\
obf:/tmp/myobf.obf
curdir:/tmp/myedax
edax:/tmp/ledax
evfile:/tmp/myevfile.dat
";

        // 前のテストのファイルが残ってたら消す
        if config_path.exists() {
            println!("removed config file for test.");
            fs::remove_file(&config_path).unwrap();
        }

        {
            let cp = config_path.clone();
            let mut file = File::create(cp).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        let mut er = EdaxRunner::new();
        let result = er.read(&config_path);
        assert_eq!(result, Ok(()));
        assert_eq!(er.obfpath, "/tmp/myobf.obf");
        assert_eq!(er.curdir, "/tmp/myedax");
        assert_eq!(er.path, "/tmp/ledax");
        assert_eq!(er.evfile, "/tmp/myevfile.dat");
        fs::remove_file(config_path).unwrap();
    }

    #[test]
    fn test_edaxrunner_read_config_file_empty_arg1() {
        // 一時ファイルに設定を書き込み、
        // read で値が読み込まれ、
        // 空文字の引数が含まれているErrが返る事を確認
        let tmp = std::env::temp_dir();
        let config_path =
                tmp.join("test_edaxrunner_config_arg1.txt");
        let contents = "\
obf:/tmp/myobf.obf
curdir:/tmp/myedax
edax:/tmp/ledax
evfile:/tmp/myevfile.dat
args:a,b,c,
";

        // 前のテストのファイルが残ってたら消す
        if config_path.exists() {
            println!("removed config file for test.");
            fs::remove_file(&config_path).unwrap();
        }

        {
            let cp = config_path.clone();
            let mut file = File::create(cp).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        let mut er = EdaxRunner::new();
        let result = er.read(&config_path);
        assert_eq!(result, Err("\"a,b,c,\" contains empty part @3! [\"a\", \"b\", \"c\", \"\"]".to_string()));
        fs::remove_file(config_path).unwrap();
    }

    #[test]
    fn test_edaxrunner_read_config_file_empty_arg2() {
        // 一時ファイルに設定を書き込み、
        // read で値が読み込まれ、
        // 空文字の引数が含まれているErrが返る事を確認
        let tmp = std::env::temp_dir();
        let config_path =
                tmp.join("test_edaxrunner_config_arg2.txt");
        let contents = "\
obf:/tmp/myobf.obf
curdir:/tmp/myedax
edax:/tmp/ledax
evfile:/tmp/myevfile.dat
args:a,b,,c
";

        // 前のテストのファイルが残ってたら消す
        if config_path.exists() {
            println!("removed config file for test.");
            fs::remove_file(&config_path).unwrap();
        }

        {
            let cp = config_path.clone();
            let mut file = File::create(cp).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        let mut er = EdaxRunner::new();
        let result = er.read(&config_path);
        assert_eq!(result, Err("\"a,b,,c\" contains empty part @2! [\"a\", \"b\", \"\", \"c\"]".to_string()));
        fs::remove_file(config_path).unwrap();
    }

    #[test]
    fn test_edaxrunner_read_config_file_empty_arg3() {
        // 一時ファイルに設定を書き込み、
        // read で値が読み込まれることを確認
        // args:の後ろが空白だけでもエラーに鳴らないことの確認
        let tmp = std::env::temp_dir();
        let config_path =
                tmp.join("test_edaxrunner_config_arg3.txt");
        let contents = "\
obf:/tmp/myobf.obf
curdir:/tmp/myedax
edax:/tmp/ledax
evfile:/tmp/myevfile.dat
args:  
";

        // 前のテストのファイルが残ってたら消す
        if config_path.exists() {
            println!("removed config file for test.");
            fs::remove_file(&config_path).unwrap();
        }

        {
            let cp = config_path.clone();
            let mut file = File::create(cp).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        let mut er = EdaxRunner::new();
        let result = er.read(&config_path);
        assert_eq!(result, Ok(()));
        assert_eq!(er.obfpath, "/tmp/myobf.obf");
        assert_eq!(er.curdir, "/tmp/myedax");
        assert_eq!(er.path, "/tmp/ledax");
        assert_eq!(er.evfile, "/tmp/myevfile.dat");
        assert!(er.args.is_empty());
        fs::remove_file(config_path).unwrap();
    }

    #[test]
    fn test_edaxrunner_read_config_file_empty_arg4() {
        // 一時ファイルに設定を書き込み、
        // read で値が読み込まれることを確認
        // args:の後ろが空でもエラーに鳴らないことの確認
        let tmp = std::env::temp_dir();
        let config_path =
                tmp.join("test_edaxrunner_config_arg4.txt");
        let contents = "\
obf:/tmp/myobf.obf
curdir:/tmp/myedax
edax:/tmp/ledax
evfile:/tmp/myevfile.dat
args:
";

        // 前のテストのファイルが残ってたら消す
        if config_path.exists() {
            println!("removed config file for test.");
            fs::remove_file(&config_path).unwrap();
        }

        {
            let cp = config_path.clone();
            let mut file = File::create(cp).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        let mut er = EdaxRunner::new();
        let result = er.read(&config_path);
        assert_eq!(result, Ok(()));
        assert_eq!(er.obfpath, "/tmp/myobf.obf");
        assert_eq!(er.curdir, "/tmp/myedax");
        assert_eq!(er.path, "/tmp/ledax");
        assert_eq!(er.evfile, "/tmp/myevfile.dat");
        assert!(er.args.is_empty());
        fs::remove_file(config_path).unwrap();
    }

    #[test]
    fn test_edaxrunner_read_config_file_not_found() {
        // 存在しないファイルを指定した場合、Errが返ることを確認
        let mut er = EdaxRunner::new();
        let path = std::path::PathBuf::from(
                "/tmp/no_such_edaxrunner_config.txt");
        let result = er.read(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_edaxrunner_from_config_empty() {
        // from_config("") でErrが返ることを確認
        let er = EdaxRunner::from_config(&std::path::PathBuf::new());
        // assert_eq!(er.obfpath, "/tmp/test.obf");
        assert!(er.is_err());
    }

    #[test]
    fn test_edaxrunner_from_config_file() {
        // from_config で設定ファイルを読み取る
        let tmp = std::env::temp_dir();
        let config_path =
                tmp.join("test_edaxrunner2_config.txt");
        let contents = "obf:/tmp/zzz.obf\n";
        {
            let cp = config_path.clone();
            let mut file = File::create(cp).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        let er = EdaxRunner::from_config(&config_path).unwrap();
        assert_eq!(er.obfpath, "/tmp/zzz.obf");
        fs::remove_file(config_path).unwrap();
    }

    #[test]
    fn test_obf2file_creates_file_and_writes_content() {
        // obf2file でファイルが作成され、内容が正しいことを確認
        let er = EdaxRunner::new();
        let test_obf_content = "test_obf_content";
        er.obf2file(test_obf_content);
        let written = std::fs::read_to_string(er.obfpath.clone()).unwrap();
        assert!(written.contains(test_obf_content));
        // テスト用ファイルを削除
        let _ = std::fs::remove_file(er.obfpath.clone());
    }


    #[test]
    fn test_ruversirunner_default_values() {
        // RuversiRunner::new() で各フィールドがデフォルト値になっていることを確認
        let rr = RuversiRunner::new();
        assert_eq!(rr.curdir, "../ruversi2");
        assert_eq!(rr.path, "./target/release/ruversi");
        assert_eq!(rr.evfile, "data/evaltable.txt");
        assert!(rr.verbose);
    }

    #[test]
    fn test_ruversirunner_set_verbose() {
        // set_verbose で verbose フィールドが変更されることを確認
        let mut rr = RuversiRunner::new();
        rr.set_verbose(false);
        assert!(!rr.verbose);
        rr.set_verbose(true);
        assert!(rr.verbose);
    }

    #[test]
    fn test_ruversirunner_to_str() {
        // to_str の内容がフィールドに基づくことを確認
        let rr = RuversiRunner::new();
        let s = rr.to_string();
        assert!(s.contains("curdir:../ruversi2"));
        assert!(s.contains("ruversi:./target/release/ruversi"));
        assert!(s.contains("evfile:data/evaltable.txt"));
    }

    #[test]
    fn test_ruversirunner_read_config_file() {
        // 設定ファイルを作成し、read で値が読み込まれることを確認
        let tmp = std::env::temp_dir();
        let config_path =
                tmp.join("test_ruversirunner_config.txt");
        let contents = "\
curdir:/tmp/myruversi
path:/tmp/myruversi_bin
evfile:/tmp/myruversi_evfile.txt
";
        {
            let cp = config_path.clone();
            let mut file = File::create(cp).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        let mut rr = RuversiRunner::new();
        let result = rr.read(&config_path);
        assert!(result.is_ok());
        assert_eq!(rr.curdir, "/tmp/myruversi");
        assert_eq!(rr.path, "/tmp/myruversi_bin");
        assert_eq!(rr.evfile, "/tmp/myruversi_evfile.txt");
        fs::remove_file(config_path).unwrap();
    }

    #[test]
    fn test_ruversirunner_read_config_file_empty_arg1() {
        // 一時ファイルに設定を書き込み、
        // read で値が読み込まれ、
        // 空文字の引数が含まれているErrが返る事を確認
        let tmp = std::env::temp_dir();
        let config_path =
                tmp.join("test_ruversirunner_config_arg1.txt");
        let contents = "\
curdir:/tmp/myruversi
path:/tmp/myruversi_bin
evfile:/tmp/myruversi_evfile.txt
args:a,b,c,
";

        // 前のテストのファイルが残ってたら消す
        if config_path.exists() {
            println!("removed config file for test.");
            fs::remove_file(&config_path).unwrap();
        }

        {
            let cp = config_path.clone();
            let mut file = File::create(cp).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        let mut rr = RuversiRunner::new();
        let result = rr.read(&config_path);
        assert_eq!(result, Err("\"a,b,c,\" contains empty part @3! [\"a\", \"b\", \"c\", \"\"]".to_string()));
        fs::remove_file(config_path).unwrap();
    }

    #[test]
    fn test_ruversirunner_read_config_file_empty_arg2() {
        // 一時ファイルに設定を書き込み、
        // read で値が読み込まれ、
        // 空文字の引数が含まれているErrが返る事を確認
        let tmp = std::env::temp_dir();
        let config_path =
                tmp.join("test_ruversirunner_config_arg2.txt");
        let contents = "\
curdir:/tmp/myruversi
path:/tmp/myruversi_bin
evfile:/tmp/myruversi_evfile.txt
args:a,b,,c
";

        // 前のテストのファイルが残ってたら消す
        if config_path.exists() {
            println!("removed config file for test.");
            fs::remove_file(&config_path).unwrap();
        }

        {
            let cp = config_path.clone();
            let mut file = File::create(cp).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        let mut er = RuversiRunner::new();
        let result = er.read(&config_path);
        assert_eq!(result, Err("\"a,b,,c\" contains empty part @2! [\"a\", \"b\", \"\", \"c\"]".to_string()));
        fs::remove_file(config_path).unwrap();
    }

    #[test]
    fn test_ruversirunner_read_config_file_empty_arg3() {
        // 一時ファイルに設定を書き込み、
        // read で値が読み込まれることを確認
        // args:の後ろが空白だけでもエラーに鳴らないことの確認
        let tmp = std::env::temp_dir();
        let config_path =
                tmp.join("test_ruversirunner_config_arg3.txt");
        let contents = "\
curdir:/tmp/myruversi
path:/tmp/myruversi_bin
evfile:/tmp/myruversi_evfile.txt
args:  
";

        // 前のテストのファイルが残ってたら消す
        if config_path.exists() {
            println!("removed config file for test.");
            fs::remove_file(&config_path).unwrap();
        }

        {
            let cp = config_path.clone();
            let mut file = File::create(cp).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        let mut er = RuversiRunner::new();
        let result = er.read(&config_path);
        assert_eq!(result, Ok(()));
        assert_eq!(er.curdir, "/tmp/myruversi");
        assert_eq!(er.path, "/tmp/myruversi_bin");
        assert_eq!(er.evfile, "/tmp/myruversi_evfile.txt");
        assert!(er.args.is_empty());
        fs::remove_file(config_path).unwrap();
    }

    #[test]
    fn test_ruversirunner_read_config_file_empty_arg4() {
        // 一時ファイルに設定を書き込み、
        // read で値が読み込まれることを確認
        // args:の後ろが空でもエラーに鳴らないことの確認
        let tmp = std::env::temp_dir();
        let config_path =
                tmp.join("test_ruversirunner_config_arg4.txt");
        let contents = "\
curdir:/tmp/myruversi
path:/tmp/myruversi_bin
evfile:/tmp/myruversi_evfile.txt
args:
";

        // 前のテストのファイルが残ってたら消す
        if config_path.exists() {
            println!("removed config file for test.");
            fs::remove_file(&config_path).unwrap();
        }

        {
            let cp = config_path.clone();
            let mut file = File::create(cp).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        let mut er = RuversiRunner::new();
        let result = er.read(&config_path);
        assert_eq!(result, Ok(()));
        assert_eq!(er.curdir, "/tmp/myruversi");
        assert_eq!(er.path, "/tmp/myruversi_bin");
        assert_eq!(er.evfile, "/tmp/myruversi_evfile.txt");
        assert!(er.args.is_empty());
        fs::remove_file(config_path).unwrap();
    }

    #[test]
    fn test_ruversirunner_read_config_file_not_found() {
        // 存在しないファイルを指定した場合、Errが返ることを確認
        let mut rr = RuversiRunner::new();
        let result = rr.read(
            &std::path::PathBuf::from("/tmp/no_such_ruversi_config.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_ruversirunner_from_config_empty() {
        // from_config("") でデフォルト値
        let rr = RuversiRunner::from_config(
            &std::path::PathBuf::from("")).unwrap();
        assert_eq!(rr.curdir, "../ruversi2");
    }

    #[test]
    fn test_ruversirunner_from_config_file() {
        // from_config で設定ファイルを読み取る
        let tmp = std::env::temp_dir();
        let config_path =
                tmp.join("test_ruversirunner2_config.txt");
        let contents = "curdir:/tmp/abc\n";
        {
            let cp = config_path.clone();
            let mut file = File::create(cp).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        let rr = RuversiRunner::from_config(&config_path).unwrap();
        assert_eq!(rr.curdir, "/tmp/abc");
        fs::remove_file(config_path).unwrap();
    }

    #[test]
    fn test_cassiorunner_default_values() {
        // CassioRunner::new() で各フィールドがデフォルト値になっていることを確認
        let cr = CassioRunner::new();
        assert_eq!(cr.curdir, "../../edax-reversi/");
        assert_eq!(cr.path, "./bin/lEdax-x64-modern");
        assert_eq!(cr.evfile, "data/eval.dat");
        assert_eq!(cr.cas, "-cassio");
    }

    #[test]
    fn test_cassiorunner_to_str() {
        // to_str の内容がフィールドに基づくことを確認
        let cr = CassioRunner::new();
        let s = cr.to_string();
        assert!(s.contains("curdir:"));
        assert!(s.contains("path:"));
        assert!(s.contains("evfile:"));
        assert!(s.contains("cassio:"));
    }

    #[test]
    fn test_cassiorunner_read_config_file() {
        // 設定ファイルを作成し、read で値が読み込まれることを確認
        let tmp = std::env::temp_dir();
        let config_path =
                tmp.join("test_cassiorunner_config.txt");
        let contents = "\
curdir:/tmp/mycassio
path:/tmp/mycassio_path
evfile:/tmp/mycassio_evfile.txt
cas:--cassioX
";
        {
            let cp = config_path.clone();
            let mut file = File::create(cp).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        let mut cr = CassioRunner::new();
        let result = cr.read(&config_path);
        assert!(result.is_ok());
        assert_eq!(cr.curdir, "/tmp/mycassio");
        assert_eq!(cr.path, "/tmp/mycassio_path");
        assert_eq!(cr.evfile, "/tmp/mycassio_evfile.txt");
        assert_eq!(cr.cas, "--cassioX");
        fs::remove_file(config_path).unwrap();
    }

    #[test]
    fn test_cassiorunner_read_config_file_empty_arg1() {
        // 一時ファイルに設定を書き込み、
        // read で値が読み込まれ、
        // 空文字の引数が含まれているErrが返る事を確認
        let tmp = std::env::temp_dir();
        let config_path =
                tmp.join("test_cassiorunner_config_arg1.txt");
        let contents = "\
curdir:/tmp/mycassio
path:/tmp/mycassio_path
evfile:/tmp/mycassio_evfile.txt
cas:--cassioX
args:a,b,c,
";

        // 前のテストのファイルが残ってたら消す
        if config_path.exists() {
            println!("removed config file for test.");
            fs::remove_file(&config_path).unwrap();
        }

        {
            let cp = config_path.clone();
            let mut file = File::create(cp).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        let mut rr = CassioRunner::new();
        let result = rr.read(&config_path);
        assert_eq!(result, Err("\"a,b,c,\" contains empty part @3! [\"a\", \"b\", \"c\", \"\"]".to_string()));
        fs::remove_file(config_path).unwrap();
    }

    #[test]
    fn test_cassiorunner_read_config_file_empty_arg2() {
        // 一時ファイルに設定を書き込み、
        // read で値が読み込まれ、
        // 空文字の引数が含まれているErrが返る事を確認
        let tmp = std::env::temp_dir();
        let config_path =
                tmp.join("test_cassiorunner_config_arg2.txt");
        let contents = "\
curdir:/tmp/mycassio
path:/tmp/mycassio_path
evfile:/tmp/mycassio_evfile.txt
cas:--cassioX
args:a,b,,c
";

        // 前のテストのファイルが残ってたら消す
        if config_path.exists() {
            println!("removed config file for test.");
            fs::remove_file(&config_path).unwrap();
        }

        {
            let cp = config_path.clone();
            let mut file = File::create(cp).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        let mut cr = CassioRunner::new();
        let result = cr.read(&config_path);
        assert_eq!(result, Err("\"a,b,,c\" contains empty part @2! [\"a\", \"b\", \"\", \"c\"]".to_string()));
        fs::remove_file(config_path).unwrap();
    }

    #[test]
    fn test_cassiorunner_read_config_file_empty_arg3() {
        // 一時ファイルに設定を書き込み、
        // read で値が読み込まれることを確認
        // args:の後ろが空白だけでもエラーに鳴らないことの確認
        let tmp = std::env::temp_dir();
        let config_path =
                tmp.join("test_cassiorunner_config_arg3.txt");
        let contents = "\
curdir:/tmp/mycassio
path:/tmp/mycassio_path
evfile:/tmp/mycassio_evfile.txt
cas:--cassioX
args:  
";

        // 前のテストのファイルが残ってたら消す
        if config_path.exists() {
            println!("removed config file for test.");
            fs::remove_file(&config_path).unwrap();
        }

        {
            let cp = config_path.clone();
            let mut file = File::create(cp).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        let mut cr = CassioRunner::new();
        let result = cr.read(&config_path);
        assert_eq!(result, Ok(()));
        assert_eq!(cr.curdir, "/tmp/mycassio");
        assert_eq!(cr.path, "/tmp/mycassio_path");
        assert_eq!(cr.evfile, "/tmp/mycassio_evfile.txt");
        assert_eq!(cr.cas, "--cassioX");
        assert!(cr.args.is_empty());
        fs::remove_file(config_path).unwrap();
    }

    #[test]
    fn test_cassiorunner_read_config_file_empty_arg4() {
        // 一時ファイルに設定を書き込み、
        // read で値が読み込まれることを確認
        // args:の後ろが空でもエラーに鳴らないことの確認
        let tmp = std::env::temp_dir();
        let config_path =
                tmp.join("test_cassiorunner_config_arg4.txt");
        let contents = "\
curdir:/tmp/mycassio
path:/tmp/mycassio_path
evfile:/tmp/mycassio_evfile.txt
cas:--cassioX
args:
";

        // 前のテストのファイルが残ってたら消す
        if config_path.exists() {
            println!("removed config file for test.");
            fs::remove_file(&config_path).unwrap();
        }

        {
            let cp = config_path.clone();
            let mut file = File::create(cp).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        let mut cr = CassioRunner::new();
        let result = cr.read(&config_path);
        assert_eq!(result, Ok(()));
        assert_eq!(cr.curdir, "/tmp/mycassio");
        assert_eq!(cr.path, "/tmp/mycassio_path");
        assert_eq!(cr.evfile, "/tmp/mycassio_evfile.txt");
        assert_eq!(cr.cas, "--cassioX");
        assert!(cr.args.is_empty());
        fs::remove_file(config_path).unwrap();
    }

    #[test]
    fn test_cassiorunner_read_config_file_not_found() {
        // 存在しないファイルを指定した場合、Errが返ることを確認
        let mut cr = CassioRunner::new();
        let result = cr.read(
            &std::path::PathBuf::from("/tmp/no_such_cassio_config.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_cassiorunner_from_config_empty() {
        // from_config("") でデフォルト値
        let cr = CassioRunner::from_config(
            &std::path::PathBuf::from("")).unwrap();
        assert_eq!(cr.curdir, "../../edax-reversi/");
    }

    #[test]
    fn test_cassiorunner_from_config_file() {
        // from_config で設定ファイルを読み取る
        let tmp = std::env::temp_dir();
        let config_path =
                tmp.join("test_cassiorunner2_config.txt");
        let contents = "cas:--abc\n";
        {
            let cp = config_path.clone();
            let mut file = File::create(cp).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        let cr = CassioRunner::from_config(&config_path).unwrap();
        assert_eq!(cr.cas, "--abc");
        fs::remove_file(config_path).unwrap();
    }
}