use std::f64;
use math::mean;
use petgraph::graphmap::DiGraphMap;
use std::collections::{HashSet, HashMap};
use std::time::Instant;
use petgraph::visit::{Reversed, Bfs, Dfs};
use petgraph::{Incoming, Outgoing};
use angora_common::tag::TagSeg;
use super::fparse::CfgFile;

pub type CmpId = u32;
pub type BbId = u32;
pub type CallSiteId = u32;
pub type Edge = (BbId, BbId);
pub type Score = u32;
pub type FixedBytes = Vec<(usize, u8)>;

const TARGET_SCORE: Score = 0;
const UNDEF_SCORE: Score = std::u32::MAX;


#[derive(Clone)]
pub struct ControlFlowGraph {
    graph: DiGraphMap<BbId, Score>,
    targets: HashSet<CmpId>,
    id_mapping: HashMap<BbId, HashSet<CmpId>>,
    reverse_id_mapping: HashMap<CmpId, BbId>,
    solved_targets: HashSet<CmpId>,
    indirect_edges: HashSet<Edge>,
    callsite_edges: HashMap<CallSiteId, HashSet<Edge>>,
    callsite_dominators: HashMap<CallSiteId, HashSet<CmpId>>,
    dominator_cmps: HashSet<CmpId>,
    magic_bytes: HashMap<Edge, FixedBytes>,
}



// A CFG of branches (CMPs)
impl ControlFlowGraph {
    //pub fn new(targets: HashSet<CmpId>) -> ControlFlowGraph {
    pub fn new(data: CfgFile) -> ControlFlowGraph {
        let mut dominator_cmps = HashSet::new();
        for s in data.callsite_dominators.values() {
            dominator_cmps.extend(s)
        }
        let mut result = ControlFlowGraph {
            graph: DiGraphMap::new(),
            targets: data.targets,
            id_mapping: data.id_mapping.clone(),
            reverse_id_mapping: Self::reverse_id_mapping(data.id_mapping),
            solved_targets: HashSet::new(),
            indirect_edges: HashSet::new(),
            callsite_edges: HashMap::new(),
            callsite_dominators: data.callsite_dominators,
            dominator_cmps,
            magic_bytes: HashMap::new(),
        };

        for e in data.edges {
            result.init_add_edge(e);
        }
        result.init_prop_targets();

        info!("INIT CFG: dominators: {:?}", result.dominator_cmps);
        info!("INIT ID mapping: {:?}", result.id_mapping);

        result
    }

    pub fn empty_new() -> ControlFlowGraph {
        let result = ControlFlowGraph {
            graph: DiGraphMap::new(),
            targets: HashSet::new(),
            id_mapping: HashMap::new(),
            reverse_id_mapping: HashMap::new(),
            solved_targets: HashSet::new(),
            indirect_edges: HashSet::new(),
            callsite_edges: HashMap::new(),
            callsite_dominators: HashMap::new(),
            dominator_cmps: HashSet::new(),
            magic_bytes: HashMap::new(),
        };

        result
    }


    pub fn add_edge(&mut self, edge: Edge) -> bool {
        let result = !self.has_edge(edge);
        self.handle_new_edge(edge);
        debug!("Added CFG edge {:?} {}", edge, self.targets.contains(&edge.1));
        result
    }

    fn init_add_edge(&mut self, edge: Edge) {
        let (src, dst) = edge;
        let init_score = UNDEF_SCORE;
        self.graph.add_edge(src, dst, init_score);

        if let Some(cmp_set) = &self.id_mapping.get(&src) {
            for cmp in *cmp_set {
                if self.targets.contains(&cmp) {
                    self.graph.add_edge(src, dst, TARGET_SCORE);
                }
            }
        }
    }

    fn init_prop_targets(&mut self) {
        // should be possible without cloning
        for target in &self.targets.clone() {
            if let Some(&bb) = self.get_bb_from_cmp(&target) {
                self.propagate_score(bb);
            }
        }
    }

    pub fn set_edge_indirect(&mut self, edge: Edge, callsite: CallSiteId) {
        self.indirect_edges.insert(edge);
        let entry = self.callsite_edges.entry(callsite).or_insert(HashSet::new());
        entry.insert(edge);
    }

    pub fn set_magic_bytes(&mut self, edge: Edge, buf: &Vec<u8>, offsets: &Vec<TagSeg>) {
        let mut fixed = vec![];
        let mut indices = HashSet::new();
        for tag in offsets {
            for i in tag.begin .. tag.end {
                indices.insert(i as usize);
            }
        }
        for i in indices {
            fixed.push((i, buf[i]));
        }
        self.magic_bytes.insert(edge, fixed);
    }

