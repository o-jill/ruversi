use super::*;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};
// use std::os::fd::{AsRawFd, FromRawFd};
use std::process::{Child, ChildStdin, ChildStdout};
use std::thread::{sleep, spawn};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::Duration;

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
        let mut path = std::env::temp_dir();
        path.push("/ruversi.log");
        let log = OpenOptions::new().create(true)
            .append(true).open(path);

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
            let _thread = spawn(move || {
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
                    nodebb::NodeBB::think_ab_simple(&ban, depth).unwrap();
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
                let moves = node.bestorder();
                let nodes = node.kyokumen;
                let sec = ft.as_secs_f32();

                println!("{obf}, move {mvstr}, depth {depth}, @0%, {range}, {moves}, node {nodes}, time {sec:3}");
                running.store(false, Ordering::Relaxed);
                println!("ready.");
            });

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
            let _thread = spawn(move || {
                let elem = cmd.split(" ").collect::<Vec<_>>();
                let obf = elem[1];
                let ban = bitboard::BitBoard::from_obf(obf).unwrap();
                let _alpha = elem[2].parse::<f32>().unwrap();
                let _beta = elem[3].parse::<f32>().unwrap();
                let depth = ban.nblank() as u8;
                let _precision = elem[4].parse::<f32>().unwrap();
                // eprintln!("{obf} {_alpha}, {_beta}, {depth}, {_precision}");
                let st = Instant::now();
                let (val, node) =
                    nodebb::NodeBB::think_ab_simple(&ban, depth).unwrap();
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
            });

            return Ok(false);
        }

        if body.starts_with("stop") {
            // stop thinking.
            println!("ready.");
            self.log(body).unwrap();
            return Ok(false);
        }

        if body.starts_with("get-search-infos") {
            let running = self.running.load(Ordering::Relaxed);
            if running {
                println!("ok.");
            } else {
                println!("ready.");
            }
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
        self.log(path).unwrap();

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
    ply1 : Option<Child>,
    ply2 : Option<Child>,
    turn : i8,
}

impl OthelloEngineProtocolServer {
    pub fn new1(ch : Child) -> Self {
        OthelloEngineProtocolServer {
            ply1 : Some(ch),
            ply2 : None,
            turn : board::NONE,
        }
    }

    pub fn new2(ch1 : Child, ch2 : Child) -> Self {
        OthelloEngineProtocolServer {
            ply1 : Some(ch1),
            ply2 : Some(ch2),
            turn : board::NONE,
        }
    }

    pub fn setturn(&mut self, trn : i8) {self.turn = trn;}

