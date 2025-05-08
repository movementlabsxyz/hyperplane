use hyperplane::confirmation::ConfirmationNode;

#[tokio::main]
async fn main() {
    println!("Starting Hyperplane node...");
    let node = ConfirmationNode::new();
    println!("Node created successfully!");
} 