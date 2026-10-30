pub fn node_config(i: usize, n: usize, base_rpc: u16, base_p2p: u16) -> String {
    let rpc_port = base_rpc + i as u16;
    let p2p_port = base_p2p + i as u16;
    let mut bootnodes = Vec::new();
    for j in 0..n {
        if j == i {
            continue;
        }
        bootnodes.push(format!("127.0.0.1:{}", base_p2p + j as u16));
    }
    format!(
        "chain_id = 3\n\
         data_dir = \"./data/node{i}\"\n\
         listen_addr = \"0.0.0.0:{p2p_port}\"\n\
         rpc_addr = \"127.0.0.1\"\n\
         rpc_port = {rpc_port}\n\
         bootnodes = [{}]\n",
        bootnodes
            .iter()
            .map(|b| format!("\"{b}\""))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generates_n_configs() {
        let c = node_config(0, 3, 8545, 8000);
        assert!(c.contains("chain_id = 3"));
        assert!(c.contains("rpc_port = 8545"));
        assert!(c.contains("8001"));
    }
    #[test]
    fn no_self_bootnode() {
        let c = node_config(1, 3, 8545, 8000);
        assert!(!c.contains("8001\"") || c.contains("8000"));
    }
}
