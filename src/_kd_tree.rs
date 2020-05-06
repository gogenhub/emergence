use std::collections::{HashMap, HashSet};

#[derive(Deserialize, Serialize, Clone)]
pub struct LeafNode {
	pub name: String,
	pub parent: Option<String>,
	pub point: [f64; 2],
}

impl LeafNode {
	pub fn new(name: String, x: f64, y: f64) -> Self {
		Self {
			name: name,
			parent: None,
			point: [x, y],
		}
	}

	pub fn dist(&self, n: &LeafNode) -> f64 {
		((n.point[0] - self.point[0]).powi(2) + (n.point[1] - self.point[1]).powi(2)).sqrt()
	}

	pub fn contains(&self, n: &LeafNode) -> bool {
		n.point[0] > self.point[0] && n.point[1] < self.point[1]
	}
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
enum Node {
	Leaf(LeafNode),
	Internal(InternalNode),
}

impl Node {
	pub fn is_leaf(&self) -> bool {
		matches!(*self, Node::Leaf(_))
	}

	pub fn leaf(&self) -> &LeafNode {
		match self {
			Node::Leaf(n) => n,
			_ => panic!("Node is not a leaf!"),
		}
	}

	fn internal(&self) -> &InternalNode {
		match self {
			Node::Internal(n) => n,
			_ => panic!("Node is not a internal!"),
		}
	}
}

#[derive(Deserialize, Serialize, Clone)]
struct InternalNode {
	parent: Option<String>,
	div: f64,
	less: String,
	more: String,
}

#[derive(Deserialize, Serialize)]
pub struct KdTree {
	k: u8,
	tree: HashMap<String, Node>,
	root: Option<String>,
	#[serde(default)]
	strict: bool,
}

fn get_closer<'a>(
	first: Option<&'a LeafNode>,
	second: Option<&'a LeafNode>,
	point: &LeafNode,
) -> Option<&'a LeafNode> {
	if first.is_none() {
		return second;
	}

	if second.is_none() {
		return first;
	}

	if first.unwrap().dist(point) < second.unwrap().dist(point) {
		return first;
	} else {
		return second;
	}
}

fn get_sides(in_node: &InternalNode, point: &LeafNode, axis: usize) -> (String, String) {
	match point.point[axis] < in_node.div {
		true => (in_node.less.to_owned(), in_node.more.to_owned()),
		_ => (in_node.more.to_owned(), in_node.less.to_owned()),
	}
}

impl KdTree {
	fn should_check_bad_side(
		&self,
		closest: Option<&LeafNode>,
		node: &LeafNode,
		div: f64,
		axis: usize,
	) -> bool {
		if closest.is_none() {
			return true;
		}

		if self.strict {
			if axis == 0 && (div < node.point[axis]) {
				return false;
			}

			if axis == 1 && (div > node.point[axis]) {
				return false;
			}
		}

		(node.point[axis] - div).abs() < node.dist(closest.unwrap())
	}

	fn walk(
		&self,
		blacklist: &HashSet<String>,
		point: &LeafNode,
		curr: String,
		depth: u8,
	) -> Option<&LeafNode> {
		let node = self.tree.get(&curr).unwrap();

		if node.is_leaf() {
			let group: Vec<&str> = curr.split("_").collect();
			if self.strict && point.contains(node.leaf()) {
				return None;
			}
			if blacklist.contains(group[1]) {
				return None;
			}
			return Some(node.leaf());
		}

		let axis = (depth % self.k) as usize;

		let in_node = node.internal();
		let (good_side, bad_side) = get_sides(in_node, point, axis);
		let mut closest = self.walk(blacklist, point, good_side, depth + 1);
		if self.should_check_bad_side(closest, point, in_node.div, axis) {
			let closest_bad_side = self.walk(blacklist, point, bad_side, depth + 1);
			closest = get_closer(closest, closest_bad_side, point);
		}

		return closest;
	}

	pub fn search(&mut self, x: f64, y: f64, blacklist: &mut HashSet<String>) -> Option<LeafNode> {
		if self.root.as_ref().is_none() {
			return None;
		}

		let closest = self
			.walk(
				blacklist,
				&LeafNode::new("new".to_owned(), x, y),
				self.root.as_ref().unwrap().to_owned(),
				0,
			)
			.cloned();
		if closest.is_some() {
			let n = closest.as_ref().unwrap();
			let group: Vec<&str> = n.name.split("_").collect();
			blacklist.insert(group[1].to_owned());
		}
		closest
	}
}
