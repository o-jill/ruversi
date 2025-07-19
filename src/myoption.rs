use super::*;

#[derive(Debug, PartialEq)]
pub enum Mode {
  None,
  GenKifu,
  Learn,
  Duel,
  DuelExt,
  Gtp,
  Oep,
  Rfen,
  Play,
  Help,
  InitPos,
  Equal,
}

#[derive(Debug, PartialEq)]
pub enum Opponent {
    None,
    Cui,
    Edax,
    Ruversi,
    Cassio,
}

#[derive(Debug, PartialEq)]
pub enum TrainingMode {
    OneByOne,  // "Oneline" in other words
    MiniBatch,
}


#[derive(Debug, PartialEq)]
pub enum Verbose {
    Silent,
    Normal,
    Verbose,
}

impl Verbose {
    pub fn is_silent(&self) -> bool {self.eq(&Verbose::Silent)}
    pub fn is_normal(&self) -> bool {self.eq(&Verbose::Normal)}
    pub fn is_verbose(&self) -> bool {self.eq(&Verbose::Verbose)}
}

/// Options specified in command line args.
/// See 'options:' section in Readme.md.
#[derive(Debug)]
pub struct MyOption {
    pub depth : u8,
    pub eta : Option<f32>,
    pub duellv : i8,
    pub edaxconfig : String,
    pub evaltable1 : String,
    pub evaltable2 : String,
    pub initpos : String,
    pub mode : Mode,
    pub n : Option<usize>,
    pub opponent : Opponent,
    pub repeat : Option<usize>,
    pub rfen : String,
    pub progress : Vec<u32>,
    pub think : String,  // "all", "ab"
    pub outtrain : String,  // progress,exrfens,summary
    pub turn : i8,  // SENTE, GOTE
    pub trmode : TrainingMode,
    pub minibsize : usize,
    pub verbose : Verbose,
    pub treedump : Option<String>,
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
    /// - duellv: 5
    /// - evaltable1: ""
    /// - evaltable2: ""
    /// - initpos: ""
    /// - mode: Mode::None
    /// - n: None
    /// - opponent: Opponent::None
    /// - outtrain: ""
    /// - progress: []
    /// - repeat: None
    /// - rfen: ""
    /// - think: ""
    /// - trmode: OneByOne
    /// - verbose: Normal
    pub fn new(args: Vec<String>) -> Result<MyOption, String> {
        let mut opt = MyOption {
            depth : 7,
            eta : None,
            duellv : 5,
            edaxconfig : String::new(),
            evaltable1 : String::new(),
            evaltable2 : String::new(),
            initpos: String::new(),
            mode : Mode::None,
            n : None,
            opponent: Opponent::None,
            outtrain: String::new(),
            progress: Vec::new(),
            repeat : None,
            rfen : String::new(),
            think : String::new(),
            turn : board::NONE,
            trmode : TrainingMode::OneByOne,
            minibsize : 128,
            verbose : Verbose::Normal,
            treedump : None,
        };
        let mut old = String::new();
        let mut skip = 0;
        for i in 1..args.len() {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            let e = args.get(i).unwrap().to_string();
            if e.starts_with("--") {
                if !old.is_empty() {
                    panic!("\"{old}\" was not specified correctly.");
                }
                if e == "--genkifu" {
                    opt.mode = Mode::GenKifu;
                } else if e == "--learn" {
                    opt.mode = Mode::Learn;
                } else if e == "--duel" {
                    opt.mode = Mode::Duel;
                    if let Some(lvl) = args.get(i + 1) {
                        if let Ok(level) = lvl.parse::<i8>() {
                            if level > 0 {
                                opt.duellv = level;
                                skip = 1;
                            }
                        }
                    }
                } else if e == "--duelext" {
                    opt.mode = Mode::DuelExt;
                    if let Some(lvl) = args.get(i + 1) {
                        if let Ok(level) = lvl.parse::<i8>() {
                            if level > 0 {
                                opt.duellv = level;
                                skip = 1;
                            }
                        }
                    }
                } else if e == "--play" {
                    opt.mode = Mode::Play;
                    if opt.opponent == Opponent::None {
                        opt.opponent = Opponent::Cui;
                    }
                } else if e == "--playb" {
                    opt.mode = Mode::Play;
                    opt.turn = board::SENTE;
                    if opt.opponent == Opponent::None {
                        opt.opponent = Opponent::Cui;
                    }
                } else if e == "--playw" {
                    opt.mode = Mode::Play;
                    opt.turn = board::GOTE;
                    if opt.opponent == Opponent::None {
                        opt.opponent = Opponent::Cui;
                    }
                } else if e == "--rfen" {
                    opt.mode = Mode::Rfen;
                    old = e;
                } else if [
                        "--depth", "--Edconf", "--eta", "--ev1", "--ev2",
                         "--progress", "--Ruconf", "--repeat", "--trainout",
                    ].contains(&e.as_str()) {
                    old = e;
                } else if e == "--help" || e == "-h" {
                    opt.mode = Mode::Help;
                } else if e == "--initpos" {
                    opt.mode = Mode::InitPos;
                    old = e;
                } else if e == "--equal" {
                    opt.mode = Mode::Equal;
                } else if e == "--thinkab" {
                    opt.think = "ab".to_string();
                } else if e == "--thinkall" {
                    opt.think = "all".to_string();
                } else if e == "--onebyone" {
                    opt.trmode = TrainingMode::OneByOne;
                } else if e == "--minibatch" {
                    old = e;
                    opt.trmode = TrainingMode::MiniBatch;
                } else if e == "--Edax" {
                    opt.opponent = Opponent::Edax;
                } else if e == "--Ruversi" {
                    opt.opponent = Opponent::Ruversi;
                } else if e == "--Cassio" {
                    opt.opponent = Opponent::Cassio;
                } else if e == "--silent" {
                    opt.verbose = Verbose::Silent;
                } else if e == "--verbose" {
                    opt.verbose = Verbose::Verbose;
                } else if e == "--gtp" {
                    opt.mode = Mode::Gtp;
                } else if e == "--oep" {
                    opt.mode = Mode::Oep;
                } else if e == "--treedump" {
                    old = e;
                    opt.treedump = Some("treeinfo.puml".to_string());
                // } else {
                }
            } else if old.is_empty() && e.starts_with("-") {
                if e.contains("-N") {
                    let n : Vec<&str> = e.split("N").collect();
                    let n = n[1].parse::<usize>();
                    if n.is_ok() {
                        opt.n = Some(n.unwrap());
                    } else {
                        return Err(format!("invalid option: {e}"));
                    }
                }
            } else if old == "--repeat" {
                let rpt = e.parse::<usize>();
                if rpt.is_err() {
                    return Err(format!("invalid option: {old} {e}"));
                } else {
                    opt.repeat = Some(rpt.unwrap());
                }
                old.clear();
            } else if old == "--progress" {
                opt.progress =
                    e.split(",").collect::<Vec<&str>>().iter().map(|&a| {
                        a.parse::<u32>().unwrap()}).collect();
                old.clear();
            } else if old == "--minibatch" {
                let mbs = e.parse::<usize>();
                if mbs.is_err() {
                    return Err(format!("invalid option: {old} {e}"));
                } else {
                    opt.minibsize = mbs.unwrap();
                }
                old.clear();
            } else if old == "--Edconf" || old == "--Ruconf" {
                if std::path::Path::new(&e).exists() {
                    opt.edaxconfig = e;
                } else {
                    return Err(format!("failed find \"{e}\"."));
                }
                old.clear();
            } else if old == "--eta" {
                let eta = e.parse::<f32>();
                if eta.is_err() {
                    return Err(format!("invalid option: {old} {e}"));
                } else {
                    opt.eta = Some(eta.unwrap());
                }
                old.clear();
            } else if old == "--ev1" {
                if std::path::Path::new(&e).exists() || "RANDOM" == e {
                    opt.evaltable1 = e;
                } else {
                    return Err(format!("failed find \"{e}\"."));
                }
                old.clear();
            } else if old == "--ev2" {
                if std::path::Path::new(&e).exists() || "RANDOM" == e {
                    opt.evaltable2 = e;
                } else {
                    return Err(format!("failed find \"{e}\"."));
                }
                old.clear();
            } else if old == "--rfen" {
                opt.rfen = e;
                old.clear();
            } else if old == "--depth" {
                match e.parse::<i32>() {
                    Ok(dep) => {
                        if dep <= 0 || dep > 60 {
                            return Err(format!("depth {dep} is invalid number."));
                        } else {
                            opt.depth = dep as u8;
                        }
                    },
                    Err(err) => {
                        return Err(format!("failed read {old} {e}. ({err})"));
                    }
                }
                old.clear();
            } else if old == "--initpos" {
                opt.initpos = e;
                old.clear();
            } else if old == "--trainout" {
                opt.outtrain = e;
                old.clear();
            } else if old == "--treedump" {
                opt.treedump = Some(e.to_string());
                old.clear();
            } else {
                println!("unknown option: {e}");
            }
        }
        Ok(opt)
    }
}

