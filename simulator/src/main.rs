// Set up network nodes
let nodes = network::setup_nodes(&chain_ids, &chain_delays, block_interval).await;
let cl_node = nodes[0].clone(); 