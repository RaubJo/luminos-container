use luminos_container::{Container, injectable};
use luminos_contracts::container::Contract;
use luminos_contracts::support::ServiceProvider;
use std::sync::Arc;

// ============================================================================
// Domain Models
// ============================================================================

#[injectable]
impl MyRepository {
    fn new() -> Self {
        Self {}
    }

    fn get(&self, _key: &str) -> String {
        "bar".to_string()
    }
}

#[injectable]
impl MyService {
    fn new(repo: Arc<MyRepository>) -> Self {
        Self { repo }
    }

    fn foo(&self) -> String {
        self.repo.get("foo")
    }
}

// ============================================================================
// Service Providers
// ============================================================================

/// Repository Service Provider
/// Registers all repository services
pub struct RepositoryServiceProvider;

impl ServiceProvider<Container> for RepositoryServiceProvider {
    fn register(&self, container: &Container) {
        println!("[RepositoryServiceProvider] Registering repositories...");

        // Register MyRepository
        container.bind::<MyRepository, _>(|_c| {
            println!("  -> Creating MyRepository instance");
            Arc::new(MyRepository::new())
        });
    }

    fn boot(&self, container: &Container) {
        println!("[RepositoryServiceProvider] Booting repositories...");

        // Verify repository is available
        let repo = container.resolve::<MyRepository>();
        println!("  -> Repository test: {}", repo.get("boot-test"));
    }
}

/// Application Service Provider
/// Registers all application services
pub struct ApplicationServiceProvider;

impl ServiceProvider<Container> for ApplicationServiceProvider {
    fn register(&self, container: &Container) {
        println!("[ApplicationServiceProvider] Registering application services...");

        // Register MyService with dependency injection
        container.bind::<MyService, _>(|c| {
            println!("  -> Creating MyService instance with dependencies");
            let repo = c.resolve::<MyRepository>();
            Arc::new(MyService::new(repo))
        });
    }

    fn boot(&self, container: &Container) {
        println!("[ApplicationServiceProvider] Booting application services...");

        // Run any service initialization
        let service = container.resolve::<MyService>();
        println!("  -> Service test: {}", service.foo());
    }
}

/// Logging Service Provider (example of a feature provider)
pub struct LoggingServiceProvider;

impl ServiceProvider<Container> for LoggingServiceProvider {
    fn register(&self, _container: &Container) {
        println!("[LoggingServiceProvider] Registering logging...");
        // In a real app, you'd register a Logger service here
    }

    fn boot(&self, _container: &Container) {
        println!("[LoggingServiceProvider] Logger ready!");
    }
}

// ============================================================================
// Main Application
// ============================================================================

fn main() {
    println!("=== Starting Application ===\n");

    // Create container and register providers
    let container = Container::new();

    println!("--- Registering Providers ---");
    container
        .add_provider(Box::new(LoggingServiceProvider))
        .add_provider(Box::new(RepositoryServiceProvider))
        .add_provider(Box::new(ApplicationServiceProvider));

    println!("\n--- Booting Container ---");
    container.boot();

    println!("\n--- Application Running ---");

    // Now use the services
    let service = container.resolve::<MyService>();
    println!("Result: {}", service.foo());

    println!("\n=== Application Complete ===");
}

// ============================================================================
// Alternative: Builder Pattern
// ============================================================================

#[allow(dead_code)]
fn main_with_builder() {
    println!("=== Starting Application (Builder Pattern) ===\n");

    let container = Container::new()
        .with_provider(Box::new(LoggingServiceProvider))
        .with_provider(Box::new(RepositoryServiceProvider))
        .with_provider(Box::new(ApplicationServiceProvider));

    println!("\n--- Booting Container ---");
    container.boot();

    println!("\n--- Application Running ---");
    let service = container.resolve::<MyService>();
    println!("Result: {}", service.foo());
}

// ============================================================================
// Example: Conditional Provider Registration
// ============================================================================

#[allow(dead_code)]
fn main_with_features() {
    let container = Container::new();

    // Always register core services
    container.add_provider(Box::new(RepositoryServiceProvider));
    container.add_provider(Box::new(ApplicationServiceProvider));

    // Conditionally register based on environment
    if std::env::var("ENABLE_LOGGING").is_ok() {
        container.add_provider(Box::new(LoggingServiceProvider));
    }

    // Boot everything at once
    container.boot();

    let service = container.resolve::<MyService>();
    println!("Result: {}", service.foo());
}
