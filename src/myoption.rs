use super::*;

#[derive(Debug, PartialEq)]
pub enum Mode {
  None,
  GenKifu,
  Learn,
  Duel,
  RFEN,
  Play,
  Help,
}

#[derive(Debug, PartialEq)]
pub enum Opponent {
    None,
    CUI,
    Edax,
}

/// Options specified in command line args.
/// See 'options:' section in Readme.md.
#[derive(Debug)]
pub struct MyOption {
    pub n : Option<usize>,
    pub repeat : Option<usize>,
    pub eta : Option<f32>,
    pub mode : Mode,
    pub evaltable1 : String,
    pub evaltable2 : String,
    pub think : String,  // "all", "ab"
    pub rfen : String,
    pub turn : i8,  // SENTE, GOTE
    pub opponent : Opponent,
}

impl MyOption {
    /// instantiate MyOption.
    ///
    /// # Arguments
    /// * `args` - args from command line.
    ///
    /// # Return value
    /// instance of MyOptions.<br>
    /// default:<br>
    /// - n: None
    /// - repeat: None
    /// - eta: None
    /// - mode: Mode::None
    /// - evaltable1: ""
    /// - evaltable2: ""
    /// - think: ""
    /// - rfen: ""
    /// - opponent: Opponent::None
    pub fn new(args: Vec<String>) -> MyOption {
        let mut opt = MyOption {
            n : None,
            repeat : None,
            eta : None,
            mode : Mode::None,
            evaltable1 : String::new(),
            evaltable2 : String::new(),
            think : String::new(),
            rfen : String::new(),
            turn : board::NONE,
            opponent: Opponent::None,
        };
        let mut old = String::new();
        for e in args {
            if e == "--genkifu" {
                opt.mode = Mode::GenKifu;
            } else if e == "--learn" {
                opt.mode = Mode::Learn;
            } else if e == "--duel" {
                opt.mode = Mode::Duel;
            } else if e == "--play" {
                opt.mode = Mode::Play;
                if opt.opponent == Opponent::None {
                    opt.opponent = Opponent::CUI;
                }
            } else if e == "--playb" {
                opt.mode = Mode::Play;
                opt.turn = board::SENTE;
                if opt.opponent == Opponent::None {
                    opt.opponent = Opponent::CUI;
                }
            } else if e == "--playw" {
                opt.mode = Mode::Play;
                opt.turn = board::GOTE;
                if opt.opponent == Opponent::None {
                    opt.opponent = Opponent::CUI;
                }
            } else if e == "--rfen" {
                opt.mode = Mode::RFEN;
                old = e;
            } else if e == "--help" || e == "-h" {
                opt.mode = Mode::Help;
            } else if e == "--thinkab" {
                opt.think = "ab".to_string();
            } else if e == "--thinkall" {
                opt.think = "all".to_string();
            } else if e == "--repeat" {
                old = e;
            } else if e == "--eta" {
                old = e;
            } else if e == "--ev1" {
                old = e;
            } else if e == "--ev2" {
                old = e;
            } else if e == "--Edax" {
                opt.opponent = Opponent::Edax;
            } else if e.find("-N").is_some() {
                let n : Vec<&str> = e.split("N").collect();
                let n = n[1].parse::<usize>();
                if n.is_ok() {
                    opt.n = Some(n.unwrap());
                } else {
                    println!("invalid option: {}", e);
                }
            } else  {
                if old == "--repeat" {
                    let rpt = e.parse::<usize>();
                    if rpt.is_err() {
                        println!("invalid option: {} {}", old, e);
                    } else {
                        opt.repeat = Some(rpt.unwrap());
                    }
                    old.clear();
                } else if old == "--eta" {
                    let eta = e.parse::<f32>();
                    if eta.is_err() {
                        println!("invalid option: {} {}", old, e);
                    } else {
                        opt.eta = Some(eta.unwrap());
                    }
                    old.clear();
                } else if old == "--ev1" {
                    if std::path::Path::new(&e).exists() {
                        opt.evaltable1 = e;
                    } else {
                        println!("failed find \"{}\".", e);
                    }
                } else if old == "--ev2" {
                    if std::path::Path::new(&e).exists() {
                        opt.evaltable2 = e;
                    } else {
                        println!("failed find \"{}\".", e);
                    }
                } else if old == "--rfen" {
                    opt.rfen = e;
                } else {
                    println!("unknown option: {}", e);
                }
            }
        }
        opt
    }
}
