

pub struct Registry<T: Registerable> {
    pub data: Vec<T>,
}

impl<T: Registerable> Registry<T> {
    pub fn new() -> Registry<T> {
        Self{
            data: Vec::<T>::new(),
        }
    }

    pub fn add(&mut self, mut item: T) -> u32 {
        let registry_id = self.data.len() as u32;
        item.register(registry_id);
        self.data.push( item );
        registry_id
    }

    pub fn get(&self, index: u32) -> Option<&T> {
        self.data.get(index as usize)
    }

    pub fn get_size(&self) -> usize {
        self.data.len()
    }
}

pub trait Registerable {
    fn register(&mut self, id: u32);
}
