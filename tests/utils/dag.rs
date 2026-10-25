use super::*;

/// Depth-first walk from the operation `from` up to its ancestors, returning each visited
/// operation once, in visit order.
pub fn dag_ancestors(dag: &OpsDag, from: OpId) -> Vec<OpId> {
    let mut stack = vec![dag.op_node(&from).expect("operation in DAG")];
    let mut visited = HashSet::new();
    let mut order = vec![];
    while let Some(node) = stack.pop() {
        if !visited.insert(node) {
            continue;
        }
        order.push(dag.opid(node));
        for (_, parent) in dag.spent_opouts(node) {
            if !visited.contains(&parent) {
                stack.push(parent);
            }
        }
    }
    order
}

/// Checks that the DAG has exactly one node per consignment operation and one edge
/// per transition input, that each edge links an input to its spender, and that no opout is
/// spent more than once.
pub fn assert_dag_matches_consignment<const TRANSFER: bool>(
    dag: &OpsDag,
    consignment: &Consignment<TRANSFER>,
) {
    let genesis_opid = OpId::from(*consignment.contract_id());
    let transitions = consignment
        .bundles
        .iter()
        .flat_map(|wb| wb.bundle.known_transitions.iter())
        .map(|KnownTransition { opid, transition }| (*opid, transition))
        .collect::<Vec<_>>();

    assert_eq!(dag.node_count(), 1 + transitions.len());
    let genesis_node = dag.op_node(&genesis_opid).expect("genesis in DAG");
    assert_eq!(dag.spent_opouts(genesis_node).count(), 0);

    let mut input_count = 0;
    let mut all_spent = HashSet::new();
    for (opid, transition) in &transitions {
        let node = dag.op_node(opid).expect("transition in DAG");
        let inputs = transition.inputs.into_iter().collect::<HashSet<_>>();
        let mut spent = HashSet::new();
        for (opout, parent) in dag.spent_opouts(node) {
            assert_eq!(dag.opid(parent), opout.op);
            assert!(all_spent.insert(opout), "{opout} is spent more than once");
            spent.insert(opout);
        }
        assert_eq!(
            spent, inputs,
            "DAG parents of {opid} differ from its inputs"
        );
        input_count += inputs.len();
    }
    assert_eq!(dag.edge_count(), input_count);
}
