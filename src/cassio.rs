use super::*;
use std::fs::OpenOptions;
use std::io::{BufRead, Read};
use std::process::{ChildStdin, ChildStdout};
use std::thread::spawn;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

const HEADER : &str = "ENGINE-PROTOCOL ";
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct OthelloEngineProtocol {
    logg : File,
    // thread : Option<JoinHandle<()>>,
    cmd : String,
    running : Arc<AtomicBool>,
}

impl OthelloEngineProtocol {
    pub fn new() -> Self {
        let log = OpenOptions::new().create(true)
            .append(true).open("/tmp/ruversi.log");

        OthelloEngineProtocol {
            logg : log.unwrap(),
            // thread : None,
            cmd : String::default(),
            running : Arc::new(AtomicBool::default()),
        }
    }

    fn processcmd(&mut self, cmd : &str) -> Result<bool, String> {
        if cmd.is_empty() {
            let running = self.running.load(Ordering::Relaxed);
            if running {
                println!("ok.");
            } else {
                println!("ready.");
            }
            return Ok(false);
        }

        if !cmd.starts_with(HEADER) {
            self.log(&format!("unknown header:{cmd}")).unwrap();
            return Ok(false);
        }

        self.cmd = cmd.to_string();
        let body = &cmd[16..];
        if body.starts_with("midgame-search") {
            let running = self.running.load(Ordering::Relaxed);
            if running {
                self.log("Error: already thinking... {cmd}").unwrap();
                return Ok(false);
            }
            self.running.store(true, Ordering::Relaxed);

            self.log(body).unwrap();

            let running = self.running.clone();
            let cmd = body.to_string();
            let _thread = Some(spawn(move || {
                let elem = cmd.split(" ").collect::<Vec<_>>();
                let obf = elem[1];
                let ban = bitboard::BitBoard::from_obf(obf).unwrap();
                let _alpha = elem[2].parse::<f32>().unwrap();
                let _beta = elem[3].parse::<f32>().unwrap();
                let depth = elem[4].parse::<u8>().unwrap();
                let _precision = elem[5].parse::<f32>().unwrap();
                // eprintln!("{obf} {_alpha}, {_beta}, {depth}, {_precision}");
                let st = Instant::now();
                let (val, node) =
                    nodebb::NodeBB::thinko_ab_simple(&ban, depth).unwrap();
                let ft = st.elapsed();
                // eprintln!("val:{:?} {} {}msec", val, node.dump(), ft.as_millis());
                let mvstr;
                if let Some(best) = node.best.as_ref() {
                    let xy = best.pos();
                    if xy == "00" {
                        mvstr = "Pa".to_string();
                    } else {
                        mvstr = xy.to_uppercase();
                    }
                } else {
                    mvstr = "--".to_string();
                }

                let range =
                    if val.is_sign_negative() {
                        format!("W:{val:.1} <= v <= W:{val:.1}")
                    } else {
                        format!("B:{val:.1} <= v <= B:{val:.1}")
                    };
                let hash = "0123456789ABCDEF";
                let nodes = node.kyokumen;
                let sec = ft.as_secs_f32();

                println!("{obf}, move {mvstr}, depth {depth}, @0%, {range}, {hash}, node {nodes}, time {sec:3}");
                running.store(false, Ordering::Relaxed);
                println!("ready.");
            })).unwrap();

            return Ok(false);
        }

        if body.starts_with("endgame-search") {
            let running = self.running.load(Ordering::Relaxed);
            if running {
                self.log("Error: already thinking... {cmd}").unwrap();
                return Ok(false);
            }
            self.running.store(true, Ordering::Relaxed);

            self.log(body).unwrap();

            let running = self.running.clone();
            let cmd = body.to_string();
            let _thread = Some(spawn(move || {
                let elem = cmd.split(" ").collect::<Vec<_>>();
                let obf = elem[1];
                let ban = bitboard::BitBoard::from_obf(obf);
                let _alpha = elem[2].parse::<f32>().unwrap();
                let _beta = elem[3].parse::<f32>().unwrap();
                let depth = ban.nblank() as u8;
                let _precision = elem[4].parse::<f32>().unwrap();
                // eprintln!("{obf} {_alpha}, {_beta}, {depth}, {_precision}");
                let st = Instant::now();
                let (val, node) =
                    nodebb::NodeBB::thinko_ab_simple(&ban, depth).unwrap();
                let ft = st.elapsed();
                // eprintln!("val:{:?} {} {}msec", val, node.dump(), ft.as_millis());
                let mvstr;
                if let Some(best) = node.best.as_ref() {
                    let xy = best.pos();
                    if xy == "00" {
                        mvstr = "Pa".to_string();
                    } else {
                        mvstr = xy.to_uppercase();
                    }
                } else {
                    mvstr = "--".to_string();
                }

                let range =
                    if val.is_sign_negative() {
                        format!("W:{val:.1} <= v <= W:{val:.1}")
                    } else {
                        format!("B:{val:.1} <= v <= B:{val:.1}")
                    };
                let hash = "0123456789ABCDEF";
                let nodes = node.kyokumen;
                let sec = ft.as_secs_f32();

                println!("{obf}, move {mvstr}, depth {depth}, @0%, {range}, {hash}, node {nodes}, time {sec:3}");
                running.store(false, Ordering::Relaxed);
                println!("ready.");
            })).unwrap();

            return Ok(false);
        }

        if body.starts_with("stop") {
            // stop thinking.
            println!("ready.");
            self.log(body).unwrap();
            return Ok(false);
        }

        if body.starts_with("get-search-infos") {
            println!("ready.");
            self.log(body).unwrap();
            return Ok(false);
        }

        if body.starts_with("new-position") {
            println!("ready.");
            self.log(body).unwrap();
            return Ok(false);
        }

        if body.starts_with("init") {
            println!("ready.");
            self.log(body).unwrap();
            return Ok(false);
        }

        if body.starts_with("get-version") {
            println!("version: ruversi {VERSION}");
            println!("ready.");
            self.log(body).unwrap();
            return Ok(false);
        }

        if body.starts_with("empty-hash") {
            println!("ready.");
            self.log(body).unwrap();
            return Ok(false);
        }

        if body.starts_with("quit") {
            self.log(body).unwrap();
            return Ok(true)
        }
        
        self.log(&format!("unknown:{cmd}")).unwrap();
        Ok(false)
    }

