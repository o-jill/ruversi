Ruversi  

a reversi game program written in rust.  

# Lisence  
CC0. in other words, Public domain.

# Warranty  
NO Warranty.  

# Command options  
* --evaltable1 <path>  
  a file for boardevaluation.
* --evaltable2 <path>  
  reserved.
* --genkifu  
  set generatin kifu mode.
* -Nx  
  initial board group x for generating kifu. 0~9. all of the initial board positions will be used when this option is not specified.
* --learn  
  set lerning mode.
* --repeat <number>  
  number of learning.
* --eta <ratio>  
  learning ratio.

| command option | generating kifu | learning |
|:--------------:|:---------------:|:--------:|
| none           |             yes |      yes |
| --learn        |              no |      yes |
| --genkifu      |             yes |       no |
| --genkifu --learn |          yes |      yes |

--