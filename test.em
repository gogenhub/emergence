fn not a -> b {
	b = ~a;
}

fn nor (a, b) -> c {
	c = a ~| b;
}

gene main(TetR, AraC) -> RFP {
	let a = not(TetR);
	let b = not(AraC);
	RFP = nor(a, b);
}