    fn selectplayer(&mut self) -> Result<&mut Child, String> {
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

    pub fn init(&mut self) -> Result<(), String> {
        let (toeng, fromeng) = self.getio()?;

        if let Err(e) = toeng.write("ENGINE-PROTOCOL init\n".as_bytes()) {
            return Err(e.to_string());
        }

        sleep(Duration::from_millis(1));

        let mut bufreader = BufReader::new(fromeng);
        let mut buf = String::default();
        bufreader.read_line(&mut buf).unwrap();
        if buf == "ready.\n" {
            return Ok(())
        }
        Err(format!("unknown response ii: \"{buf}\""))
    }

    pub fn get_version(&mut self) -> Result<String, String> {
        let (toeng, fromeng) = self.getio()?;

        if let Err(e) =
                toeng.write("ENGINE-PROTOCOL get-version\n".as_bytes()) {
            return Err(e.to_string());
        }

        sleep(Duration::from_millis(1));

        let mut bufreader = BufReader::new(fromeng);
        let mut buf = String::default();
        bufreader.read_line(&mut buf).unwrap();
        let ret = buf.trim().to_string();
        buf.clear();
        bufreader.read_line(&mut buf).unwrap();
        if buf == "ready.\n" {
            return Ok(ret)
        }
        Err(format!("unknown response gv: \"{buf}\" \"{ret}\""))
    }

    pub fn new_position(&mut self) -> Result<(), String> {
        let (toeng, fromeng) = self.getio()?;

        if let Err(e) =
                toeng.write("ENGINE-PROTOCOL new-position\n".as_bytes()) {
            return Err(e.to_string());
        }

        sleep(Duration::from_millis(1));

        let mut bufreader = BufReader::new(fromeng);
        let mut buf = String::default();
        bufreader.read_line(&mut buf).unwrap();
        if buf == "ready.\n" {
            return Ok(())
        }
        Err(format!("unknown response np: \"{buf}\""))
    }

    pub fn midgame_search(&mut self,
            obf : &str, alpha : f32, beta : f32, depth : u8, precision : i8)
            -> Result<String, String> {
        let (toeng, fromeng) = self.getio()?;

        if let Err(e) = toeng.write(
            format!(
                "ENGINE-PROTOCOL midgame-search {obf} {alpha} {beta} {depth} {precision}\n"
            ).as_bytes()) {
            return Err(e.to_string());
        }

        sleep(Duration::from_millis(1));

        let mut bufreader = BufReader::new(fromeng);
        let mut buf = String::default();
        bufreader.read_line(&mut buf).unwrap();
        let ret = buf.trim().to_string();
        buf.clear();
        bufreader.read_line(&mut buf).unwrap();
        if buf == "ready.\n" {
            return Ok(ret)
        }
        Err(format!("unknown response ms: \"{buf}\" \"{ret}\""))
    }

    // pub fn midgame_search_thr(&mut self,
    //         obf : &str, alpha : f32, beta : f32, depth : u8, precision : i8)
    //          -> Result<String, String> {
    //     let (toeng, fromeng) = self.getio()?;

    //     if let Err(e) = toeng.write(
    //         format!(
    //             "ENGINE-PROTOCOL midgame-search {obf} {alpha} {beta} {depth} {precision}\n"
    //         ).as_bytes()) {
    //         return Err(e.to_string());
    //     }

    //     sleep(Duration::from_millis(1));

    //     let finished = Arc::new(AtomicBool::new(false));
    //     let finishthread = finished.clone();
    //     // let outfd = toeng.as_raw_fd();
    //     let thread = spawn(move || {
    //         // let mut toeng = unsafe {std::fs::File::from_raw_fd(outfd)};
    //         loop {
    //             for _i in 0..100 {
    //                 let fin = finishthread.load(Ordering::Relaxed);
    //                 if fin {return;}

    //                 std::thread::sleep(Duration::from_millis(10));
    //             }
    //             toeng.write("\n".as_bytes()).unwrap();
    //         }
    //     });
    //     let mut bufreader = BufReader::new(fromeng);
    //     let mut buf = String::default();
    //     let mut ret = String::default();
    //     loop {
    //         buf.clear();
    //         bufreader.read_line(&mut buf).unwrap();
    //         eprint!("recv:{buf}");
    //         let resp = buf.trim();
    //         if resp == "ok." {
    //             continue;
    //         } else if resp == "ready." {
    //             if ret.is_empty() {continue;}

    //             finished.store(true, Ordering::Relaxed);
    //             // drop(bufreader);
    //             thread.join().unwrap();
    //             let _ = toeng;
    //             return Ok(ret);
    //         } else if resp.is_empty() {
    //             // error
    //             finished.store(true, Ordering::Relaxed);
    //             // drop(bufreader);
    //             thread.join().unwrap();
    //             let _ = toeng;
    //             return Err(format!("unknown response mgs: \"{resp}\""));
    //         }
    //         ret = resp.to_string();
    //     }

    // }

    pub fn endgame_search(&mut self) {unimplemented!()}

    pub fn get_serach_infos(&mut self) {unimplemented!()}

    pub fn stop(&mut self) -> Result<(), String> {
        let (toeng, fromeng) = self.getio()?;

        if let Err(e) = toeng.write("ENGINE-PROTOCOL stop\n".as_bytes()) {
            return Err(e.to_string());
        }

        sleep(Duration::from_millis(1));

        let mut bufreader = BufReader::new(fromeng);
        let mut buf = String::default();
        bufreader.read_line(&mut buf).unwrap();
        if buf == "ready.\n" {
            return Ok(())
        }
        Err(format!("unknown response sp: \"{buf}\""))
    }

    pub fn empty_hash(&mut self) -> Result<(), String> {
        let (toeng, fromeng) = self.getio()?;

        if let Err(e) =
                toeng.write("ENGINE-PROTOCOL empty-hash\n".as_bytes()) {
            return Err(e.to_string());
        }

        sleep(Duration::from_millis(1));

        let mut bufreader = BufReader::new(fromeng);
        let mut buf = String::default();
        bufreader.read_line(&mut buf).unwrap();
        if buf == "ready.\n" {
            return Ok(())
        }
        Err(format!("unknown response eh: \"{buf}\""))
    }

    pub fn quit(&mut self) -> Result<(), String> {
        let (toeng, _fromeng) = self.getio()?;

        if let Err(e) = toeng.write("ENGINE-PROTOCOL quit\n".as_bytes()) {
            return Err(e.to_string());
        }

        sleep(Duration::from_millis(1));

        Ok(())
    }
}