    pub fn get_magic_bytes(&self, edge: Edge) -> FixedBytes {
        if let Some(fixed) = self.magic_bytes.get(&edge) {
            return fixed.clone();
        }
        return vec![];
    }

    pub fn dominates_indirect_call(&self, cmp: CmpId) -> bool{
        self.dominator_cmps.contains(&cmp)
    }

    pub fn get_callsite_dominators(&self, cs: CallSiteId) -> HashSet<CmpId> {
        let res = self.callsite_dominators.get(&cs);
        debug!("GET CALLSITE DOM: {}, {:?}", cs, res);
        if let Some(s) = res {
            return s.clone();
        }
        let result = HashSet::new();
        return result;
    }

    pub fn remove_target(&mut self, cmp: CmpId) {
        if self.targets.remove(&cmp) {
            if let Some(&bb) = self.get_bb_from_cmp(&cmp) {
                self.propagate_score(bb);
            }
            else {
                warn!("CFG warning: couldn't propagate score when removing target");
            }
            self.solved_targets.insert(cmp);
        }
    }

    pub fn is_target(&self, cmp: CmpId) -> bool {
        self.targets.contains(&cmp) || self.solved_targets.contains(&cmp)
    }

    pub fn get_bb_from_cmp(&self, cmp: &CmpId) -> Option<&BbId> {
        return self.reverse_id_mapping.get(cmp);
    }

    fn handle_new_edge(&mut self, edge: Edge) {
        let (src, dst) = edge;

        // 1) Get score for dst
        let dst_score = self._score_for_bb(dst);
        
        // 2) if src_score changed
        let old_src_score = self._score_for_bb(src);

        // Insert edge in graph
        self.graph.add_edge(src, dst, dst_score);

        let new_src_score = self._score_for_bb(src);

        if old_src_score == new_src_score {
            // No change in score
            return;
        }

        self.graph.add_edge(src, dst, dst_score);
        self.propagate_score(src);
    }


    fn propagate_score(&mut self, bb: BbId) {
        // Check how long it takes to clone the full graph. Result: On my machine the cloning part of adding all the edges to the graph at the beginning takes around 25% of the total time. Definitely expensive but probably not worth to change it right now.
        #[cfg(test)]
        let now = Instant::now();
        let graph_copy = &self.graph.clone();
        #[cfg(test)]
        println!("time to copy with node size {:?} and edge size {:?}: {}", &self.graph.node_count(), &self.graph.edge_count(), now.elapsed().as_micros());
        #[cfg(test)]
        let now2 = Instant::now();
        let rev_graph = Reversed(graph_copy);
        let mut visitor = Bfs::new(rev_graph, bb);

        while let Some(visited) = visitor.next(rev_graph) {
            let new_score = self._score_for_bb(visited);
            let mut predecessors = vec![];
            {
                let neighbors = self.graph.neighbors_directed(visited, Incoming);
                for n in neighbors {
                    predecessors.push(n);
                }
            }
            for p in predecessors {
                self.graph.add_edge(p, visited, new_score);
            }
        }
        #[cfg(test)]
        println!("time for rest of propagate: {}", now2.elapsed().as_micros());
    }
    

    pub fn has_edge(&self, edge: Edge) -> bool {
        let (a,b) = edge;
        self.graph.contains_edge(a, b)
    }

    pub fn has_score(&self, bb: BbId) -> bool {
        if self._score_for_bb(bb) != UNDEF_SCORE {
            return true;
        } 
        false
    }

    fn reverse_id_mapping(id_mapping: HashMap<BbId, HashSet<CmpId>>) -> HashMap<CmpId, BbId> {
        let mut rev_mapping = HashMap::new();
        for (bbid, cmp_set) in id_mapping.iter() {
            for cmp in cmp_set {
                rev_mapping.insert(*cmp, *bbid);
            }
        }
        return rev_mapping;
    }

    fn aggregate_score(ovals: Vec<Score>) -> Score {
        //Self::score_greedy(ovals)
        //Self::score_coverage(ovals)
        Self::score_harmonic_mean(ovals)
    }

    fn score_harmonic_mean(ovals: Vec<Score>) -> Score {
        if ovals.len() == 0 {
            return UNDEF_SCORE;
        }
        let vals = ovals.into_iter().filter(|v| *v != UNDEF_SCORE);
        let fvals : Vec<f64> = vals.into_iter().map(|x| x as f64).collect();
        if fvals.is_empty() {
            return UNDEF_SCORE;
        }
        return mean::harmonic(fvals.as_slice()) as u32;
    }

