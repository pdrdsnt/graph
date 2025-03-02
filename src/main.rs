use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use graph;
use graph::pathfinder::Pathfind;
use graph::graph::Connection;
use graph::graph::Graph;
use graph::graph::IntoConnections;
use graph::edge::Edge;

fn main() {
    println!("Hello, world!");
    let tile_map = TileMap::new(10);
    println!("Original Map:");
    tile_map.print_grid();

    let mut graph = Graph::<Vector2, Tile, u32>::new(HashMap::new());
    for tile in &tile_map.map {
        graph.add_node(tile.read().unwrap().position, tile.clone());
    }
    let best_paths = graph.bellman_ford(&Vector2 { a: 0, b: 0 }).unwrap();
    if best_paths.is_empty() {
        println!("No path found.");
        return;
    }
    // Loop over each best path (acyclic paths)
    println!("\n--- Best Paths ---");

    for k in best_paths.keys() {
        println!(
            "Path to {:?} with cost {}",
            k,
            best_paths.get(k).unwrap().0
        );
        let path_set: std::collections::HashSet<Vector2> = best_paths.get(k).unwrap().1
            .iter()
            .map(|e| e.b)
            .collect();

        tile_map.print_grid_with_path(&path_set);
        println!("--------------------------");
    }
}

#[derive(Debug)]
pub struct TileMap {
    size: i64,
    map: Vec<Arc<RwLock<Tile>>>,
}

impl TileMap {
    pub fn new(size: i64) -> Self {
        let mut map = Vec::new();
        for y in 0..size {
            for x in 0..size {
                let pos = Vector2 { a: x, b: y };
                let tile = Tile {
                    tile_type: if random_bool() {
                        TileType::Path
                    } else {
                        TileType::Block
                    },
                    position: pos,
                };
                map.push(Arc::new(RwLock::new(tile)));
            }
        }

        TileMap { size, map }
    }

    pub fn print_grid(&self) {
        for y in 0..self.size {
            for x in 0..self.size {
                if x == 0 && y == 0 {
                    print!("S ");
                    continue;
                }

                let index = (y * self.size + x) as usize;
                let tile = &self.map[index];
                let ch = match tile.read().unwrap().tile_type {
                    TileType::Path => "  ",  // Dois espaços para caminho aberto
                    TileType::Block => "██", // Blocos para tiles bloqueados
                };
                print!("{}", ch);
            }
            println!();
        }
    }

    pub fn print_grid_with_path(&self, path: &std::collections::HashSet<Vector2>) {
        for y in 0..self.size {
            for x in 0..self.size {
                let pos = Vector2 { a: x, b: y };
                // Mark the starting point with "S "
                if x == 0 && y == 0 {
                    print!("S ");
                    continue;
                }
                // If the current tile is part of the given path, print a special char.
                if path.contains(&pos) {
                    print!("##");
                } else {
                    let index = (y * self.size + x) as usize;
                    let tile = &self.map[index];
                    let ch = match tile.read().unwrap().tile_type {
                        TileType::Path => "  ",
                        TileType::Block => "██",
                    };
                    print!("{}", ch);
                }
            }
            println!();
        }
    }
}
#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy, PartialOrd, Ord)]
pub struct Vector2 {
    a: i64,
    b: i64,
}

use std::fmt;
use std::ops::{Add, Sub};
impl fmt::Display for Vector2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.a, self.b)
    }
}

impl Sub for Vector2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            a: self.a - other.a,
            b: self.b - other.b,
        }
    }
}
impl Add for Vector2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            a: self.a + other.a,
            b: self.b + other.b,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TileType {
    Path,
    Block,
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tile {
    pub tile_type: TileType,
    pub position: Vector2,
}

impl IntoConnections<Vector2, u32> for Tile {
    type Item = TileConnection;

    fn into_connections(&self, map: &HashMap<Vector2, Arc<RwLock<Self>>>) -> Vec<Self::Item> {
        let mut dirs = vec![
            Vector2 { a: -1, b: -1 },
            Vector2 { a: 0, b: -1 },
            Vector2 { a: 1, b: -1 },
            Vector2 { a: -1, b: 0 },
            Vector2 { a: 1, b: 0 },
            Vector2 { a: -1, b: 1 },
            Vector2 { a: 0, b: 1 },
            Vector2 { a: 1, b: 1 },
        ];
        dirs.retain(|dir| {
            let pos = self.position + *dir;
            // The target must be on the map.
            if !map.contains_key(&pos) {
                return false;
            }
            // And the tile at the target must not be blocked.
            if let Some(tile_lock) = map.get(&pos) {
                if tile_lock.read().unwrap().tile_type == TileType::Block {
                    return false;
                }
            }
            true
        });

        let r = dirs
            .into_iter()
            .map(|dir| {
                // If both components are non-zero, it's a diagonal move.
                let cost = if dir.a != 0 && dir.b != 0 {
                    1414 // diagonal move (approximation of 1000 * √2)
                } else {
                    1000 // orthogonal move
                };
                TileConnection {
                    orign: self.position,
                    dir,
                    cost,
                }
            })
            .collect();

        println!("Tile at {:?} has connections: {:?}", self.position, r);

        r
    }
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TileConnection {
    pub orign: Vector2,
    pub dir: Vector2,
    pub cost: u32,
}

impl Connection<Vector2, u32> for TileConnection {
    fn into_edge(&self, id: usize) -> Edge<Vector2, u32> {
        Edge {
            id,
            a: self.orign,
            b: self.orign + self.dir,
            h: self.cost,
        }
    }
}

fn random_bool() -> bool {
    // Obtém o tempo atual em nanossegundos desde a época UNIX.
    let now = SystemTime::now();
    let duration = now
        .duration_since(UNIX_EPOCH)
        .expect("Falha ao obter a duração desde a época UNIX");
    let nanos = duration.subsec_nanos();

    // Retorna 'true' ou 'false' com base na paridade dos nanossegundos.
    nanos % 2 == 0
}
