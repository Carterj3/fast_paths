/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

use std::collections::BinaryHeap;

use crate::constants::Weight;
use crate::constants::WEIGHT_MAX;
use crate::constants::{Edge, EdgeId, Node, NodeId};
use crate::fast_graph::FastGraph;
use crate::heap_item::HeapItem;
use crate::shortest_path::ShortestPath;
use crate::valid_flags::ValidFlags;

pub struct PathCalculator {
    num_nodes: usize,
    data_fwd: Vec<Data>,
    data_bwd: Vec<Data>,
    valid_flags_fwd: ValidFlags,
    valid_flags_bwd: ValidFlags,
    heap_fwd: BinaryHeap<HeapItem>,
    heap_bwd: BinaryHeap<HeapItem>,
}

impl PathCalculator {
    pub fn new(num_nodes: usize) -> Self {
        PathCalculator {
            num_nodes,
            data_fwd: (0..num_nodes).map(|_i| Data::new()).collect(),
            data_bwd: (0..num_nodes).map(|_i| Data::new()).collect(),
            valid_flags_fwd: ValidFlags::new(num_nodes),
            valid_flags_bwd: ValidFlags::new(num_nodes),
            heap_fwd: BinaryHeap::new(),
            heap_bwd: BinaryHeap::new(),
        }
    }

    pub fn calc_path(
        &mut self,
        graph: &FastGraph,
        start: NodeId,
        end: NodeId,
    ) -> Option<ShortestPath> {
        assert_eq!(
            graph.get_num_nodes(),
            self.num_nodes,
            "given graph has invalid node count"
        );
        assert!(start < self.num_nodes, "invalid start node");
        assert!(end < self.num_nodes, "invalid end node");
        self.heap_fwd.clear();
        self.heap_bwd.clear();
        self.valid_flags_fwd.invalidate_all();
        self.valid_flags_bwd.invalidate_all();
        if start == end {
            return Some(ShortestPath::singular(start));
        }

        self.update_node_fwd(start, 0, Node::Invalid, Edge::Invalid);
        self.update_node_bwd(end, 0, Node::Invalid, Edge::Invalid);
        self.heap_fwd.push(HeapItem::new(0, start));
        self.heap_bwd.push(HeapItem::new(0, end));

        let mut best_weight = WEIGHT_MAX;
        let mut meeting_node = Node::Invalid;

        loop {
            if self.heap_fwd.is_empty() && self.heap_bwd.is_empty() {
                break;
            }
            loop {
                if self.heap_fwd.is_empty() {
                    break;
                }
                let curr = self.heap_fwd.pop().unwrap();
                if self.is_settled_fwd(curr.node_id) {
                    continue;
                }
                if curr.weight > best_weight {
                    break;
                }
                let begin = graph.begin_out_edges(curr.node_id);
                let end = graph.end_out_edges(curr.node_id);
                for edge_id in begin..end {
                    let adj = graph.edges_fwd[edge_id].adj_node;
                    let edge_weight = graph.edges_fwd[edge_id].weight;
                    let weight = curr.weight + edge_weight;
                    if weight < self.get_weight_fwd(adj) {
                        self.update_node_fwd(
                            adj,
                            weight,
                            Node::Node(curr.node_id),
                            Edge::Edge(edge_id),
                        );
                        self.heap_fwd.push(HeapItem::new(weight, adj));
                    }
                }
                self.data_fwd[curr.node_id].settled = true;
                if self.valid_flags_bwd.is_valid(curr.node_id)
                    && curr.weight + self.get_weight_bwd(curr.node_id) < best_weight
                {
                    best_weight = curr.weight + self.get_weight_bwd(curr.node_id);
                    meeting_node = Node::Node(curr.node_id);
                }
                break;
            }

            loop {
                if self.heap_bwd.is_empty() {
                    break;
                }
                let curr = self.heap_bwd.pop().unwrap();
                if self.is_settled_bwd(curr.node_id) {
                    continue;
                }
                if curr.weight > best_weight {
                    break;
                }
                let begin = graph.begin_in_edges(curr.node_id);
                let end = graph.end_in_edges(curr.node_id);
                for edge_id in begin..end {
                    let adj = graph.edges_bwd[edge_id].adj_node;
                    let edge_weight = graph.edges_bwd[edge_id].weight;
                    let weight = curr.weight + edge_weight;
                    if weight < self.get_weight_bwd(adj) {
                        self.update_node_bwd(
                            adj,
                            weight,
                            Node::Node(curr.node_id),
                            Edge::Edge(edge_id),
                        );
                        self.heap_bwd.push(HeapItem::new(weight, adj));
                    }
                }
                self.data_bwd[curr.node_id].settled = true;
                if self.valid_flags_fwd.is_valid(curr.node_id)
                    && curr.weight + self.get_weight_fwd(curr.node_id) < best_weight
                {
                    best_weight = curr.weight + self.get_weight_fwd(curr.node_id);
                    meeting_node = Node::Node(curr.node_id);
                }
                break;
            }
        }

        match meeting_node {
            Node::Invalid => None,
            Node::Node(id) => {
                let node_ids = self.extract_nodes(graph, start, end, id);
                Some(ShortestPath::new(start, end, best_weight, node_ids))
            }
        }
    }

