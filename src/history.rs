use crate::engine::{Parameters, Simulation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphRecord {
    pub parameters: Parameters,
    pub timestamp: String,
}
#[derive(Clone)]
pub struct Snapshot {
    pub record: GraphRecord,
    pub simulation: Arc<Simulation>,
}
#[derive(Default)]
pub struct History {
    pub graphs: Vec<Snapshot>,
    pub selected: usize,
    /// Só slots anteriores ao último são revisados no lugar; o último gera um novo.
    pub edit_target: Option<usize>,
    pub overlay: bool,
}
impl History {
    pub fn record(&mut self, snapshot: Snapshot) {
        if let Some(index) = self.edit_target.filter(|i| *i < self.graphs.len()) {
            // Mantém a posição editada; guarda a versão anterior imediatamente à esquerda.
            // Para o slot 2: [A,B,C] -> [B,Bnovo,C]. O mais antigo é descartado.
            if index > 0 {
                self.graphs.remove(0);
                self.graphs.insert(index, snapshot);
            } else {
                self.graphs[0] = snapshot;
            }
            self.selected = index;
        } else {
            if self.graphs.len() == 3 {
                self.graphs.remove(0);
            }
            self.graphs.push(snapshot);
            self.selected = self.graphs.len() - 1;
        }
        self.edit_target = None;
    }
    pub fn select(&mut self, index: usize) -> Option<Snapshot> {
        let snapshot = self.graphs.get(index)?.clone();
        self.selected = index;
        self.edit_target = (index + 1 < self.graphs.len()).then_some(index);
        Some(snapshot)
    }
    pub fn records(&self) -> Vec<GraphRecord> {
        self.graphs.iter().map(|g| g.record.clone()).collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rolling_history_and_middle_revision_match_requested_order() {
        let sim = Arc::new(crate::engine::calculate(&Parameters::default()).unwrap());
        let graph = |n| Snapshot {
            record: GraphRecord {
                parameters: Parameters::default(),
                timestamp: format!("t{n}"),
            },
            simulation: sim.clone(),
        };
        let mut h = History::default();
        for n in 1..=4 {
            h.record(graph(n));
        }
        assert_eq!(
            h.graphs
                .iter()
                .map(|x| x.record.timestamp.as_str())
                .collect::<Vec<_>>(),
            ["t2", "t3", "t4"]
        );
        h.select(1).unwrap();
        h.record(graph(5));
        assert_eq!(h.selected, 1);
        assert_eq!(
            h.graphs
                .iter()
                .map(|x| x.record.timestamp.as_str())
                .collect::<Vec<_>>(),
            ["t3", "t5", "t4"]
        );
        // Repeated edits of the same selected slot continue to preserve its revision.
        h.edit_target = Some(1);
        h.record(graph(6));
        assert_eq!(
            h.graphs
                .iter()
                .map(|x| x.record.timestamp.as_str())
                .collect::<Vec<_>>(),
            ["t5", "t6", "t4"]
        );
        h.select(2);
        h.record(graph(7));
        assert_eq!(
            h.graphs
                .iter()
                .map(|x| x.record.timestamp.as_str())
                .collect::<Vec<_>>(),
            ["t6", "t4", "t7"]
        );
    }
}
