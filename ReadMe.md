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
* --duel  
  play games from some situations with evaltable1 and 2.  
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
* --learn  
  set lerning mode.
* --repeat \<number>  
  number of learning. default 10000.
* --eta \<ratio>  
  learning ratio. default 0.0001.
* -- rfen \<rfen>  
  think from rfen for debug. don't forget "" not to be recognized as 2 part.  

| command option | generating kifu | learning |
|:--------------:|:---------------:|:--------:|
| none           |             yes |      yes |
| --learn        |              no |      yes |
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
* (reserved)nnv3  
* avx  
  use AVX floating-point SIMD instructions.  
* bitboard (default)  
  use bitboard instead of byteboard.  

---
