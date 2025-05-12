use hyperplane::confirmation_layer::ConfirmationNode;

#[tokio::main]
async fn main() {
    println!("Starting Hyperplane node...");
    let _node = ConfirmationNode::new();
    println!("Node created successfully!");
} 