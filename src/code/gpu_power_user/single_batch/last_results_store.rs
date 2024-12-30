pub struct LatestResultsStore {
    results: HashMap<String, Box<dyn Any + Send + Sync>>,
    type_registry: HashMap<String, TypeId>,
}

impl LatestResultsStore {
    pub fn new(specs: &GpuAccBevyComputeTaskOutputSpecs) -> Self {
        let type_registry = specs
            .specs
            .iter()
            .map(|(k, (_, type_id))| (k.clone(), *type_id))
            .collect();

        Self {
            results: HashMap::new(),
            type_registry,
        }
    }

    // Type-safe getter for users
    pub fn get<T: Pod + 'static>(&self, label: &str) -> Option<&Vec<T>> {
        let expected_type = TypeId::of::<Vec<T>>();
        if self.type_registry.get(label) != Some(&expected_type) {
            return None;
        }
        self.results.get(label)?.downcast_ref()
    }
}
