This a terminal based csv manipulator written in Rust. It's extremely personalised and I wouldn't recommend it for large files, I mainly use it for a small TODO list.

The keybindings are inspired by Vim but are not exactly the same.

For visual mode:
h, j, k, l == regular movement one cell at a time
g == go to first row in the file
G == go to last row in the file
I == go to first column
A == go to last column
i == insert mode
n == create new column after the current one
N == create new column before the current one
H == edit column name

For insert mode:
<Enter> == exit insert mode
