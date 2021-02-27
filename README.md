Emergence is a language for writing genetic circuits, which describe the decision-making of a cell. The program is written in the form of logic functions, which describe boolean logic between variables and produce an output. These functions together form an abstract logic circuit which is then converted to a genetic circuit.

### Example program

```rust
func main(in1, in2) {
	let o = nor(in1, in2);
	out o;
}

test main(TetR, LacI) {
	@300
	TetR = true;
	LacI = true;
}
```

After parsing a program into a parse tree, the compiler creates an abstract logic circuit:

<img src="./images/NOR.svg" width="300" />

which then gets converted to genetic circuit by the Assigner:

<img src="./images/bio-gate-example.svg" width="600" />

### Testbench

TBA
