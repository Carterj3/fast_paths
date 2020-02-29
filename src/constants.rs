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

pub type NodeId = usize;
pub type EdgeId = usize;
pub type Weight = usize;

#[cfg(feature = "invalid_usize")]
mod maximum {
    pub const INVALID_NODE: usize = std::usize::MAX;
    pub const INVALID_EDGE: usize = std::usize::MAX;
    pub const WEIGHT_MAX: usize = std::usize::MAX;
}

#[cfg(feature = "invalid_u32")]
mod maximum {
    pub const INVALID_NODE: u32 = std::u32::MAX;
    pub const INVALID_EDGE: u32 = std::u32::MAX;
    pub const WEIGHT_MAX: u32 = std::u32::MAX;
}

#[cfg(feature = "invalid_u64")]
mod maximum {
    pub const INVALID_NODE: u64 = std::u64::MAX;
    pub const INVALID_EDGE: u64 = std::u64::MAX;
    pub const WEIGHT_MAX: u64 = std::u64::MAX;
}

pub const INVALID_NODE: NodeId = maximum::INVALID_NODE as NodeId;
pub const INVALID_EDGE: EdgeId = maximum::INVALID_EDGE as EdgeId;
pub const WEIGHT_MAX: Weight = maximum::WEIGHT_MAX as Weight;
pub const WEIGHT_ZERO: Weight = 0;