    fn extract_nodes(
        &self,
        graph: &FastGraph,
        _start: NodeId,
        end: NodeId,
        meeting_node: NodeId,
    ) -> Vec<NodeId> {
        // assert_ne!(meeting_node, Node::Invalid);
        assert!(self.valid_flags_fwd.is_valid(meeting_node));
        assert!(self.valid_flags_bwd.is_valid(meeting_node));

        let mut result = Vec::new();
        let mut node_id = meeting_node;
        while let Edge::Edge(edge_id) = self.data_fwd[node_id].inc_edge {
            PathCalculator::unpack_fwd(graph, &mut result, edge_id, true);
            node_id = match self.data_fwd[node_id].parent {
                Node::Invalid => unreachable!("Nodes are valid"),
                Node::Node(node_id) => node_id,
            };
        }

        result.reverse();
        let mut node_id = meeting_node;
        while let Edge::Edge(edge_id) = self.data_bwd[node_id].inc_edge {
            PathCalculator::unpack_bwd(graph, &mut result, edge_id, false);
            node_id = match self.data_bwd[node_id].parent {
                Node::Invalid => unreachable!("Nodes are valid"),
                Node::Node(node_id) => node_id,
            };
        }
        result.push(end);
        result
    }

    fn unpack_fwd(graph: &FastGraph, nodes: &mut Vec<NodeId>, edge_id: EdgeId, reverse: bool) {
        if !graph.edges_fwd[edge_id].is_shortcut() {
            nodes.push(graph.edges_fwd[edge_id].base_node);
            return;
        }
        if reverse {
            if let Edge::Edge(id) = graph.edges_fwd[edge_id].replaced_out_edge {
                PathCalculator::unpack_fwd(graph, nodes, id, reverse);
            }
            if let Edge::Edge(id) = graph.edges_fwd[edge_id].replaced_in_edge {
                PathCalculator::unpack_bwd(graph, nodes, id, reverse);
            }
        } else {
            if let Edge::Edge(id) = graph.edges_fwd[edge_id].replaced_in_edge {
                PathCalculator::unpack_bwd(graph, nodes, id, reverse);
            }
            if let Edge::Edge(id) = graph.edges_fwd[edge_id].replaced_out_edge {
                PathCalculator::unpack_fwd(graph, nodes, id, reverse);
            }
        }
    }

