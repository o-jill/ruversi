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
  InitPos,
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
    pub depth : u8,
    pub eta : Option<f32>,
    pub evaltable1 : String,
    pub evaltable2 : String,
    pub initpos : String,
    pub mode : Mode,
    pub n : Option<usize>,
    pub opponent : Opponent,
    pub repeat : Option<usize>,
    pub rfen : String,
    pub think : String,  // "all", "ab"
    pub turn : i8,  // SENTE, GOTE
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
    /// - depth: 7
    /// - eta: None
    /// - evaltable1: ""
    /// - evaltable2: ""
    /// - initpos: ""
    /// - mode: Mode::None
    /// - n: None
    /// - opponent: Opponent::None
    /// - repeat: None
    /// - rfen: ""
    /// - think: ""
    pub fn new(args: Vec<String>) -> MyOption {
        let mut opt = MyOption {
            depth : 7,
            eta : None,
            evaltable1 : String::new(),
            evaltable2 : String::new(),
            initpos: String::new(),
            mode : Mode::None,
            n : None,
            opponent: Opponent::None,
            repeat : None,
            rfen : String::new(),
            think : String::new(),
            turn : board::NONE,
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
            } else if e == "--depth" {
                old = e;
            } else if e == "--help" || e == "-h" {
                opt.mode = Mode::Help;
            } else if e == "--initpos" {
                opt.mode = Mode::InitPos;
                old = e;
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
                } else if old == "--depth" {
                    match i32::from_str_radix(&e, 10) {
                        Ok(dep) => {
                            if dep <= 0 || dep > 60 {
                                println!("depth {dep} is invalid number.");
                            } else {
                                opt.depth = dep as u8;
                            }
                        },
                        Err(err) => {
                            println!("failed read {} {}. ({})", old, e, err);
                        }
                    }
                } else if old == "--initpos" {
                    opt.initpos = e;
                } else {
                    println!("unknown option: {}", e);
                }
            }
        }
        opt
    }
}

/// Show command options.
/// # Arguments
/// * `str` - some message to be put before options.
pub fn showhelp(msg : &str) {
    println!("{}\n", msg);

    println!("options:
  --duel     play games from some situations with evaltable1 and 2.
  --Edax     play against Edax instead of you. please use with --play(bw).
  --genkifu  set generatin kifu mode. default.
  --help or -h  show this help.
  --learn    set lerning mode. default.
  --play     play a game agaist you. turn is random.
  --playb    play a game agaist you. your turn is black(SENTE).
             you can use w/ --Edax to make ruversi black.
  --playw    play a game agaist you. your turn is white(GOTE).
             you can use w/ --Edax to make ruversi white.
  --rfen <rfen>  think from rfen for debug. don't forget \"\" not to be recognized as 2 part.

  Common:
    --thinkab   use alpha-beta pruning. default.
    --thinkall  search every node. (no pruning)
    --depth x   searching depth. default 7.
  Duel:
    --ev1 <path>  a file for board evaluation.
    --ev2 <path>  a file for board evaluation.
  GenKifu:
    -Nx  initial board group x for generating kifu. 0~9.
        all of the initial board positions will be used when this option is not specified.
  Learn:
    --repeat <number>  number of learning. default 10000.
    --eta <ratio>      learning ratio. default 0.0001.
");
}