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

use serde::{Deserialize, Serialize};

pub type NodeId = usize;
pub type EdgeId = usize;
pub type Weight = usize;

#[derive(Eq, PartialEq, Clone, Copy, Deserialize, Serialize, Debug)]
pub enum Node {
    Invalid,
    Node(NodeId),
}

impl Node {
    pub fn has_id(&self, id: NodeId) -> bool {
        match self {
            Node::Invalid => false,
            Node::Node(node_id) => *node_id == id,
        }
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Deserialize, Serialize, Debug)]
pub enum Edge {
    Invalid,
    Edge(EdgeId),
}

impl Edge {
    pub fn has_id(&self, id: EdgeId) -> bool {
        match self {
            Edge::Invalid => false,
            Edge::Edge(edge_id) => *edge_id == id,
        }
    }
}

pub const INVALID_NODE: Node = Node::Invalid;
pub const INVALID_EDGE: Edge = Edge::Invalid;

pub const WEIGHT_MAX: Weight = std::usize::MAX;
pub const WEIGHT_ZERO: Weight = 0;
