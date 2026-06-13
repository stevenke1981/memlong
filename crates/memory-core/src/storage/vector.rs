use crate::error::{MemoryError, Result};
use std::fs;
use std::path::Path;
use usearch::{Index, IndexOptions, MetricKind, ScalarKind};

const INITIAL_CAPACITY: usize = 1_024;

pub struct VectorStore {
    index: Index,
    dimensions: usize,
    path: String,
}

impl VectorStore {
    pub fn new(path: &str, dimensions: usize) -> Result<Self> {
        if dimensions == 0 {
            return Err(MemoryError::VectorIndex(
                "Embedding dimensions must be greater than zero".to_string(),
            ));
        }

        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)?;
        }

        let index = if Path::new(path).exists() && fs::metadata(path)?.len() > 0 {
            match Index::restore(path) {
                Ok(index) => index,
                Err(_) => migrate_legacy_flat_index(path, dimensions)?,
            }
        } else {
            create_index(dimensions)?
        };

        if index.dimensions() != dimensions {
            return Err(MemoryError::VectorIndex(format!(
                "Dimension mismatch. Configured {}, index contains {}",
                dimensions,
                index.dimensions()
            )));
        }

        if index.capacity() == 0 {
            map_usearch(index.reserve(INITIAL_CAPACITY))?;
        }

        let store = Self {
            index,
            dimensions,
            path: path.to_string(),
        };
        store.persist()?;
        Ok(store)
    }

    pub fn add(&self, id: i64, vector: &[f32]) -> Result<()> {
        self.validate_vector(vector)?;
        let key = to_key(id)?;

        if self.index.contains(key) {
            map_usearch(self.index.remove(key))?;
        }
        if self.index.size() + 1 > self.index.capacity() {
            let next_capacity = (self.index.capacity().max(1) * 2).max(self.index.size() + 1);
            map_usearch(self.index.reserve(next_capacity))?;
        }

        map_usearch(self.index.add(key, vector))?;
        self.persist()
    }

    pub fn search(&self, query: &[f32], top_k: usize) -> Result<Vec<(i64, f32)>> {
        self.validate_vector(query)?;
        if top_k == 0 || self.index.size() == 0 {
            return Ok(Vec::new());
        }

        let matches = map_usearch(self.index.search(query, top_k.min(self.index.size())))?;
        matches
            .keys
            .into_iter()
            .zip(matches.distances)
            .map(|(key, distance)| {
                let id = i64::try_from(key).map_err(|_| {
                    MemoryError::VectorIndex(format!("Vector key {key} exceeds i64 range"))
                })?;
                Ok((id, (1.0 - distance).clamp(-1.0, 1.0)))
            })
            .collect()
    }

    pub fn remove(&self, id: i64) -> Result<()> {
        let key = to_key(id)?;
        if self.index.contains(key) {
            map_usearch(self.index.remove(key))?;
            self.persist()?;
        }
        Ok(())
    }

    pub fn size(&self) -> usize {
        self.index.size()
    }

    fn validate_vector(&self, vector: &[f32]) -> Result<()> {
        if vector.len() != self.dimensions {
            return Err(MemoryError::VectorIndex(format!(
                "Dimension mismatch. Expected {}, got {}",
                self.dimensions,
                vector.len()
            )));
        }
        Ok(())
    }

    fn persist(&self) -> Result<()> {
        map_usearch(self.index.save(&self.path))
    }
}

fn create_index(dimensions: usize) -> Result<Index> {
    let options = IndexOptions {
        dimensions,
        metric: MetricKind::Cos,
        quantization: ScalarKind::F32,
        ..Default::default()
    };
    let index = map_usearch(Index::new(&options))?;
    map_usearch(index.reserve(INITIAL_CAPACITY))?;
    Ok(index)
}

fn migrate_legacy_flat_index(path: &str, dimensions: usize) -> Result<Index> {
    let bytes = fs::read(path)?;
    let record_size = 8 + dimensions * std::mem::size_of::<f32>();
    if record_size == 0 || bytes.len() % record_size != 0 {
        return Err(MemoryError::VectorIndex(format!(
            "Unable to restore USearch index or migrate legacy vector file: {path}"
        )));
    }

    let index = create_index(dimensions)?;
    let record_count = bytes.len() / record_size;
    if record_count > index.capacity() {
        map_usearch(index.reserve(record_count.next_power_of_two()))?;
    }

    for record in bytes.chunks_exact(record_size) {
        let id = i64::from_le_bytes(
            record[..8]
                .try_into()
                .map_err(|_| MemoryError::VectorIndex("Invalid legacy vector ID".to_string()))?,
        );
        let mut vector = Vec::with_capacity(dimensions);
        for value in record[8..].chunks_exact(4) {
            vector.push(f32::from_le_bytes(value.try_into().map_err(|_| {
                MemoryError::VectorIndex("Invalid legacy vector value".to_string())
            })?));
        }
        map_usearch(index.add(to_key(id)?, &vector))?;
    }

    map_usearch(index.save(path))?;
    Ok(index)
}

fn to_key(id: i64) -> Result<u64> {
    u64::try_from(id)
        .map_err(|_| MemoryError::VectorIndex(format!("Vector ID must be non-negative: {id}")))
}

fn map_usearch<T, E: std::fmt::Display>(result: std::result::Result<T, E>) -> Result<T> {
    result.map_err(|error| MemoryError::VectorIndex(error.to_string()))
}
