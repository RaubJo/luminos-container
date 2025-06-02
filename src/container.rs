use std::any::{Any, TypeId};
use std::collections::HashMap;
use luminos_contracts::container::Contract;
use luminos_contracts::support::ServiceProvider;

#[derive(Default)]
#[allow(clippy::type_complexity)]
pub struct Container {
    services: HashMap<TypeId, Box<dyn Any>>,
    singletons: HashMap<TypeId, Box<dyn Any>>,
    factories: HashMap<TypeId, Box<dyn Fn(&dyn Contract) -> Box<dyn Any>>>,
    providers: Vec<Box<dyn ServiceProvider>>,
}

impl Container {
    fn new() -> Self {
        Self {
            services: HashMap::new(),
            factories: HashMap::new(),
            singletons: HashMap::new(),
            providers: Vec::new(),
        }
    }

    pub fn bind<T: Any + 'static, F>(&mut self, _marker: T, f: F)
    where
        F: Fn(&dyn Contract) -> T + 'static,
    {
        let factory = move |container: &dyn Contract| -> Box<dyn Any> { Box::new(f(container)) };
        self.bind_factory(TypeId::of::<T>(), Box::new(factory));
    }
}

impl Contract for Container {
    fn register_provider(&mut self, mut provider: Box<dyn ServiceProvider>) {
        provider.register(self);
        self.providers.push(provider);
    }

    fn boot(&mut self) {
        let mut providers = std::mem::take(&mut self.providers);

        for provider in &mut providers {
            provider.boot(self);
        }

        self.providers = providers;
    }

    fn bind_any(&mut self, type_id: TypeId, value: Box<dyn Any>) {
        self.services.insert(type_id, value);
    }

    fn bind_factory(&mut self, type_id: TypeId, factory: Box<dyn Fn(&dyn Contract) -> Box<dyn Any>>) {
        self.factories.insert(type_id, factory);
    }

    fn resolve_any(&self, type_id: TypeId) -> Option<&dyn Any> {
        if self.singletons.contains_key(&type_id) {
            return self.singletons.get(&type_id).map(|boxed| boxed.as_ref())
        }

        self.services.get(&type_id).map(|boxed| boxed.as_ref())
    }

    fn transient(&self, type_id: TypeId) -> Option<Box<dyn Any>> {
        self.factories.get(&type_id).map(|factory| factory(self))
    }

    fn singleton(&mut self, type_id: TypeId, value: Box<dyn Any>) {
        self.singletons.insert(type_id, value);
    }

    fn singleton_factory(&mut self, type_id: TypeId, factory: Box<dyn Fn(&dyn Contract) -> Box<dyn Any>>) {
        self.singletons.insert(type_id, factory(self));
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::any::{Any, TypeId};
    use std::sync::Arc;

    // Test struct to use as a service
    #[derive(Debug, PartialEq)]
    struct TestService {
        value: i32,
    }

    impl TestService {
        fn new(value: i32) -> Self {
            Self { value }
        }
    }

    // Helper to create a container
    fn setup_container() -> Container {
        Container::new()
    }

    #[test]
    fn test_bind_and_transient() {
        let mut container = setup_container();
        let value = 42;

        // Bind a factory for TestService
        container.bind(TestService::new(0), move |_| TestService::new(value));

        // Resolve transient instance
        let instance = container.transient(TypeId::of::<TestService>());
        assert!(instance.is_some());

        let instance = instance.unwrap();
        let service = instance.downcast_ref::<TestService>().unwrap();
        assert_eq!(service.value, value);
    }

    #[test]
    fn test_bind_and_resolve_any() {
        let mut container = setup_container();
        let value = 42;

        // Bind a factory for TestService
        container.bind(TestService::new(0), move |_| TestService::new(value));

        // Resolve any (should not cache since it's not a singleton)
        let instance1 = container.resolve_any(TypeId::of::<TestService>());
        assert!(instance1.is_none()); // No instance cached yet

        // Resolve transient to trigger factory
        let _ = container.transient(TypeId::of::<TestService>());

        // Still not cached in services
        let instance2 = container.resolve_any(TypeId::of::<TestService>());
        assert!(instance2.is_none());
    }

    #[test]
    fn test_singleton() {
        let mut container = setup_container();
        let value = 42;
        let type_id = TypeId::of::<TestService>();

        // Register a singleton instance
        container.singleton(type_id, Box::new(TestService::new(value)));

        // Resolve singleton
        let instance = container.resolve_any(type_id);
        assert!(instance.is_some());

        let service = instance.unwrap().downcast_ref::<TestService>().unwrap();
        assert_eq!(service.value, value);

        // Ensure same instance is returned
        let instance2 = container.resolve_any(type_id);
        assert!(instance2.is_some());
        let service2 = instance2.unwrap().downcast_ref::<TestService>().unwrap();
        assert!(std::ptr::eq(service, service2));
    }

    #[test]
    fn test_singleton_factory() {
        let mut container = setup_container();
        let value = 42;
        let type_id = TypeId::of::<TestService>();

        // Register a singleton factory
        let factory = move |_: &dyn Contract| -> Box<dyn Any> { Box::new(TestService::new(value)) };
        container.singleton_factory(type_id, Box::new(factory));

        // Resolve singleton
        let instance = container.resolve_any(type_id);
        assert!(instance.is_some());

        let service = instance.unwrap().downcast_ref::<TestService>().unwrap();
        assert_eq!(service.value, value);

        // Ensure same instance is returned
        let instance2 = container.resolve_any(type_id);
        assert!(instance2.is_some());
        let service2 = instance2.unwrap().downcast_ref::<TestService>().unwrap();
        assert!(std::ptr::eq(service, service2));
    }

    #[test]
    fn test_mixed_bindings() {
        let mut container = setup_container();
        let singleton_value = 42;
        let transient_value = 99;
        let singleton_type_id = TypeId::of::<TestService>();
        let transient_type_id = TypeId::of::<Arc<TestService>>();

        // Register a singleton
        container.singleton(singleton_type_id, Box::new(TestService::new(singleton_value)));

        // Bind a transient factory
        container.bind(
            Arc::new(TestService::new(0)),
            move |_| Arc::new(TestService::new(transient_value)),
        );

        // Resolve singleton
        let singleton_instance = container.resolve_any(singleton_type_id).unwrap();
        let singleton_service = singleton_instance.downcast_ref::<TestService>().unwrap();
        assert_eq!(singleton_service.value, singleton_value);

        // Resolve transient
        let transient_instance = container.transient(transient_type_id).unwrap();
        let transient_service = transient_instance.downcast_ref::<Arc<TestService>>().unwrap();
        assert_eq!(transient_service.value, transient_value);

        // Ensure singleton persists, transient does not
        let singleton_instance2 = container.resolve_any(singleton_type_id).unwrap();
        assert!(
            std::ptr::eq(
                singleton_instance.downcast_ref::<TestService>().unwrap(),
                singleton_instance2.downcast_ref::<TestService>().unwrap()
            ),
        );

        let transient_resolve = container.resolve_any(transient_type_id);
        assert!(transient_resolve.is_none()); // Transient not cached
    }
}
