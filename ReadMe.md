Ruversi  

a reversi game program written in rust.  

# Lisence  
CC0. in other words, Public domain.

# Warranty  
NO Warranty.  

# Command options  
* --duel  
  play games from some situations with evaltable1 and 2.  
* --evaltable1 \<path>  
  a file for boardevaluation.
* --evaltable2 \<path>  
  reserved.
* --genkifu  
  set generatin kifu mode.
* -Nx  
  initial board group x for generating kifu. 0~9. all of the initial board positions will be used when this option is not specified.
* --thinkab  
  use alpha-beta prunning. default.
* --thinkall  
  search every node. (no prunning)
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

---
