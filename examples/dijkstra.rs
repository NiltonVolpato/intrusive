use std::cell::Cell;

use priority_queue::{
    IntrusiveNode, IntrusivePriorityQueue,
    binary_heap::{BinaryHeapHook, IntrusiveBinaryHeap},
};

pub struct Vertex {
    pub id: usize,
    pub edges: Vec<(usize, u32)>, // (neighbor_id, edge_weight)
    pub dist: Cell<u32>,
    heap_hook: BinaryHeapHook,
}

impl Vertex {
    pub fn new(id: usize, edges: Vec<(usize, u32)>) -> Self {
        Self {
            id,
            edges,
            dist: Cell::new(u32::MAX),
            heap_hook: BinaryHeapHook::new(),
        }
    }
}

impl IntrusiveNode for Vertex {
    type Priority = u32;
    type Hook = BinaryHeapHook;

    #[inline]
    fn priority(&self) -> Self::Priority {
        self.dist.get()
    }

    #[inline]
    fn set_priority(&self, priority: Self::Priority) {
        self.dist.set(priority);
    }

    #[inline]
    fn hook(&self) -> &BinaryHeapHook {
        &self.heap_hook
    }
}

fn print_distances(graph: &[Vertex]) {
    print!("   -");
    for vertex in graph {
        let dist = vertex.priority();
        if dist == u32::MAX {
            print!(" ∞");
        } else {
            print!(" {}", dist);
        }
    }
    println!();
}

// ---------------------------------------------------------------------------
// Executing Dijkstra with Zero Extra Allocations
// ---------------------------------------------------------------------------
fn run_dijkstra(graph: &[Vertex], source_id: usize) {
    let mut pq = IntrusiveBinaryHeap::new();

    graph[source_id].set_priority(0);
    for vertex in graph.iter() {
        pq.push(vertex);
    }

    while let Some(u) = pq.pop() {
        println!("## {}", u.id);
        let u_dist = u.priority();
        if u_dist == u32::MAX {
            break; // Unreachable remaining components
        }

        print_distances(graph);
        for &(v_id, weight) in &u.edges {
            let v = &graph[v_id];
            let alt = u_dist + weight;

            if alt < v.priority() {
                // In-place sift-up via intrusive trait
                let _ = pq.decrease_key(v, alt);
            }
            print_distances(graph);
        }
    }
}

fn main() {
    // Example graph represented as an adjacency list
    let graph = vec![
        Vertex::new(0, vec![(1, 4), (2, 1), (3, 5)]),
        Vertex::new(1, vec![(3, 1)]),
        Vertex::new(2, vec![(1, 2), (3, 5)]),
        Vertex::new(3, vec![]),
    ];

    run_dijkstra(&graph, 0);

    println!();
    println!("{:^8} {:^8}", "Vertex", "Distance");
    for vertex in &graph {
        println!("{:^8} {:^8}", vertex.id, vertex.priority());
    }
}
