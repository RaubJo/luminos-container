# luminos Container

A lightweight dependency injection container for Rust, designed to manage services, factories, and singletons with type-safe bindings. This container supports binding factories, registering singletons, and resolving instances, making it suitable for building modular and testable applications.

## Features

- **Type-Safe Bindings**: Bind factories to create instances of specific types.
- **Singleton Support**: Register singletons to ensure a single instance is reused.
- **Transient Factories**: Create new instances on each resolution.
- **Service Providers**: Register and boot service providers for modular configuration.
- **Dynamic Resolution**: Resolve instances by type using `TypeId`.

## Usage

The `Container` struct is the core of the library. Below are examples demonstrating how to use its key functionalities.

### Setup

Create a new `Container` instance:

```rust
use container::Container;

let mut container = Container::new();
```

### Binding a Factory

Bind a factory to create new instances of a type (transient instances):

```rust
#[derive(Debug, PartialEq)]
struct TestService {
    value: i32,
}

impl TestService {
    fn new(value: i32) -> Self {
        Self { value }
    }
}

container.bind(TestService::new(0), |_: &dyn Contract| TestService::new(42));
```

Resolve a transient instance:

```rust
let instance = container.transient(TypeId::of::<TestService>()).unwrap();
let service = instance.downcast_ref::<TestService>().unwrap();
assert_eq!(service.value, 42);
```

### Registering a Singleton

Register a singleton instance to ensure only one instance is created and reused:

```rust
let type_id = TypeId::of::<TestService>();
container.singleton(type_id, Box::new(TestService::new(42)));

let instance = container.resolve_any(type_id).unwrap();
let service = instance.downcast_ref::<TestService>().unwrap();
assert_eq!(service.value, 42);

// Subsequent resolutions return the same instance
let instance2 = container.resolve_any(type_id).unwrap();
assert_eq!(
    std::ptr::eq(
        instance.downcast_ref::<TestService>().unwrap(),
        instance2.downcast_ref::<TestService>().unwrap()
    ),
    true
);
```

### Using a Singleton Factory

Register a singleton via a factory, which is executed once to create the instance:

```rust
let type_id = TypeId::of::<TestService>();
let factory = |_: &dyn Contract| -> Box<dyn Any> { Box::new(TestService::new(42)) };
container.singleton_factory(type_id, Box::new(factory));

let instance = container.resolve_any(type_id).unwrap();
let service = instance.downcast_ref::<TestService>().unwrap();
assert_eq!(service.value, 42);

// Same instance is reused
let instance2 = container.resolve_any(type_id).unwrap();
assert_eq!(
    std::ptr::eq(
        instance.downcast_ref::<TestService>().unwrap(),
        instance2.downcast_ref::<TestService>().unwrap()
    ),
    true
);
```

### Registering Service Providers

Service providers allow modular configuration of the container:

```rust
use luminos_contracts::support::ServiceProvider;

struct MyProvider;

impl ServiceProvider for MyProvider {
    fn register(&mut self, container: &mut dyn Contract) {
        container.bind_any(TypeId::of::<TestService>(), Box::new(TestService::new(100)));
    }

    fn boot(&mut self, _container: &mut dyn Contract) {
        // Perform boot-time initialization
    }
}

container.register_provider(Box::new(MyProvider));
container.boot();
```

## Notes

- Singletons are stored separately from transient services to ensure proper instance management.
- The container uses `TypeId` for type-safe resolution, so ensure types are unique and correctly registered.
- Factories are executed each time for transient resolutions but only once for singleton factories.

## Testing

The library includes a comprehensive test suite to verify the behavior of bindings, singletons, and service providers. To run the tests:

```bash
cargo test
```

## License

This project is licensed under the MIT License.
