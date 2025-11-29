use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use luminos_contracts::container::{Injectable, Contract};
use luminos_contracts::support::ServiceProvider;

type Factory = Arc<dyn Fn(&Container) -> Arc<dyn Any + Send + Sync> + Send + Sync>;

#[derive(Default)]
pub struct Container {
    instances: Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
    factories: Mutex<HashMap<TypeId, Factory>>,
    providers: Mutex<Vec<Box<dyn ServiceProvider<Container>>>>
}

impl Container {
    pub fn new() -> Self {
        Self {
            instances: Mutex::new(HashMap::new()),
            factories: Mutex::new(HashMap::new()),
            providers: Mutex::new(Vec::new()),
        }
    }
}

impl Contract for Container {
    fn bind<T, F>(&self, factory: F)
    where
        T: Sized + Send + Sync + 'static,
        F: Fn(&Container) -> Arc<T> + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>(); 
        let boxed_factory: Factory =
            Arc::new(move |c| factory(c) as Arc<dyn Any + Send + Sync>);
        self.factories.lock().unwrap().insert(type_id, boxed_factory);
    }


    fn resolve<T>(&self) -> Arc<T>
    where
        T: Injectable + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        
        if let Some(inst) = self.instances.lock().unwrap().get(&type_id) {
            return inst.clone().downcast::<T>().unwrap();
        }
        
        {
            let factories = self.factories.lock().unwrap();
            if let Some(factory) = factories.get(&type_id) {
                let factory = factory.clone();
                drop(factories); 
                
                let built = factory(self);
                self.instances.lock().unwrap().insert(type_id, built.clone());
                return built.downcast::<T>().unwrap();
            }
        }
        
        T::__register(self);
        
        let factories = self.factories.lock().unwrap();
        if let Some(factory) = factories.get(&type_id) {
            let factory = factory.clone();
            drop(factories);
            
            let built = factory(self);
            self.instances.lock().unwrap().insert(type_id, built.clone());
            return built.downcast::<T>().unwrap();
        }
        
        panic!("Failed to resolve type: {:?}", std::any::type_name::<T>());
    }

    fn add_provider(&self, provider: Box<dyn ServiceProvider<Self> + 'static>) -> &Self {
        self.providers.lock().unwrap().push(provider);
        self
    }

    fn add_providers(&self, providers: Vec<Box<dyn ServiceProvider<Container>>>) -> &Self
    {
        for provider in providers {
            self.add_provider(provider);
        }

        self
    }

    fn boot(&self) -> &Self
    {
        let providers = self.providers.lock().unwrap();
        
        for provider in providers.iter() {
            provider.register(self);
        }
        
        for provider in providers.iter() {
            provider.boot(self);
        }
        
        self
    }

    fn with_provider(self, provider: Box<dyn ServiceProvider<Self> + 'static>) -> Self {
        self.add_provider(provider);
        self
    }

    fn build(&self) -> &Self
    {
        self.boot()
    }
}