    fn log(&mut self, txt : &str) -> Result<(), String>{
        self.logg.write_all(txt.as_bytes()).unwrap();
        self.logg.write_all("\n".as_bytes()).unwrap();
        Ok(())
    }

    pub fn start(&mut self, path : &str) -> Result<String, String> {

        self.log("started!!!").unwrap();
        self.log(&format!("{path}")).unwrap();

        if cfg!(feature="bitboard") {
            nodebb::init_weight();
        } else {
            node::init_weight();
        }
        unsafe {
            nodebb::WEIGHT.as_mut().unwrap().read(path)?
        }

        loop {
            let mut txt = String::new();
            std::io::stdin().read_line(&mut txt).unwrap();
            if self.processcmd(txt.trim())? {
                break;
            }
        }
        Ok(String::from("Done."))
    }
}

pub struct OthelloEngineProtocolServer {
    ply1 : Option<std::process::Child>,
    ply2 : Option<std::process::Child>,
    turn : i8,
}

impl OthelloEngineProtocolServer {
    pub fn new1(ch : std::process::Child) -> Self {
        OthelloEngineProtocolServer {
            ply1 : Some(ch),
            ply2 : None,
            turn : board::NONE,
        }
    }

    pub fn new2(ch1 : std::process::Child, ch2 : std::process::Child) -> Self {
        OthelloEngineProtocolServer {
            ply1 : Some(ch1),
            ply2 : Some(ch2),
            turn : board::NONE,
        }
    }

    fn setturn(&mut self, trn : i8) {self.turn = trn;}

    fn selectplayer(&mut self) -> Result<&mut std::process::Child, String> {
        if self.turn == board::NONE {
            return Err("turn is NONE!".to_string());
        }

        Ok(if self.turn == board::SENTE {
            self.ply1.as_mut().unwrap()
        } else {
            self.ply2.as_mut().unwrap()
        })
    }

    fn getio(&mut self) -> Result<(&mut ChildStdin, &mut ChildStdout), String> {
        let ch = self.selectplayer()?;
        let toeng = if let Some(toeng) = ch.stdin.as_mut() {
            toeng
        } else {
            return Err("failed  to get to-engine pipe..".to_string());
        };
        let fromeng = if let Some(fromeng) = ch.stdout.as_mut() {
            fromeng
        } else {
            return Err("failed  to get from-engine pipe..".to_string());
        };
        Ok((toeng, fromeng))
    }

    fn init(&mut self) -> Result<(), String> {
        let (toeng, fromeng) = self.getio()?;

        toeng.write("ENGINE-PROTOCOL init\n".as_bytes());

        std::thread::sleep(std::time::Duration::from_millis(1));

        let mut bufreader = std::io::BufReader::new(fromeng);
        let mut buf = String::default();
        bufreader.read_line(&mut buf);
        if buf == "ready.\n" {
            return Ok(())
        }
        Err(format!("unknown response : \"{buf}\""))
    }

    fn get_version(&mut self) -> Result<String, String> {
        let (toeng, fromeng) = self.getio()?;

        toeng.write("ENGINE-PROTOCOL get-version\n".as_bytes());

        std::thread::sleep(std::time::Duration::from_millis(1));

        let mut bufreader = std::io::BufReader::new(fromeng);
        let mut buf = String::default();
        bufreader.read_line(&mut buf);
        let ret = buf.trim().to_string();
        bufreader.read_line(&mut buf);
        if buf == "ready.\n" {
            return Ok(ret)
        }
        Err(format!("unknown response : \"{buf}\""))
    }

    // new-position
    // midgame-search
    // endgame-search
    // get-serach-infos

    fn stop(&mut self) -> Result<(), String> {
        let (toeng, fromeng) = self.getio()?;

        toeng.write("ENGINE-PROTOCOL stop\n".as_bytes());

        std::thread::sleep(std::time::Duration::from_millis(1));

        let mut bufreader = std::io::BufReader::new(fromeng);
        let mut buf = String::default();
        bufreader.read_line(&mut buf);
        if buf == "ready.\n" {
            return Ok(())
        }
        Err(format!("unknown response : \"{buf}\""))
    }

    fn empty_hash(&mut self) -> Result<(), String> {
        let (toeng, fromeng) = self.getio()?;

        toeng.write("ENGINE-PROTOCOL empty-hash\n".as_bytes());

        std::thread::sleep(std::time::Duration::from_millis(1));

        let mut bufreader = std::io::BufReader::new(fromeng);
        let mut buf = String::default();
        bufreader.read_line(&mut buf);
        if buf == "ready.\n" {
            return Ok(())
        }
        Err(format!("unknown response : \"{buf}\""))
    }

    fn quit(&mut self) -> Result<(), String> {
        let (toeng, _fromeng) = self.getio()?;

        toeng.write("ENGINE-PROTOCOL quit\n".as_bytes());

        std::thread::sleep(std::time::Duration::from_millis(1));

        Ok(())
    }
}