    #[allow(dead_code)]
    fn score_greedy(ovals: Vec<Score>) -> Score {
        let vals = ovals.into_iter().filter(|v| *v != UNDEF_SCORE);
        if let Some(v) = vals.min() {
            v + 1
        } else {
            UNDEF_SCORE
        }
    }

    #[allow(dead_code)]
    fn score_coverage(ovals: Vec<Score>) -> Score {
        if ovals.len() == 0 {
            return UNDEF_SCORE;
        }
        let vals = ovals.into_iter().filter(|v| *v != UNDEF_SCORE);
        let vals_norm = vals.into_iter().map(|v| if v == TARGET_SCORE {1} else {v});
        vals_norm.sum()
    }

    fn has_path_to_target_bb(&self, start: BbId) -> bool {
        let distance = &self.score_for_bb(start);
        // If distance is not UNDEF we know it has a path if the scores have propagated properly
        if *distance != UNDEF_SCORE {
            return true
        }
        false
    }

    pub fn has_path_to_target(&self, cmp: CmpId) -> bool {
        if let Some(&bb) = self.get_bb_from_cmp(&cmp) {
            return self.has_path_to_target_bb(bb);
        }
        false
    }

    pub fn score_for_bb(&self, bb: CmpId) -> Score {
        let score = self._score_for_bb(bb);
        if score != UNDEF_SCORE {
            debug!("Calculated score: {}", score);
        }
        score
    }

    pub fn score_for_bb_inp(&self, bb: BbId, inp: Vec<u8>) -> Score {
        let score = self._score_for_bb_inp(bb, inp);
        if score != UNDEF_SCORE {
            debug!("Calculated score: {}", score);
        }
        score
    }

    fn _score_for_bb(&self, bb: BbId) -> Score {
        self._score_for_bb_inp(bb, vec![])
    }

    fn _score_for_bb_inp(&self, bb: BbId, inp: Vec<u8>) -> Score {
        // Get the cmpid of the bbid if there is one
        let mut has_cmp = false;
        let mut num_cmps = 1;
        if let Some(cmp_set) = &self.id_mapping.get(&bb) {
            has_cmp = true;
            num_cmps = cmp_set.len() as u32;
            for cmp in *cmp_set {
                if self.targets.contains(&cmp) {
                    debug!("Calculate score for target: {}", cmp);
                    return TARGET_SCORE;
                }
            }
        }
        let mut neighbors = self.graph.neighbors_directed(bb, Outgoing);

        let mut scores = vec![];
        while let Some(n) = neighbors.next() {
            let edge = (bb, n);
            if !self._should_count_edge(edge, &inp) {
                debug!("Skipping count edge: {:?}", edge);
                continue;
            }
            debug!("Counting edge: {:?}", edge);
            if let Some(s) = self.graph.edge_weight(bb, n) {
                scores.push(*s);
            }
        }
        let aggregate = Self::aggregate_score(scores.clone());
        // increase distance by number of cmpids when passing by.
        if has_cmp && aggregate != UNDEF_SCORE {aggregate+num_cmps} else {aggregate}
    }

    fn _should_count_edge(&self, edge: Edge, inp: &Vec<u8>) -> bool {
        if !self.indirect_edges.contains(&edge) {
            return true;
        }

        if let Some(fixed) = self.magic_bytes.get(&edge) {
            let mut equal = true;
            for (i, v) in fixed {
                if let Some(b) = inp.get(*i) {
                    if *b != *v {
                        equal = false;
                        break;
                    }
                } 
            }
            return equal;
        }

        true
    }

  
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::FromIterator;
    use crate::itertools::Itertools;
    use rand::thread_rng;
    use rand::seq::SliceRandom;

    fn test_new(targets: HashSet<CmpId>, id_mapping: HashMap<BbId, HashSet<CmpId>>) -> ControlFlowGraph {
        let result = ControlFlowGraph {
            graph: DiGraphMap::new(),
            targets: targets,
            id_mapping: id_mapping.clone(),
            reverse_id_mapping: ControlFlowGraph::reverse_id_mapping(id_mapping),
            solved_targets: HashSet::new(),
            indirect_edges: HashSet::new(),
            callsite_edges: HashMap::new(),
            callsite_dominators: HashMap::new(),
            dominator_cmps: HashSet::new(),
            magic_bytes: HashMap::new(),
        };

        result
    }

    #[test]
    fn cfg_basic() {
        // Create CFG
        let mut cfg = ControlFlowGraph::empty_new();
        let edges = vec![(10,20), (20,30), (10,40), (40,50), (20,30)];

        for e in edges.clone() {
            cfg.add_edge(e);
        }
    }
    
