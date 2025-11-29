use std::sync::Arc;
use luminos_container::{Container, injectable};
use luminos_contracts::container::Contract;

#[injectable]
impl MyRepository {
    fn new() -> Self {
        Self {}
    }

    fn get(&self, _key: &str) -> String 
    {
        "bar".to_string()
    }

}

#[injectable]
impl MyService {
    fn new(repo: Arc<MyRepository>) -> Self {
        Self { repo }
    }

    fn foo(&self) -> String
    {
        self.repo.get("foo")
    }
}

fn main() {
    let container = Container::new();
    
    let service = container.resolve::<MyService>();
    
    println!("Getting key 'foo' from repository: {}", service.foo());
}