    fn unpack_bwd(graph: &FastGraph, nodes: &mut Vec<NodeId>, edge_id: EdgeId, reverse: bool) {
        if !graph.edges_bwd[edge_id].is_shortcut() {
            nodes.push(graph.edges_bwd[edge_id].adj_node);
            return;
        }
        if reverse {
            if let Edge::Edge(id) = graph.edges_bwd[edge_id].replaced_out_edge {
                PathCalculator::unpack_fwd(graph, nodes, id, reverse);
            }
            if let Edge::Edge(id) = graph.edges_bwd[edge_id].replaced_in_edge {
                PathCalculator::unpack_bwd(graph, nodes, id, reverse);
            }
        } else {
            if let Edge::Edge(id) = graph.edges_bwd[edge_id].replaced_in_edge {
                PathCalculator::unpack_bwd(graph, nodes, id, reverse);
            }
            if let Edge::Edge(id) = graph.edges_bwd[edge_id].replaced_out_edge {
                PathCalculator::unpack_fwd(graph, nodes, id, reverse);
            }
        }
    }

    fn update_node_fwd(&mut self, node: NodeId, weight: Weight, parent: Node, inc_edge: Edge) {
        self.valid_flags_fwd.set_valid(node);
        self.data_fwd[node].settled = false;
        self.data_fwd[node].weight = weight;
        self.data_fwd[node].parent = parent;
        self.data_fwd[node].inc_edge = inc_edge;
    }

    fn update_node_bwd(&mut self, node: NodeId, weight: Weight, parent: Node, inc_edge: Edge) {
        self.valid_flags_bwd.set_valid(node);
        self.data_bwd[node].settled = false;
        self.data_bwd[node].weight = weight;
        self.data_bwd[node].parent = parent;
        self.data_bwd[node].inc_edge = inc_edge;
    }

    fn is_settled_fwd(&self, node: NodeId) -> bool {
        self.valid_flags_fwd.is_valid(node) && self.data_fwd[node].settled
    }

    fn is_settled_bwd(&self, node: NodeId) -> bool {
        self.valid_flags_bwd.is_valid(node) && self.data_bwd[node].settled
    }

    fn get_weight_fwd(&self, node: NodeId) -> Weight {
        if self.valid_flags_fwd.is_valid(node) {
            self.data_fwd[node].weight
        } else {
            WEIGHT_MAX
        }
    }

    fn get_weight_bwd(&self, node: NodeId) -> Weight {
        if self.valid_flags_bwd.is_valid(node) {
            self.data_bwd[node].weight
        } else {
            WEIGHT_MAX
        }
    }
}

// TODO: I bet `valid_flags` could be removed if this was an Enum.
struct Data {
    settled: bool,
    weight: Weight,
    parent: Node,
    inc_edge: Edge,
}

impl Data {
    fn new() -> Self {
        Data {
            settled: false,
            weight: WEIGHT_MAX,
            parent: Node::Invalid,
            inc_edge: Edge::Invalid,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fast_graph::FastGraphEdge;

    use super::*;

    #[test]
    fn unpack_fwd_single() {
        // 0 -> 1
        let mut g = FastGraph::new(2);
        g.edges_fwd
            .push(FastGraphEdge::new(0, 1, 3, Edge::Invalid, Edge::Invalid));
        let mut nodes = vec![];
        PathCalculator::unpack_fwd(&g, &mut nodes, 0, false);
        assert_eq!(nodes, vec![0]);
    }

    #[test]
    fn unpack_fwd_simple() {
        // 0 -> 1 -> 2
        let mut g = FastGraph::new(3);
        g.edges_fwd
            .push(FastGraphEdge::new(0, 1, 2, Edge::Invalid, Edge::Invalid));
        g.edges_fwd
            .push(FastGraphEdge::new(0, 2, 5, Edge::Edge(0), Edge::Edge(0)));
        g.edges_bwd
            .push(FastGraphEdge::new(2, 1, 3, Edge::Invalid, Edge::Invalid));
        g.first_edge_ids_fwd = vec![0, 2, 0, 0];
        let mut nodes = vec![];
        PathCalculator::unpack_fwd(&g, &mut nodes, 1, false);
        assert_eq!(nodes, vec![1, 0]);
    }
}