    #[test]
    fn cfg_target_mapping_simple() {
        // Create CFG
        let target_vec = vec![1100, 1200];
        let targets = HashSet::from_iter(target_vec.iter().cloned());

        let id_mapping: HashMap<BbId, HashSet<CmpId>> = [(10, vec![1000].into_iter().collect()), (50, vec![1100].into_iter().collect()), (80, vec![1200].into_iter().collect())].iter().cloned().collect();

        let mut cfg = test_new(targets, id_mapping);
        let edges = vec![(0,10), (10, 20), (20,30), (30,40), (40,50), (10,60), (60,70), (70,80)];

        // Test adding BBId edges
        for e in edges.clone() {
            cfg.init_add_edge(e);
        }
        cfg.init_prop_targets();
        for e in edges.clone() {
            let (from, to) = e;
            println!("weight for ({:?}, {:?}): {:?}", from, to, cfg.graph.edge_weight(from, to))
        }
    }

    #[test]
    fn cfg_target_mapping_complex() {
        // Create CFG
        let target_vec = vec![1700];
        let targets = HashSet::from_iter(target_vec.iter().cloned());

        let id_mapping: HashMap<BbId, HashSet<CmpId>> = [(10, vec![1000].into_iter().collect()), (20, vec![1100].into_iter().collect()), (50, vec![1200].into_iter().collect()), (60, vec![1300].into_iter().collect()), (140, vec![1400].into_iter().collect()), (160, vec![1500].into_iter().collect()), (100, vec![1600].into_iter().collect()), (180, vec![1700, 1800].into_iter().collect())].iter().cloned().collect();

        let mut cfg = test_new(targets, id_mapping);
        let edges = vec![(0,10), (10, 20), (20,30), (30,50), (50,60), (60,130), (60,140), (140,150), (140,160), (160,170), (160,180), (50,70), (20,40), (40,80), (10,90), (90,100), (100,110), (100,120)];

        // Test adding BBId edges
        for e in edges.clone() {
            cfg.init_add_edge(e);
        }
        cfg.init_prop_targets();
        for e in edges.clone() {
            let (from, to) = e;
            println!("weight for ({:?}, {:?}): {:?}", from, to, cfg.graph.edge_weight(from, to))
        }
    }

    // Test whether or not the graph clone in propagate_score is too time consuming.
    #[test]
    fn cfg_clone_time() {
        // Create CFG
        let num_nodes = 1000000;
        let target_vec = vec![num_nodes*10];
        let targets: HashSet<CmpId> = HashSet::from_iter(target_vec.iter().cloned());
        let id_mapping: HashMap<BbId, HashSet<CmpId>> = [(num_nodes-1, vec![num_nodes*10].into_iter().collect())].iter().cloned().collect();

        let mut cfg = test_new(targets, id_mapping);
        let nodes: Vec<BbId> = (0..num_nodes).collect();
        
        let mut edges = vec![];
        for (a,b) in nodes.clone().into_iter().tuple_windows() {
            edges.push((a,b));
        }

        edges.shuffle(&mut thread_rng());
        
        let now = Instant::now();
        for e in edges.clone() {
            cfg.init_add_edge(e);
        }
        cfg.init_prop_targets();
        println!("total time: {}", now.elapsed().as_micros());
    }

    // Test whether or not has_path_to_target_bb works
    #[test]
    fn cfg_path_to_target() {
        // Create CFG
        let target_vec = vec![1700];
        let targets = HashSet::from_iter(target_vec.iter().cloned());

        let id_mapping: HashMap<BbId, HashSet<CmpId>> = [(10, vec![1000].into_iter().collect()), (20, vec![1100].into_iter().collect()), (50, vec![1200].into_iter().collect()), (60, vec![1300].into_iter().collect()), (140, vec![1400].into_iter().collect()), (160, vec![1500].into_iter().collect()), (100, vec![1600].into_iter().collect()), (180, vec![1700].into_iter().collect())].iter().cloned().collect();

        let mut cfg = test_new(targets, id_mapping);
        let edges = vec![(0,10), (10, 20), (20,30), (30,50), (50,60), (60,130), (60,140), (140,150), (140,160), (160,170), (160,180), (50,70), (20,40), (40,80), (10,90), (90,100), (100,110), (100,120)];

        // Adding BBId edges
        for e in edges.clone() {
            cfg.init_add_edge(e);
        }
        cfg.init_prop_targets();

        // Test path to target
        assert_eq!(cfg.has_path_to_target_bb(0), true);
        assert_eq!(cfg.has_path_to_target_bb(90), false);
        assert_eq!(cfg.has_path_to_target_bb(80), false);
        assert_eq!(cfg.has_path_to_target_bb(30), true);
        assert_eq!(cfg.has_path_to_target_bb(140), true);
    }
}

