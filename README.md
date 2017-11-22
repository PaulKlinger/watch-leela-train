# watch leela train

A small tool to view [leela zero](https://github.com/gcp/leela-zero)'s training games live. This just adds a go board in the console output ("o" for white stones, "x" for black ones, "." for empty locations).

Launch it in the directory containing the "autogtp" executable. Any command line arguments will be passed on to autogtp, if none are supplied the command `./autogtp -k sgfs` (i.e. save the sgf files in the directory `sgfs`) is executed.

A Windows .exe is available [here](https://github.com/PaulKlinger/watch-leela-train/releases/download/v0.2/watch_leela_train.exe).
