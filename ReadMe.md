Ruversi  

a reversi game program written in rust.  

# Lisence  
CC0. in other words, Public domain.

# Warranty  
NO Warranty.  

# Command options  
* --play  
  play a game agaist you. turn is random.  
* --playb  
  play a game agaist you. your turn is black(SENTE).  
* --playw  
  play a game agaist you. your turn is white(GOTE).  
* --Edax  
  play against Edax instead of you. please use with --play(bw).  
* --duel N  
  play games from some situations with evaltable1 and 2. N is optional. default 5.  
* --gtp  
  go text protocol mode.
* --ev1 \<path>  
  a file for board evaluation.
* --ev2 \<path>  
  a file for board evaluation.
* --genkifu  
  set generating kifu mode.
* -Nx  
  initial board group x for generating kifu. 0~9. all of the initial board positions will be used when this option is not specified.
* --thinkab  
  use alpha-beta pruning. default.
* --thinkall  
  search every node. (no pruning)
* --rfen \<rfen>  
  think from rfen for debug. don't forget putting "" to an RFEN not to be recognized as 2 part.  
* --help  
  show help.  
* --silent  
  reduce console outputs.
* --learn  
  [deprecated] set lerning mode.
* --repeat \<number>  
  [deprecated] number of learning. default 10000.
* --eta \<ratio>  
  [deprecated] learning ratio. default 0.0001.

| command option | generating kifu | learning |
|:--------------:|:---------------:|:--------:|
| none           |             yes |       no |
| [deprecated] --learn |        no |      yes |
| --genkifu      |             yes |       no |
| --genkifu --learn |          yes |      yes |

# Compile options(features)  
* nosimd  
  stop using simd instructions(SSE) for evaluation.  
  SSE floating-point SIMD instructions will be used when no features are specified.  
* nnv1  
  evaluate with neural network version 1.  
* nnv2  
  evaluate with neural network version 2.  
* nnv3  
  evaluate with neural network version 3.  
* nnv4  
  evaluate with neural network version 4.  
* (reserved)nnv5  
* avx  
  use AVX floating-point SIMD instructions.  
* bitboard (default)  
  use bitboard instead of byteboard.  

---