/// Show command options.
/// # Arguments
/// * `str` - some message to be put before options.
pub fn showhelp(msg : &str) {
    println!("{msg}\n");

    println!("options:
  --duel N   play games from some situations with evaltable1 and 2. N is optional. 1 ~ default 5 ~ 13.
  --duelext N   play games from some situations against Edax. N is optional. 1 ~ default 5 ~ 13.
  --genkifu  set generatin kifu mode. default.
  --help or -h  show this help.
  --play     play a game agaist you. turn is random.
  --playb    play a game agaist you. your turn is black(SENTE).
  you can use w/ --Edax to make ruversi black.
  --playw    play a game agaist you. your turn is white(GOTE).
  you can use w/ --Edax to make ruversi white.
  --rfen <rfen>  think from rfen for debug. don't forget \"\" not to be recognized as 2 part.
  --gtp      go text protocol mode.
  --oep      othello engine protocol mode.
  --learn    [deprecated]set lerning mode. default.

  Common:
    --thinkab   use alpha-beta pruning. default.
    --thinkall  search every node. (no pruning)
    --depth x   searching depth. default 7.
    --silent    reduce console outputs.
  Duel:
    --ev1 <path>  a file for board evaluation.
    --ev2 <path>  a file for board evaluation.
  DuelExt:
    --Edconf <path>  a file for edax(or ruversi) path configuration.
    --Ruversi  play against another Ruversi, not against Edax.
    --Cassio   play against edax via othello engine protocol.
  GenKifu:
    -Nx  initial board group x for generating kifu. 0~99.
        all of the initial board positions will be used when this option is not specified.
  Play:
    --Edax     play against Edax instead of you. please use with --play(bw).
    --Edconf <path>  a file for edax path configuration.
  Rfen:
    --treedump <path>  put search tree into a file w/ PlantUML format.
  [deprecated]Learn:
    --repeat <number>  [deprecated]number of learning. default 10000.
    --eta <ratio>      [deprecated]learning ratio. default 0.1.
    --trainout <options>  [deprecated]output control.
        exrfens  : put RFENs in 2nd moves in every kifus.
        nosave   : skip saving weights.
        progress : show progress
        summary  : show input summary.
        time     : show processing time.
        default: progress,summary,time
    --progress <numbers>  [deprecated]storing weight after some iterations as newevaltable.rN.txt.
        default: nothing.
    --onebyone  [deprecated]train w/o minibatch. minibatch=1 in other words. default.
    --minibatch <number>  [deprecated]train w/ minibatch.
        size of minibatch. default 128.
");
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        // デフォルト引数（引数なし）の場合のフィールド値を確認
        let args = vec!["prog".to_string()];
        let opt = MyOption::new(args).unwrap();
        assert_eq!(opt.depth, 7);
        // eta のデフォルト値は None
        assert_eq!(opt.eta, None);
        // duellv のデフォルト値は 5
        assert_eq!(opt.duellv, 5);
        // edaxconfig のデフォルト値は空文字列
        assert_eq!(opt.edaxconfig, "");
        // evaltable1 のデフォルト値は空文字列
        assert_eq!(opt.evaltable1, "");
        // evaltable2 のデフォルト値は空文字列
        assert_eq!(opt.evaltable2, "");
        // initpos のデフォルト値は空文字列
        assert_eq!(opt.initpos, "");
        // mode のデフォルト値は Mode::None
        assert_eq!(opt.mode, Mode::None);
        // n のデフォルト値は None
        assert_eq!(opt.n, None);
        // opponent のデフォルト値は Opponent::None
        assert_eq!(opt.opponent, Opponent::None);
        // repeat のデフォルト値は None
        assert_eq!(opt.repeat, None);
        // rfen のデフォルト値は空文字列
        assert_eq!(opt.rfen, "");
        // progress のデフォルト値は空のベクタ
        assert_eq!(opt.progress, Vec::<u32>::new());
        // think のデフォルト値は空文字列
        assert_eq!(opt.think, "");
        // outtrain のデフォルト値は空文字列
        assert_eq!(opt.outtrain, "");
        // turn のデフォルト値は board::NONE
        assert_eq!(opt.turn, board::NONE);
        // trmode のデフォルト値は TrainingMode::OneByOne
        assert_eq!(opt.trmode, TrainingMode::OneByOne);
        // minibsize のデフォルト値は 128
        assert_eq!(opt.minibsize, 128);
        // verbose のデフォルト値は Verbose::Normal
        assert_eq!(opt.verbose, Verbose::Normal);
        // treedump のデフォルト値は None
        assert_eq!(opt.treedump, None);
    }

    #[test]
    fn test_valid_depth() {
        // --depthオプションで値を指定した場合の動作を確認
        let args = vec!["prog".to_string(), "--depth".to_string(), "10".to_string()];
        let opt = MyOption::new(args).unwrap();
        // depth フィールドが 10 になっているか確認
        assert_eq!(opt.depth, 10);
    }

    #[test]
    fn test_invalid_depth() {
        // 無効なdepth(0)の場合、エラーになることを確認
        let args = vec!["prog".to_string(), "--depth".to_string(), "0".to_string()];
        let err = MyOption::new(args).unwrap_err();
        // エラーメッセージに "invalid number" が含まれることを確認
        assert!(err.contains("invalid number"));
    }

    #[test]
    fn test_eta() {
        // --eta オプションの値のパースを確認
        let args = vec!["prog".to_string(), "--eta".to_string(), "0.5".to_string()];
        let opt = MyOption::new(args).unwrap();
        // eta フィールドが Some(0.5) になっているか確認
        assert_eq!(opt.eta, Some(0.5));
    }

    #[test]
    fn test_silent_option() {
        // --silent オプションで verbose フィールドが Silent になることを確認するテスト
        let args = vec!["prog".to_string(), "--silent".to_string()];
        let opt = MyOption::new(args).unwrap();
        // verbose が Verbose::Silent かどうか確認
        assert_eq!(opt.verbose, Verbose::Silent);
        // .is_silent() メソッドでも確認
        assert!(opt.verbose.is_silent());
    }

    #[test]
    fn test_verbose_option() {
        // --verbose オプションで verbose フィールドが Verbose になることを確認するテスト
        let args = vec!["prog".to_string(), "--verbose".to_string()];
        let opt = MyOption::new(args).unwrap();
        // verbose が Verbose::Verbose かどうか確認
        assert_eq!(opt.verbose, Verbose::Verbose);
        // .is_verbose() メソッドでも確認
        assert!(opt.verbose.is_verbose());
    }


    #[test]
    fn test_ev1_file_not_exist_should_fail() {
        // --ev1 で存在しないファイルを指定した場合、Errが返ることを確認
        let args = vec![
            "prog".to_string(),
            "--ev1".to_string(),
            "this_file_should_not_exist.ev".to_string(),
        ];
        let err = MyOption::new(args).unwrap_err();
        // エラーメッセージに "failed find" やファイル名が含まれていることを確認
        assert!(err.contains("failed find"));
        assert!(err.contains("this_file_should_not_exist.ev"));
    }

    #[test]
    fn test_ev2_file_not_exist_should_fail() {
        // --ev2 で存在しないファイルを指定した場合、Errが返ることを確認
        let args = vec![
            "prog".to_string(),
            "--ev2".to_string(),
            "another_missing_file.ev".to_string(),
        ];
        let err = MyOption::new(args).unwrap_err();
        // エラーメッセージに "failed find" やファイル名が含まれていることを確認
        assert!(err.contains("failed find"));
        assert!(err.contains("another_missing_file.ev"));
    }
}
